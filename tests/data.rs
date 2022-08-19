#[cfg(test)]
mod dataset_test {
    use artful::Art;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn test_words() {
        let mut art = Art::<String, String>::new();

        let mut manifest = manifest_dir();
        manifest.push("tests/data/words.txt");
        let file = File::open(manifest).expect("open words.txt failed");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("read words line failed");
            art.insert(line.clone(), line.clone());
            assert_eq!(art.get(&line), Some(&line));
        }
        assert_eq!(art.size(), 235886);
    }

    #[test]
    fn test_uuid() {
        let mut art = Art::<String, String>::new();

        let mut manifest = manifest_dir();
        manifest.push("tests/data/uuid.txt");
        let file = File::open(manifest).expect("open words.txt failed");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("read words line failed");
            art.insert(line.clone(), line.clone());
            assert_eq!(art.get(&line), Some(&line));
        }
        assert_eq!(art.size(), 100000);
    }

    #[test]
    fn test_hsk_words() {
        let mut art = Art::<String, String>::new();

        let mut manifest = manifest_dir();
        manifest.push("tests/data/hsk_words.txt");
        let file = File::open(manifest).expect("open hsk_words.txt failed");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("read hsk_words line failed");
            art.insert(line.clone(), line.clone());
            assert_eq!(art.get(&line), Some(&line));
        }
        assert_eq!(art.size(), 4995);
    }
}
