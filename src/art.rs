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

/// Art is an **adaptive radix tree**, which are also known as radix trees and
/// prefix trees.
///
/// Radix tress consist of two types of nodes: inner and leaf nodes. Inner node map
/// partial keys to other nodes and leaf node store key-value pair.
///
/// For large keys, comparisons actually take $O(k)$ time. the complexity of
/// search trees is $O(k\log_n)$ and the art complexity of $O(k)$.
///
/// Art key idea that achieves both space and time efficiency is to adaptively use different node size with
/// the same, relatively large space, but with different fanout.
///
/// Specifically, Art uses four node types of dynamically adaptive representation of inner nodes to
/// achieve space and time efficiency.
/// 1. Node4, which store up to 4 child pointers and uses an array of length 4 for keys. The keys
///    and pointers are stored at corresponding positions and the keys are sorted. In the
///    future,Art will no maintenance order and use simd.
/// 2. Node16, Which storing 5 and 16 child pointers.  Like the Node4, the keys and pointers
///    are stored in separate arrays at corresponding positions, but both arrays have space
///    for 16 entries. **A key use SIMD instructions to parallel comparisons on modern hardware.**
/// 3. Node48, which node storing 17 and 48 child pointers. Unlike Node4 and Node16, Node48 uses
///    an array of length 256 for keys to index child. because seaching becomes expensive.
/// 4. Node256, which largest node type is simply an array of 256 pointers.
///
///
/// See [The Adaptive Radix Tree: ARTful indexing for Main-Memory Databases](https://db.in.tum.de/~leis/papers/ART.pdf)
/// for more information.
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

    /// Returns a reference to the value corresponding to the key.
    /// The key may be any borrowed form of the map’s key type and must be implementation `ArtKey` trait.
    pub fn get(&self, key: &K) -> Option<&V> {
        ArtNode::get(&self.root, key.get_bytes(), 0)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned. The key is not updated, though;
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if let Some(old_val) = ArtNode::insert(&mut self.root, key, val, 0) {
            return Some(old_val);
        }

        self.size += 1;
        None
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    ///
    /// The key may be any borrowed form of the map’s key type and must be implementation `ArtKey` trait.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let val = ArtNode::remove(&mut self.root, key.get_bytes(), 0)?;
        self.size -= 1;
        Some(val)
    }

    /// Returns the size of key-value pairs in Art.
    ///
    /// For insertion, size is incremented only when the key is different. For deletion size is decremented
    /// only when the key exists.
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
