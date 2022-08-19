use crate::leaf;
use crate::leaf::Leaf;
use crate::node::ArtNode;
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
{
    size: usize,
    root: ArtNode<K, V, MAX_PARTIAL_LEN>,
}
impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Art<K, V, MAX_PARTIAL_LEN> {
    pub fn new() -> Art<K, V, MAX_PARTIAL_LEN> {
        Art {
            size: 0,
            root: ArtNode::none(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        ArtNode::get(&self.root, key.get_bytes(), 0)
    }

    pub fn insert(&mut self, key: K, val: V) {
        if ArtNode::insert(&mut self.root, key, val, 0) {
            self.size += 1
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let leaf = ArtNode::remove(&mut self.root, key.get_bytes(), 0)?;
        self.size -= 1;
        Some(leaf.val)
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
    art.insert("aaaaaaaabbbbbbcce".to_string(), 3);

    art.insert("ba".to_string(), 1);
    art.insert("bar".to_string(), 1);
    art.insert("baz".to_string(), 2);
    art.insert("bcd".to_string(), 3);
    art.insert("bcs".to_string(), 4);
    art.insert("bcf".to_string(), 5);
    art.insert("bdb".to_string(), 6);
    art.insert("bcbc".to_string(), 7);
    art.insert("bcde".to_string(), 8);
    art.insert("bcdef".to_string(), 8);
    art.insert("bcdeff".to_string(), 8);
    art.insert("abcd".to_string(), 222);
    art.insert("foo".to_string(), 111);

    println!("{:?}", art.get(&"aaaaaaaabbbbbbbbcccccd".to_string()));
    println!("{:?}", art.get(&"aaaaaaaabbbbbbbbccce".to_string()));
    println!("{:?}", art.get(&"aaaaaaaabbbbbbbbcce".to_string()));
    println!("{:?}", art.get(&"aaaaaaaabbbbbbcce".to_string()));
    println!("{:?}", art.get(&"ba".to_string()));
    println!("{:?}", art.get(&"bar".to_string()));
    println!("{:?}", art.get(&"baz".to_string()));
    println!("{:?}", art.get(&"bcd".to_string()));
    println!("{:?}", art.get(&"bcs".to_string()));
    println!("{:?}", art.get(&"bcf".to_string()));
    println!("{:?}", art.get(&"bdb".to_string()));
    println!("{:?}", art.get(&"bcbc".to_string()));
    println!("{:?}", art.get(&"bcde".to_string()));
    println!("{:?}", art.get(&"bcdef".to_string()));
    println!("{:?}", art.get(&"bcdeff".to_string()));
    println!("{:?}", art.get(&"abcd".to_string()));
    println!("{:?}", art.get(&"foo".to_string()));
}
#[test]
fn basic_i32() {
    let mut art = Art::<i32, i32, 8>::new();
    for i in (0..10000000).rev() {
        art.insert(i, i);
    }

    for i in 0..10000000 {
        assert_eq!(art.get(&i), Some(&i))
    }

    for i in 0..10000000 {
        art.insert(i, i);
    }

    assert_eq!(10000000, art.size);

    let mut art = Art::<String, i32, 10>::new();
}
