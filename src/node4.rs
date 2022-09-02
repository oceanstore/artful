use crate::node::ArtNode;
use crate::node16::Node16;
use crate::ArtKey;
use crate::Header;

pub(crate) struct Node4<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    pub(crate) key: [u8; 4],
    pub(crate) children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 4],
    pub(crate) prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for Node4<K, V, MAX_PARTIAL_LEN> {
    fn default() -> Node4<K, V, MAX_PARTIAL_LEN> {
        Node4 {
            header: Default::default(),
            key: [0; 4],
            children: [
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
                ArtNode::none(),
            ],
            prefixed_child: ArtNode::default(),
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Node4<K, V, MAX_PARTIAL_LEN> {
    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        self.header.non_null_children == 4
    }

    #[inline(always)]
    pub(crate) fn minimum_child(&self) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !self.prefixed_child.is_none() {
            Some(&self.prefixed_child)
        } else {
            Some(&self.children[0])
        }
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

    pub(crate) fn insert_child(
        &mut self,
        valid_key: (u8, bool),
        mut new_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
    ) {
        if !valid_key.1 {
            assert!(self.prefixed_child.is_none());
            std::mem::swap(&mut self.prefixed_child, &mut new_child);
            return;
        }
        assert!(self.header.non_null_children < 4);

        let key_byte = valid_key.0;
        // find first index greater than or equal to key_byte
        let mut index = 0;
        while (index < self.header.non_null_children) && self.key[index as usize] < key_byte {
            index += 1;
        }

        if !self.children[index as usize].is_none() {
            let mut i = self.header.non_null_children;
            while i > index {
                let mut moved = std::mem::take(&mut self.children[i as usize - 1]);
                std::mem::swap(&mut self.children[i as usize], &mut moved);
                self.key[i as usize] = self.key[i as usize - 1];
                i -= 1;
            }
        }

        self.key[index as usize] = key_byte;
        std::mem::swap(&mut self.children[index as usize], &mut new_child);

        if !new_child.is_none() && !new_child.is_leaf() {
            assert_eq!(0, new_child.header().non_null_children);
        }

        self.header.non_null_children += 1;
    }

    #[inline(always)]
    pub(crate) fn grow(&mut self) -> Box<Node16<K, V, MAX_PARTIAL_LEN>> {
        assert_eq!(self.header.non_null_children, 4);
        let mut node16: Box<Node16<K, V, MAX_PARTIAL_LEN>> = Box::default();
        // copy invalid child
        std::mem::swap(&mut self.prefixed_child, &mut node16.prefixed_child);
        // copy child
        std::mem::swap(&mut self.children[0], &mut node16.children[0]);
        std::mem::swap(&mut self.children[1], &mut node16.children[1]);
        std::mem::swap(&mut self.children[2], &mut node16.children[2]);
        std::mem::swap(&mut self.children[3], &mut node16.children[3]);
        node16.key[0] = self.key[0];
        node16.key[1] = self.key[1];
        node16.key[2] = self.key[2];
        node16.key[3] = self.key[3];
        // copy the old node header to the new grown node.
        node16.header.partial.clone_from(&self.header.partial);
        node16.header.non_null_children = self.header.non_null_children;
        // node16.header = self.header;
        node16
    }

    pub fn is_few(&self) -> bool {
        let mut child_num = self.header.non_null_children;
        if !self.prefixed_child.is_none() {
            child_num += 1;
        }

        child_num < 2
    }

    pub(crate) fn remove_child(
        &mut self,
        valid_key: (u8, bool),
    ) -> Option<ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !valid_key.1 {
            assert_eq!(self.prefixed_child.is_none(), false);
            return Some(std::mem::take(&mut self.prefixed_child));
        }

        // TODO: This time the lookup can be optimized to be passed externally.
        let mut idx = self.find_child_index(valid_key.0)?;
        self.key[idx] = 0;
        self.header.non_null_children -= 1;
        let child = std::mem::take(&mut self.children[idx]);

        // to keep order
        while idx < self.header.non_null_children as usize {
            self.key[idx] = self.key[idx + 1];
            let mut moved = std::mem::take(&mut self.children[idx + 1]);
            std::mem::swap(&mut self.children[idx], &mut moved);
            idx += 1
        }

        // again remaining
        while idx < 4 {
            let _ = std::mem::take(&mut self.children[idx]);
            idx += 1
        }

        Some(child)
    }

    #[inline]
    fn find_child_index(&self, key: u8) -> Option<usize> {
        for i in 0..self.header.non_null_children {
            if self.key[i as usize] == key {
                return Some(i as usize);
            }
        }

        None
    }

    // shrink node4
    /// Safety: the node number of children must be equal 1 (include prefix child).
    pub(crate) fn shrink_to_fit(&mut self) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let mut single_child = std::mem::take(&mut self.children[0]);
        if single_child.is_none() {
            assert_eq!(self.prefixed_child.is_none(), false);
            single_child = std::mem::take(&mut self.prefixed_child);
        }

        if single_child.is_leaf() {
            return single_child;
        }

        // shrink single child
        // 1. Merge the key pointing to a single child
        let mut prefix_len = self.header.partial.len as usize;
        if prefix_len < MAX_PARTIAL_LEN {
            self.header.partial.data[prefix_len] = self.key[0];
            prefix_len += 1;
        }

        // let header = single_child.header_mut();
        let mut header = Header::<MAX_PARTIAL_LEN>::default();
        header.partial.clone_from(&single_child.header().partial);
        header.non_null_children = single_child.header().non_null_children;

        // 2. Merge partial parts of the Child
        if prefix_len < MAX_PARTIAL_LEN {
            let minimum = std::cmp::min(header.partial.len as usize, MAX_PARTIAL_LEN - prefix_len);
            self.header.partial.data[prefix_len..prefix_len + minimum]
                .copy_from_slice(&header.partial.data[..minimum]);
            prefix_len += minimum;
        }

        let minimum = std::cmp::min(prefix_len, MAX_PARTIAL_LEN);
        header.partial.data[..minimum].copy_from_slice(&self.header.partial.data[..minimum]);
        header.partial.len += (self.header.partial.len + 1) as u32;
        std::mem::swap(single_child.header_mut(), &mut header);
        single_child
    }
}
