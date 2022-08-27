use crate::leaf;
use crate::leaf::Leaf;
use crate::node::ArtNode;
use crate::node::ArtNodeMut;
use crate::node::ArtNodeRef;
use crate::node4::Node4;
use crate::ArtKey;
use crate::Partial;
use std::cmp::min;
use std::mem::take;
use std::ops::Deref;

pub struct Art<K, V, const MAX_PARTIAL_LEN: usize = 8>
where
    K: ArtKey,
    V: Default,
{
    size: usize,
    root: ArtNode<K, V, MAX_PARTIAL_LEN>,
}
impl<K: ArtKey, V: Default, const MAX_PARTIAL_LEN: usize> Art<K, V, MAX_PARTIAL_LEN> {
    pub fn new() -> Art<K, V, MAX_PARTIAL_LEN> {
        Art {
            size: 0,
            root: ArtNode::none(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        ArtNode::get(&self.root, key.get_bytes(), 0)
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if let Some(old_val) = ArtNode::insert(&mut self.root, key, val, 0) {
            return Some(old_val);
        }

        self.size += 1;
        None
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let val = ArtNode::remove(&mut self.root, key.get_bytes(), 0)?;
        self.size -= 1;
        Some(val)
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }
}

#[test]
fn basic_art() {
    let mut art = Art::<String, i32, 10>::new();

    art.insert("aaaaaaaabbbbbbbbcccccd".to_string(), 1);
    art.insert("aaaaaaaabbbbbbbbccce".to_string(), 2);
    art.insert("aaaaaaaabbbbbbbbcce".to_string(), 3);
    art.insert("aaaaaaaabbbbbbcce".to_string(), 4);
    art.insert("ba".to_string(), 5);
    art.insert("bar".to_string(), 6);
    art.insert("baz".to_string(), 7);
    art.insert("bcd".to_string(), 8);
    art.insert("bcs".to_string(), 9);
    art.insert("bcf".to_string(), 10);
    art.insert("bdb".to_string(), 11);
    art.insert("bcbc".to_string(), 12);
    art.insert("bcde".to_string(), 13);
    art.insert("bcdef".to_string(), 14);
    art.insert("bcdeff".to_string(), 15);
    art.insert("abcd".to_string(), 17);
    art.insert("foo".to_string(), 18);
    art.insert("X".to_string(), 19);
    art.insert("x".to_string(), 20);
    art.insert("xanthaline".to_string(), 21);
    art.insert("xanthamic".to_string(), 22);

    println!("get {:?}", art.get(&"aaaaaaaabbbbbbbbcccccd".to_string()));
    println!("get {:?}", art.get(&"aaaaaaaabbbbbbbbccce".to_string()));
    println!("get {:?}", art.get(&"aaaaaaaabbbbbbbbcce".to_string()));
    println!("get {:?}", art.get(&"aaaaaaaabbbbbbcce".to_string()));
    println!("get {:?}", art.get(&"ba".to_string()));
    println!("get {:?}", art.get(&"bar".to_string()));
    println!("get {:?}", art.get(&"baz".to_string()));
    println!("get {:?}", art.get(&"bcd".to_string()));
    println!("get {:?}", art.get(&"bcs".to_string()));
    println!("get {:?}", art.get(&"bcf".to_string()));
    println!("get {:?}", art.get(&"bdb".to_string()));
    println!("get {:?}", art.get(&"bcbc".to_string()));
    println!("get {:?}", art.get(&"bcde".to_string()));
    println!("get {:?}", art.get(&"bcdef".to_string()));
    println!("get {:?}", art.get(&"bcdeff".to_string()));
    println!("get {:?}", art.get(&"abcd".to_string()));
    println!("get {:?}", art.get(&"foo".to_string()));
    println!("get {:?}", art.get(&"X".to_string()));
    println!("get {:?}", art.get(&"x".to_string()));

    println!(
        "remove {:?}",
        art.remove(&"aaaaaaaabbbbbbbbcccccd".to_string())
    );
    println!(
        "remove {:?}",
        art.remove(&"aaaaaaaabbbbbbbbccce".to_string())
    );
    println!(
        "remove {:?}",
        art.remove(&"aaaaaaaabbbbbbbbcce".to_string())
    );
    println!("remove {:?}", art.remove(&"aaaaaaaabbbbbbcce".to_string()));
    println!("remove {:?}", art.remove(&"ba".to_string()));
    println!("remove {:?}", art.remove(&"bar".to_string()));
    println!("remove {:?}", art.remove(&"baz".to_string()));
    println!("remove {:?}", art.remove(&"bcd".to_string()));
    println!("remove {:?}", art.remove(&"bcs".to_string()));
    println!("remove {:?}", art.remove(&"bcf".to_string()));
    println!("remove {:?}", art.remove(&"bdb".to_string()));
    println!("remove {:?}", art.remove(&"bcbc".to_string()));
    println!("remove {:?}", art.remove(&"bcde".to_string()));
    println!("remove {:?}", art.remove(&"bcdef".to_string()));
    println!("remove {:?}", art.remove(&"bcdeff".to_string()));
    println!("remove {:?}", art.remove(&"abcd".to_string()));
    println!("remove {:?}", art.remove(&"foo".to_string()));
    println!("remove {:?}", art.remove(&"X".to_string()));
    println!("remove {:?}", art.remove(&"x".to_string()));
}

#[test]
fn basic_one_hundred() {
    let mut art = Art::<i32, i32, 8>::new();
    for i in (0..1000000).rev() {
        assert_eq!(art.insert(i, i), None);
    }

    for i in 0..1000000 {
        assert_eq!(art.get(&i), Some(&i))
    }

    for i in 0..1000000 {
        art.insert(i, i);
    }

    assert_eq!(1000000, art.size);
}
