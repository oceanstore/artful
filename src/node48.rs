use crate::node::ArtNode;
use crate::node16::Node16;
use crate::node256::Node256;
use crate::ArtKey;
use crate::Header;

const EMPTY_INDEX: u8 = 48;

pub(crate) struct Node48<K: ArtKey, V: Default, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    pub(crate) child_index: [u8; 256], // invert index of children
    pub(crate) children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 48],
    pub(crate) prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V: Default, const MAX_PARTIAL_LEN: usize> Default
    for Node48<K, V, MAX_PARTIAL_LEN>
{
    fn default() -> Node48<K, V, MAX_PARTIAL_LEN> {
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

impl<K: ArtKey, V: Default, const MAX_PARTIAL_LEN: usize> Node48<K, V, MAX_PARTIAL_LEN> {
    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        self.header.non_null_children == 48
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

        Some(&self.children[self.find_child_index(key.0)?])
    }

    pub(crate) fn get_mut_child(
        &mut self,
        key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&mut self.prefixed_child);
        }

        Some(&mut self.children[self.find_child_index(key.0)?])
    }

    #[inline]
    fn find_child_index(&self, key: u8) -> Option<usize> {
        let index = self.child_index[key as usize];
        if index == EMPTY_INDEX {
            None
        } else {
            Some(index as usize)
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

        std::mem::swap(&mut self.children[pos], &mut new_child);
        assert_eq!(self.children[pos].is_none(), false);
        assert_eq!(new_child.is_none(), true);
        self.child_index[key.0 as usize] = pos as u8;
        self.header.non_null_children += 1;
    }

    /// Grow node48 to node256.
    #[inline(always)]
    pub(crate) fn grow(&mut self) -> Box<Node256<K, V, MAX_PARTIAL_LEN>> {
        let mut node256: Box<Node256<K, V, MAX_PARTIAL_LEN>> = Box::default();
        for (byte, index) in self.child_index.iter_mut().enumerate() {
            if *index != EMPTY_INDEX {
                assert_eq!(self.children[*index as usize].is_none(), false);
                std::mem::swap(
                    &mut node256.children[byte],
                    &mut self.children[*index as usize],
                );
                assert_ne!(node256.children[byte].0, self.children[*index as usize].0);
                *index = EMPTY_INDEX;
            }
        }
        std::mem::swap(&mut self.prefixed_child, &mut node256.prefixed_child);
        node256.header.partial.clone_from(&self.header.partial);
        node256.header.non_null_children = self.header.non_null_children;
        // node256.header = self.header;
        node256
    }

    pub fn is_few(&self) -> bool {
        self.header.non_null_children == 16
    }

    pub(crate) fn remove_child(
        &mut self,
        valid_key: (u8, bool),
    ) -> Option<ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !valid_key.1 {
            return Some(std::mem::take(&mut self.prefixed_child));
        }

        let index = self.find_child_index(valid_key.0)?;
        assert_eq!(self.children[index].is_none(), false);
        let removed = std::mem::take(&mut self.children[index]);
        self.child_index[valid_key.0 as usize] = EMPTY_INDEX;
        self.header.non_null_children -= 1;
        return Some(removed);
    }

    pub(crate) fn shrink_to_fit(&mut self) -> Box<Node16<K, V, MAX_PARTIAL_LEN>> {
        let mut node16: Box<Node16<K, V, MAX_PARTIAL_LEN>> = Box::default();
        let mut node16_index = 0;
        for idx in 0..256 {
            if self.child_index[idx] != EMPTY_INDEX {
                std::mem::swap(
                    &mut self.children[self.child_index[idx] as usize],
                    &mut node16.children[node16_index],
                );
                node16.key[node16_index] = idx as u8;
                node16_index += 1;
            }
        }

        std::mem::swap(&mut self.prefixed_child, &mut node16.prefixed_child);
        // node16.header = self.header;
        node16.header.partial.clone_from(&self.header.partial);
        node16.header.non_null_children = node16_index as u16;
        node16
    }
}
