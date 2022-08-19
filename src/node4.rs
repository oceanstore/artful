use crate::node::{ArtNode, ArtNodeMut, ArtNodeRef};
use crate::node16::Node16;
use crate::ArtKey;
use crate::Header;
use std::mem::take;

const FULL_NODE_SIZE: u16 = 4;

// #[derive( Clone)]
/// 16 + 4 + 32 and padding 4
pub(crate) struct Node4<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    key: [u8; 4],
    children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 4],
    prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for Node4<K, V, MAX_PARTIAL_LEN> {
    fn default() -> Node4<K, V, MAX_PARTIAL_LEN> {
        Node4 {
            header: Default::default(),
            key: [255; 4],
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
        self.header.non_null_children >= FULL_NODE_SIZE
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

        for i in 0..self.header.non_null_children {
            if self.key[i as usize] == key.0 {
                return Some(&self.children[i as usize]);
            }
        }

        return None;
    }

    pub(crate) fn get_mut_child(
        &mut self,
        key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&mut self.prefixed_child);
        }

        for i in 0..self.header.non_null_children {
            if self.key[i as usize] == key.0 {
                return Some(&mut self.children[i as usize]);
            }
        }

        return None;
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
                self.children[i as usize] = std::mem::take(&mut self.children[i as usize - 1]);
                self.key[i as usize] = self.key[i as usize - 1];
                i -= 1;
            }
        }

        self.key[index as usize] = key_byte;
        let _ = std::mem::replace(&mut self.children[index as usize], new_child);
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
        node16.header = self.header;
        node16
    }
}

#[test]
#[should_panic]
fn node4_bad_grow() {
    let mut n4: Node4<String, i32, 8> = Default::default();
    let _ = n4.grow();
}

// #[test]
// fn node4_basic() {
//     struct Case {
//         key_byte: u8,
//         excepted_key: [u8; 4],
//         excepted_none_child: bool,
//         leaf_key: String,
//         leaf_val: i32,
//         excepted_children_count: u16,
//     }
//
//     let mut n4: Node4<String, i32, 8> = Default::default();
//     let mut cases: Vec<Case> = Vec::new();
//     cases.push(Case {
//         key_byte: 0,
//         excepted_key: [0, 255, 255, 255],
//         excepted_none_child: true,
//         leaf_key: "foo".to_string(),
//         leaf_val: 10,
//         excepted_children_count: 1,
//     });
//
//     cases.push(Case {
//         key_byte: 254,
//         excepted_key: [0, 254, 255, 255],
//         excepted_none_child: true,
//
//         leaf_key: "foz".to_string(),
//         leaf_val: 20,
//         excepted_children_count: 2,
//     });
//
//     cases.push(Case {
//         key_byte: 94,
//         excepted_key: [0, 94, 254, 255],
//         excepted_none_child: true,
//         leaf_key: "bar".to_string(),
//         leaf_val: 30,
//         excepted_children_count: 3,
//     });
//
//     cases.push(Case {
//         key_byte: 95,
//         excepted_key: [0, 94, 95, 254],
//         excepted_none_child: true,
//         leaf_key: "baz".to_string(),
//         leaf_val: 40,
//         excepted_children_count: 4,
//     });
//
//     for case in cases.iter() {
//         assert_eq!(
//             n4.get_child(false, case.key_byte).is_none(),
//             case.excepted_none_child
//         );
//         n4.add_child(
//             false,
//             case.key_byte,
//             ArtNode::leaf(case.leaf_key.clone(), case.leaf_val),
//         );
//         assert_eq!(n4.header.non_null_children, case.excepted_children_count);
//         assert_eq!(n4.key, case.excepted_key);
//         let node = n4.get_child(false, case.key_byte);
//         assert_eq!(node.is_some(), true);
//         match node.unwrap().get_ref() {
//             ArtNodeRef::Leaf(leaf) => {
//                 assert_eq!(leaf.key, case.leaf_key.clone());
//                 assert_eq!(leaf.val, case.leaf_val);
//             }
//             _ => unreachable!(),
//         }
//
//         let node = n4.get_mut_child(case.key_byte);
//         assert_eq!(node.is_some(), true);
//         match node.unwrap().get_mut() {
//             ArtNodeMut::Leaf(leaf) => {
//                 assert_eq!(leaf.key, case.leaf_key.clone());
//                 assert_eq!(leaf.val, case.leaf_val);
//             }
//             _ => unreachable!(),
//         }
//     }
//
//     n4.add_child(true, 255, ArtNode::leaf("prefix".to_string(), 50));
//     let node = n4.get_child(true, 255);
//     assert_eq!(node.is_some(), true);
//     match node.unwrap().get_ref() {
//         ArtNodeRef::Leaf(leaf) => {
//             assert_eq!(leaf.key, "prefix".to_string());
//             assert_eq!(leaf.val, 50);
//         }
//         _ => unreachable!(),
//     }
//
//     let node16 = n4.grow();
//     assert_eq!(node16.key[0..4], cases[3].excepted_key);
//     assert_eq!(node16.header.non_null_children, 4);
// }
