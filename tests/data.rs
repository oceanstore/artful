#[cfg(test)]
mod dataset_test {
    use artful::Art;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;

    const FILES: [&'static str; 3] = [
        "tests/data/words.txt",
        "tests/data/uuid.txt",
        "tests/data/hsk_words.txt",
    ];

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn read_dataset(path: &str) -> BufReader<File> {
        let mut manifest = manifest_dir();
        manifest.push(path);
        let file = File::open(manifest).expect(format!("open {} failed", path).as_str());
        BufReader::new(file)
    }

    #[test]
    fn test_insert() {
        for file in FILES.iter() {
            let mut art = Art::<String, String>::new();
            for (index, line) in read_dataset(*file).lines().enumerate() {
                let line = line.expect("read words line failed");
                assert_eq!(art.size(), index);
                assert_eq!(art.insert(line.clone(), line.clone()), None);
            }
        }
    }

    #[test]
    fn test_insert_get() {
        for file in FILES.iter() {
            let mut art = Art::<String, String>::new();
            let mut lines = 0;
            for line in read_dataset(*file).lines() {
                let line = line.expect("read words line failed");
                assert_eq!(art.insert(line.clone(), line.clone()), None);
                assert_eq!(art.get(&line), Some(&line));
                lines += 1;
            }
            assert_eq!(art.size(), lines);
        }
    }

    #[test]
    fn test_insert_get_remove_get() {
        for file in FILES.iter() {
            let mut art = Art::<String, String>::new();
            let mut lines = 0;
            let read_buf = read_dataset(*file);
            for line in read_buf.lines() {
                let line = line.expect("read words line failed");
                assert_eq!(art.insert(line.clone(), line.clone()), None);
                lines += 1;
            }
            assert_eq!(art.size(), lines);

            let read_buf = read_dataset(*file);
            for (index, line) in read_buf.lines().enumerate() {
                let line = line.expect("read words line failed");
                assert_eq!(art.size(), lines - index);
                assert_eq!(art.get(&line), Some(&line));
                assert_eq!(art.remove(&line), Some(line.clone()));
                assert_eq!(art.get(&line), None);
            }
            assert_eq!(art.size(), 0);
        }
    }

    #[test]
    fn test_get_insert_remove_get() {
        for file in FILES.iter() {
            let mut art = Art::<String, String>::new();
            let mut lines = 0;
            let read_buf = read_dataset(*file);
            for line in read_buf.lines() {
                let line = line.expect("read words line failed");
                assert_eq!(art.get(&line), None);
                assert_eq!(art.insert(line.clone(), line.clone()), None);
                assert_eq!(art.get(&line), Some(&line));
                assert_eq!(art.remove(&line), Some(line.clone()));
                assert_eq!(art.get(&line), None);
                lines += 1;
            }
            assert_eq!(art.size(), 0);
        }
    }

    #[test]
    fn test_insert_random_remove() {
        let byte_len_range = (9, 12); // remove this of range byte len
        for file in FILES.iter() {
            let mut art = Art::<String, String>::new();
            let mut lines = 0;
            let read_buf = read_dataset(*file);
            for line in read_buf.lines() {
                let line = line.expect("read words line failed");
                assert_eq!(art.insert(line.clone(), line.clone()), None);
                lines += 1;
            }
            assert_eq!(art.size(), lines);

            // random keys in the byte len range randomly
            let read_buf = read_dataset(*file);
            let mut hit_lines = 0;
            for (index, line) in read_buf.lines().enumerate() {
                let line = line.expect("read words line failed");
                let byte_len = line.as_str().as_bytes().len();
                if byte_len >= byte_len_range.0 && byte_len <= byte_len_range.1 {
                    assert_eq!(art.remove(&line), Some(line.clone()));
                    hit_lines += 1;
                }
            }
            assert_eq!(art.size(), lines - hit_lines);

            // get insert get
            let read_buf = read_dataset(*file);
            for (index, line) in read_buf.lines().enumerate() {
                let line = line.expect("read words line failed");
                let byte_len = line.as_str().as_bytes().len();
                if byte_len >= byte_len_range.0 && byte_len <= byte_len_range.1 {
                    assert_eq!(art.get(&line), None);
                    assert_eq!(art.insert(line.clone(), line.clone()), None);
                    assert_eq!(art.get(&line), Some(&line));
                }
            }
            assert_eq!(art.size(), lines);
        }
    }

    #[test]
    fn test_short_prefixed() {
        let mut art = Art::<String, i32>::new();
        let cases: Vec<(String, i32)> = vec![
            ("bcd".to_string(), 1),
            ("bcdd".to_string(), 2),
            ("bcdde".to_string(), 3),
            ("bcddee".to_string(), 4),
            ("bcddeef".to_string(), 5),
            ("bcddeeff".to_string(), 6),
            ("bcddeeffg".to_string(), 7),
            ("bcddeeffgg".to_string(), 8),
        ];

        for (key, val) in cases.iter() {
            assert_eq!(art.insert(key.clone(), *val), None);
        }

        for (key, val) in cases.iter() {
            assert_eq!(art.get(key), Some(val));
        }
    }

    #[test]
    fn test_long_prefixed() {
        let mut art = Art::<String, i32>::new();
        let cases: Vec<(String, i32)> = vec![
            ("this:key:has:a:long:prefix:3".to_string(), 3),
            ("this:key:has:a:long:common:prefix:2".to_string(), 2),
            ("this:key:has:a:long:common:prefix:1".to_string(), 1),
        ];

        for (key, val) in cases.iter() {
            assert_eq!(art.insert(key.clone(), *val), None);
        }

        for (key, val) in cases.iter() {
            assert_eq!(art.get(key), Some(val));
        }
    }
}
