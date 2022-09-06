use crate::node::ArtNode;
use crate::ArtKey;

/// Art is an **adaptive radix tree**, which are also known as radix trees and
/// prefix trees.
///
/// Art requires 3 generic parameters, where `K` needs to implement the [ArtKey] trait and `V` is
/// self-explanatory. `MAX_PARTIAL_LEN` specifies the size of the array used by each inner node to
/// store the common prefix.
///
/// **Note:** `MAX_PARTIAL_LEN` is designed as a constant generic parameter because setting its size
/// requires users trade-off.
///
/// Radix tress consist of two types of nodes: inner and leaf nodes. Inner node map
/// partial keys to other nodes and leaf node store key-value pair.
///
/// For large keys, comparisons actually take $O(k)$ time. the complexity of
/// search trees is $O(k\log_n)$ and the art complexity of $O(k)$.
///
/// Art key idea that achieves both space and time efficiency is to adaptively use different node size with
/// the same, relatively large space, but with different fan-out.
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

    /// Returns a reference to the value corresponding to the key.
    /// The key may be any borrowed form of the map’s key type and must be implementation `ArtKey` trait.
    ///
    /// # Examples
    /// ```rust
    /// use artful::Art;
    ///
    /// let mut art = Art::<i32, &str, 8>::new();
    /// assert_eq!(art.insert(37, "a"), None);
    /// assert_eq!(art.insert(37, "b"), Some("a"));
    /// ```
    pub fn get(&self, key: &K) -> Option<&V> {
        ArtNode::get(&self.root, key.get_bytes(), 0)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The key may be any borrowed form of the map’s key type and must be implementation `ArtKey` trait.
    ///
    /// # Examples
    /// ```rust
    /// use artful::Art;
    /// let mut art = Art::<i32, &str, 8>::new();
    /// art.insert(1, "a");
    /// assert_eq!(art.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(art.get_key_value(&2), None);
    /// ```
    pub fn get_key_value<'a>(&'a self, key: &'a K) -> Option<(&'a K, &'a V)> {
        if let Some(val) = ArtNode::get(&self.root, key.get_bytes(), 0) {
            return Some((key, &*val));
        }

        None
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    ///
    /// The key may be any borrowed form of the map’s key type and must be implementation `ArtKey` trait.
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use artful::Art;
    ///
    /// let mut art = Art::<i32, &str, 8>::new();
    /// art.insert(1, "a");
    /// if let Some(x) = art.get_mut(&1) {
    ///     *x = "b";
    /// }
    /// assert_eq!(art.get(&1), Some(&"b"));
    /// ```
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        ArtNode::get_mut(&mut self.root, key.get_bytes(), 0)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned. The key is not updated, though;
    ///
    /// # Examples
    /// ```rust
    /// use artful::Art;
    ///
    /// let mut art = Art::<i32, &str, 8>::new();
    /// let _ = art.insert(1, "a");
    /// assert_eq!(art.get(&1), Some(&"a"));
    /// ```
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
    /// # Examples
    /// ```rust
    /// use artful::Art;
    ///
    /// let mut art = Art::<i32, &str, 8>::new();
    /// let _ = art.insert(1, "a");
    /// let _ = art.insert(1, "b");
    /// assert_eq!(art.size(), 1);
    /// ```
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod test {
    use super::Art;
    use super::ArtKey;
    fn primitive<
        T: ArtKey
            + Copy
            + std::cmp::PartialEq
            + std::fmt::Debug
            + std::ops::AddAssign<T>
            + std::ops::Div<Output = T>
            + std::ops::Add<Output = T>,
    >(
        range: impl Iterator<Item = T> + std::clone::Clone,
        mut_value: T,
    ) {
        let mut art = Art::<T, T, 8>::new();
        // get after insert and get
        for i in range.clone().into_iter() {
            assert_eq!(art.get(&i), None);
            assert_eq!(art.insert(i, i), None);
            assert_eq!(art.get(&i), Some(&i));
        }

        // get_mut after get
        for i in range.clone().into_iter() {
            let old_val = art.get_mut(&i).unwrap();
            *old_val += mut_value;
            assert_eq!(art.get(&i), Some(&(i + mut_value)))
        }

        // remove after get
        for i in range.into_iter() {
            assert_eq!(art.remove(&i), Some(i + mut_value));
            assert_eq!(art.get(&i), None)
        }
    }

    #[test]
    fn basic_primitive_i8() {
        primitive::<i8>(i8::MIN..i8::MAX, 1);
    }

    #[test]
    fn basic_primitive_i16() {
        primitive::<i16>(i16::MIN..i16::MAX, 1);
    }

    #[test]
    fn basic_primitive_i32() {
        primitive::<i32>(-1000000..10000000, 1);
    }

    #[test]
    fn basic_primitive_i64() {
        primitive::<i64>(-1000000..10000000, 1);
    }

    #[test]
    fn basic_primitive_u8() {
        primitive::<u8>(u8::MIN..u8::MAX, 1);
    }

    #[test]
    fn basic_primitive_u16() {
        primitive::<u16>(u16::MIN..u16::MAX, 1);
    }

    #[test]
    fn basic_primitive_u32() {
        primitive::<u32>(0..10000000, 1);
    }

    #[test]
    fn basic_primitive_u64() {
        primitive::<u64>(0..10000000, 1);
    }
}
