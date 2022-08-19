use crate::node::ArtNode;
use crate::ArtKey;
use crate::Header;

const FULL_NODE_SIZE: u16 = 256;

pub(crate) struct Node256<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    pub(crate) children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 256],
    pub(crate) prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for Node256<K, V, MAX_PARTIAL_LEN> {
    fn default() -> Node256<K, V, MAX_PARTIAL_LEN> {
        // Why dont' i use macro `vec![]` initialize the children?
        // just like, you know `vec![ArtNode::none(); 16].try_into()...`.
        // because it need clone and our intilization with occur on
        // the insert critial performacne path. so, we manual do it.
        Node256 {
            header: Default::default(),
            prefixed_child: ArtNode::default(),
            children: [
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 32
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                //64
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                //96
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 128
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 160
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 192
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 224
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                // 256
            ],
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Node256<K, V, MAX_PARTIAL_LEN> {
    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        self.header.non_null_children == FULL_NODE_SIZE
    }

    #[inline(always)]
    pub(crate) fn minimum_child(&self) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !self.prefixed_child.is_none() {
            return Some(&self.prefixed_child);
        }
        // TODO: simd split
        for node in self.children.iter() {
            if !node.is_none() {
                return Some(node);
            }
        }
        return None;
    }

    #[inline]
    pub(crate) fn get_child(&self, key: (u8, bool)) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&self.prefixed_child);
        }
        return Some(&self.children[key.0 as usize]);
    }

    pub(crate) fn get_mut_child(
        &mut self,
        key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&mut self.prefixed_child);
        }
        return Some(&mut self.children[key.0 as usize]);
    }

    /// Safety: grow.
    pub(crate) fn insert_child(
        &mut self,
        key: (u8, bool),
        mut new_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
    ) {
        if !key.1 {
            assert!(self.prefixed_child.is_none());
            std::mem::swap(&mut self.prefixed_child, &mut new_child);
            return;
        }
        self.header.non_null_children += 1;
        self.children[key.0 as usize] = new_child;
    }
}
