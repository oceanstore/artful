use crate::ArtKey;

#[repr(align(8))]
pub(crate) struct Leaf<K: ArtKey, V> {
    pub(crate) key: K,
    pub(crate) val: V,
}

impl<K: ArtKey, V> Leaf<K, V> {
    pub(crate) fn new(key: K, val: V) -> Leaf<K, V> {
        Leaf { key, val }
    }

    pub(crate) fn matches(&self, key: &[u8]) -> bool {
        let leaf_key = self.key.get_bytes();
        if leaf_key.len() != key.len() {
            return false;
        }

        return leaf_key == key;
    }
}
