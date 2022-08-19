use crate::node::ArtNode;
use crate::node256::Node256;
use crate::ArtKey;
use crate::Header;
use std::process::id;
use std::ptr::copy_nonoverlapping;

const EMPTY_INDEX: u8 = 48;
const FULL_NODE_SIZE: u16 = 48;

pub(crate) struct Node48<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    pub(crate) child_index: [u8; 256], // invert index of children
    pub(crate) children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 48],
    pub(crate) prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for Node48<K, V, MAX_PARTIAL_LEN> {
    fn default() -> Node48<K, V, MAX_PARTIAL_LEN> {
        // Why dont' i use macro `vec![]` initialize the children?
        // just like, you know `vec![ArtNode::none(); 16].try_into()...`.
        // because it need clone and our intilization with occur on
        // the insert critial performacne path. so, we manual do it.
        Node48 {
            header: Default::default(),
            prefixed_child: ArtNode::default(),
            child_index: [EMPTY_INDEX; 256],
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
            ],
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Node48<K, V, MAX_PARTIAL_LEN> {
    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        self.header.non_null_children >= FULL_NODE_SIZE
    }

    #[inline(always)]
    pub(crate) fn minimum_child(&self) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !self.prefixed_child.is_none() {
            return Some(&self.prefixed_child);
        }

        // TODO: simd split
        for i in 0..256 {
            if self.child_index[i] != EMPTY_INDEX {
                return Some(&self.children[i]);
            }
        }
        return None;
    }

    #[inline]
    pub(crate) fn get_child(&self, key: (u8, bool)) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&self.prefixed_child);
        }

        let index = self.child_index[key.0 as usize];
        if index == EMPTY_INDEX {
            None
        } else {
            Some(&self.children[index as usize])
        }
    }

    pub(crate) fn get_mut_child(
        &mut self,
        key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&mut self.prefixed_child);
        }

        let index = self.child_index[key.0 as usize];
        if index == EMPTY_INDEX {
            None
        } else {
            Some(&mut self.children[index as usize])
        }
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

        assert!(self.header.non_null_children < 48);
        let mut pos = self.header.non_null_children as usize;
        // When the next position is none, we should preferentially
        // attempt to insert at the next position.
        if !self.children[pos].is_none() {
            pos = 0;
            while !self.children[pos].is_none() {
                pos += 1;
            }
        }
        self.children[pos] = new_child;
        self.child_index[key.0 as usize] = pos as u8;
        self.header.non_null_children += 1;
    }

    /// Grow node48 to node256.
    #[inline(always)]
    pub(crate) fn grow(&mut self) -> Box<Node256<K, V, MAX_PARTIAL_LEN>> {
        let mut node256: Box<Node256<K, V, MAX_PARTIAL_LEN>> = Box::default();
        std::mem::swap(&mut self.prefixed_child, &mut node256.prefixed_child);
        for i in 0..256 {
            if self.child_index[i] != EMPTY_INDEX {
                std::mem::swap(
                    &mut node256.children[i],
                    &mut self.children[self.child_index[i] as usize],
                )
            }
        }
        // for (idx, key) in self.child_index.iter().enumerate() {
        //     if *key != EMPTY_INDEX {
        //         std::mem::swap(&mut node256.children[idx], &mut self.children[idx])
        //     }
        // }

        // copy the old node header to the new grown node.
        node256.header = self.header;
        node256
    }
}

#[test]
fn basic() {
    println!("{}", std::mem::size_of::<Node48<String, i32, 9>>());
    println!("{}", std::mem::size_of::<[u8; 256]>());
}
