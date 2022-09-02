use std::cmp::min;
use std::marker::PhantomData;
use std::mem::take;
use std::ptr::copy_nonoverlapping;

use crate::leaf::Leaf;
use crate::node16::Node16;
use crate::node256::Node256;
use crate::node4::Node4;
use crate::node48::Node48;
use crate::ArtKey;
use crate::Header;

const NODE_TYPE_NONE: usize = 0;
const NODE_TYPE_N4: usize = 1;
const NODE_TYPE_N16: usize = 2;
const NODE_TYPE_N48: usize = 3;
const NODE_TYPE_N256: usize = 4;
const NODE_TYPE_LEAF: usize = 5;
const NODE_TYPE_MASK: usize = 7;
const NODE_PTR_MASK: usize = usize::MAX - NODE_TYPE_MASK;

// TODO: impl PartialEq for ArtNode
pub struct ArtNode<K: ArtKey, V, const MAX_PARTIAL_LEN: usize>(
    pub(crate) usize,
    PhantomData<K>,
    PhantomData<V>,
);

pub(crate) enum ArtNodeRef<'a, K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    None,
    Leaf(&'a Leaf<K, V>),
    Node4(&'a Node4<K, V, MAX_PARTIAL_LEN>),
    Node16(&'a Node16<K, V, MAX_PARTIAL_LEN>),
    Node48(&'a Node48<K, V, MAX_PARTIAL_LEN>),
    Node256(&'a Node256<K, V, MAX_PARTIAL_LEN>),
}

pub(crate) enum ArtNodeMut<'a, K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    None,
    Leaf(&'a mut Leaf<K, V>),
    Node4(&'a mut Node4<K, V, MAX_PARTIAL_LEN>),
    Node16(&'a mut Node16<K, V, MAX_PARTIAL_LEN>),
    Node48(&'a mut Node48<K, V, MAX_PARTIAL_LEN>),
    Node256(&'a mut Node256<K, V, MAX_PARTIAL_LEN>),
}

struct ArtKeyVerifier;

impl ArtKeyVerifier {
    #[inline(always)]
    fn valid(keys: &[u8], depth: usize) -> (u8, bool) {
        if depth < keys.len() {
            (keys[depth], true)
        } else {
            (0_u8, false)
        }
    }
}

struct LazyExpand;

impl LazyExpand {
    #[inline(always)]
    fn longest_common_prefix(leaf_key1: &[u8], leaf_key2: &[u8], depth: usize) -> usize {
        let max_len = min(leaf_key1.len(), leaf_key2.len()) - depth;
        for index in 0..max_len {
            if leaf_key1[depth + index] != leaf_key2[index + depth] {
                return index;
            }
        }

        return max_len;
    }

    /// Lazy expansion to remove path to single leaf: an existing leaf is encountered,
    /// it is replaced by a new inner node storing the existing and the new leaf .
    ///
    /// **Safety**: the existing_leaf must be a leaf node.
    #[inline]
    fn expand<K: ArtKey, V, const MAX_PARTIAL_LEN: usize>(
        node: ArtNode<K, V, MAX_PARTIAL_LEN>,
        key: K,
        val: V,
        depth: usize,
    ) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let new_leaf_key = key.get_bytes();
        let leaf_key = node.static_cast_ref_leaf().key.get_bytes();

        let longest_partial_len = LazyExpand::longest_common_prefix(leaf_key, new_leaf_key, depth);
        // copy matched longest prefix to node4
        let mut node4: Box<Node4<K, V, MAX_PARTIAL_LEN>> = Box::default();
        node4.header.partial.len = longest_partial_len as u32;
        let max_copy_len = min(MAX_PARTIAL_LEN, longest_partial_len);
        node4.header.partial.data[..max_copy_len]
            .copy_from_slice(&new_leaf_key[depth..depth + max_copy_len]);

        let mut new_node = ArtNode::node4(node4);
        let depth = depth + longest_partial_len;
        new_node.insert_child(ArtKeyVerifier::valid(leaf_key, depth), node);
        new_node.insert_child(
            ArtKeyVerifier::valid(new_leaf_key, depth),
            ArtNode::leaf(key, val),
        );
        new_node
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> ArtNode<K, V, MAX_PARTIAL_LEN> {
    pub(crate) fn get<'a>(
        root: &'a ArtNode<K, V, MAX_PARTIAL_LEN>,
        key: &[u8],
        depth: usize,
    ) -> Option<&'a V> {
        let mut depth = depth;
        let mut current: &ArtNode<K, V, MAX_PARTIAL_LEN> = &*root;
        while !current.is_none() {
            if current.is_leaf() {
                let leaf = current.static_cast_ref_leaf();
                // handles lazy expansion by checking that the
                // encountered leaf fully matches the key.
                if leaf.matches(key) {
                    return Some(&leaf.val);
                }
                return None;
            }

            let header = current.header();
            if header.partial.len > 0 {
                // handle pessimistic path compression: if the compressed path
                // does not match the key, aborting.
                let prefix_matched = current.check_prefix_match(key, depth);
                if prefix_matched != min(MAX_PARTIAL_LEN, header.partial.len as usize) {
                    return None;
                }
                depth += header.partial.len as usize
            }

            current = current.get_child(ArtKeyVerifier::valid(key, depth))?;
            depth += 1;
        }

        None
    }

    #[inline(always)]
    fn check_prefix_match(&self, key_byte: &[u8], depth: usize) -> usize {
        let header = self.header();
        let max_compare_len = min(
            min(MAX_PARTIAL_LEN, header.partial.len as usize),
            key_byte.len() - depth,
        );

        for i in 0..max_compare_len {
            if header.partial.data[i] != key_byte[depth + i] {
                return i;
            }
        }

        return max_compare_len;
    }

    #[inline(always)]
    fn get_child(&self, valid_key: (u8, bool)) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        match self.get_ref() {
            ArtNodeRef::None => None,
            ArtNodeRef::Leaf(_) => Some(self),
            ArtNodeRef::Node4(n4) => n4.get_child(valid_key),
            ArtNodeRef::Node16(n16) => n16.get_child(valid_key),
            ArtNodeRef::Node48(n48) => n48.get_child(valid_key),
            ArtNodeRef::Node256(n256) => n256.get_child(valid_key),
        }
    }

    pub(crate) fn insert(
        node: &mut ArtNode<K, V, MAX_PARTIAL_LEN>,
        key: K,
        val: V,
        depth: usize,
    ) -> Option<V> {
        let mut depth = depth;
        match node.get_mut() {
            ArtNodeMut::None => {
                *node = ArtNode::leaf(key, val);
                None
            }

            ArtNodeMut::Leaf(leaf) => {
                if leaf.matches(key.get_bytes()) {
                    // TODO: Can support leaf multi version?
                    return Some(std::mem::replace(&mut leaf.val, val));
                }
                // expand leaf
                *node = LazyExpand::expand::<K, V, MAX_PARTIAL_LEN>(
                    std::mem::take(node),
                    key,
                    val,
                    depth,
                );
                None
            }

            _ => {
                let header = node.header();
                if header.partial.len > 0 {
                    let mismatched_pos = ArtNode::<K, V, MAX_PARTIAL_LEN>::prefix_mismatch(
                        node,
                        header,
                        key.get_bytes(),
                        depth,
                    );

                    if mismatched_pos >= header.partial.len as usize {
                        depth += header.partial.len as usize;
                    } else {
                        node.compression(mismatched_pos, key, depth, val);
                        return None;
                    }
                }

                if let Some(child) =
                    node.get_mut_child(ArtKeyVerifier::valid(key.get_bytes(), depth))
                {
                    return ArtNode::insert(child, key, val, depth + 1);
                }

                // node.add_child_with_grow(false, key.get_bytes()key.get_bytes()[depth], ArtNode::leaf(key, val));
                node.assert_size();
                node.insert_child(
                    ArtKeyVerifier::valid(key.get_bytes(), depth),
                    ArtNode::leaf(key, val),
                );
                return None;
            }
        }
    }

    #[inline]
    fn compression(&mut self, prefix_mismatch_pos: usize, key: K, depth: usize, val: V) {
        let mut old_node = std::mem::replace(self, ArtNode::node4(Box::default()));
        // self is already new node

        let old_node_header = old_node.header();
        let new_node_header = self.header_mut();

        // copy matched partial from old node to new node.
        let max_copy_len = min(prefix_mismatch_pos, MAX_PARTIAL_LEN);
        new_node_header.partial.data[..max_copy_len]
            .copy_from_slice(&old_node_header.partial.data[..max_copy_len]);
        new_node_header.partial.len = prefix_mismatch_pos as u32;

        // Note: The match may exceed the maximum of the vector store, so take the minimum of both.
        // Artful uses mixed compression, So the actual storage length of partial can exceed the
        // maximum value of the vector.
        // pessimistic compression
        if old_node_header.partial.len as usize <= MAX_PARTIAL_LEN {
            let old_node_header = old_node.header_mut();
            let old_node_byte = old_node_header.partial.data[prefix_mismatch_pos];
            old_node_header.partial.len -= (prefix_mismatch_pos + 1) as u32;
            old_node_header
                .partial
                .data
                .rotate_left(prefix_mismatch_pos + 1);
            // unsafe {
            //     std::ptr::copy(
            //         old_node_header
            //             .partial
            //             .data
            //             .as_ptr()
            //             .offset(prefix_mismatch_pos as isize + 1),
            //         old_node_header.partial.data.as_mut_ptr(),
            //         min(MAX_PARTIAL_LEN, old_node_header.partial.len as usize),
            //     );
            // }
            self.insert_child((old_node_byte, true), old_node);
            self.insert_child(
                ArtKeyVerifier::valid(key.get_bytes(), depth + prefix_mismatch_pos),
                ArtNode::leaf(key, val),
            );

            return;
        }

        // optimistic compression
        // TODO: optimization the Header::default() to zero size.
        let mut old_node_header = std::mem::take(old_node.header_mut());
        let leaf =
            ArtNode::minimum_child(&old_node).expect("the inner node get minimum child fail");

        let leaf_key_bytes = leaf.key.get_bytes();
        let valid_key = ArtKeyVerifier::valid(leaf_key_bytes, depth + prefix_mismatch_pos);

        // TODO add proof.
        old_node_header.partial.len -= (prefix_mismatch_pos + 1) as u32;
        let max_copy_len = min(MAX_PARTIAL_LEN, old_node_header.partial.len as usize);
        let start = depth + prefix_mismatch_pos + 1;
        let end = start + max_copy_len;
        old_node_header.partial.data[..max_copy_len].copy_from_slice(&leaf_key_bytes[start..end]);

        std::mem::swap(&mut old_node_header, old_node.header_mut());
        // unsafe {
        //     copy_nonoverlapping(
        //         key_bytes
        //             .as_ptr()
        //             .offset((depth + prefix_mismatch_pos + 1) as isize),
        //         old_node.header_mut().partial.data.as_mut_ptr(),
        //         min(MAX_PARTIAL_LEN, old_node.header().partial.len as usize),
        //     )
        // }
        self.insert_child(valid_key, old_node);
        self.insert_child(
            ArtKeyVerifier::valid(key.get_bytes(), depth + prefix_mismatch_pos),
            ArtNode::leaf(key, val),
        );
    }

    #[inline]
    fn prefix_mismatch(
        node: &ArtNode<K, V, MAX_PARTIAL_LEN>,
        node_header: &Header<MAX_PARTIAL_LEN>,
        key: &[u8],
        depth: usize,
    ) -> usize {
        // Note: the length of partial can more than MAX_PARTIAL_LEN.
        let max_compare_len = min(
            min(MAX_PARTIAL_LEN, node_header.partial.len as usize),
            key.len() - depth,
        );
        let mut matched_index = 0;
        while matched_index < max_compare_len {
            if node_header.partial.data[matched_index] != key[matched_index + depth] {
                return matched_index;
            }
            matched_index += 1;
        }
        if node_header.partial.len as usize > MAX_PARTIAL_LEN {
            let leaf = ArtNode::minimum_child(node).expect("the inner node get minimum child fail");
            let leaf_key = leaf.key.get_bytes();
            let max_compare_len = min(leaf_key.len(), key.len()) - depth;
            while matched_index < max_compare_len {
                if leaf_key[depth + matched_index] != key[depth + matched_index] {
                    return matched_index;
                }

                matched_index += 1
            }
        }

        return matched_index;
    }

    pub(crate) fn remove(
        node: &mut ArtNode<K, V, MAX_PARTIAL_LEN>,
        key: &[u8],
        depth: usize,
    ) -> Option<V> {
        let mut depth = depth;
        let mut current: &mut ArtNode<K, V, MAX_PARTIAL_LEN> = node;
        while !current.is_none() && !current.is_leaf() {
            let header = current.header();
            if header.partial.len > 0 {
                let prefix_matched = current.check_prefix_match(key, depth);
                if prefix_matched != min(MAX_PARTIAL_LEN, header.partial.len as usize) {
                    return None;
                }
                depth += header.partial.len as usize
            }
            let next = current.get_mut_child(ArtKeyVerifier::valid(key, depth))?;
            current = next;
            depth = depth + 1;
            // ref mut child
        }

        match current.get_mut() {
            ArtNodeMut::None => None,
            ArtNodeMut::Leaf(leaf) => {
                return match leaf.matches(key) {
                    true => {
                        let mut child = current.remove_child(ArtKeyVerifier::valid(key, depth))?;
                        child.take_leaf()
                    }
                    false => None,
                }
            }
            _ => unreachable!(),
        }
    }

    fn remove_child(&mut self, valid_key: (u8, bool)) -> Option<ArtNode<K, V, MAX_PARTIAL_LEN>> {
        let removed_child = match self.get_mut() {
            ArtNodeMut::Node4(n4) => n4.remove_child(valid_key),
            ArtNodeMut::Node16(n16) => n16.remove_child(valid_key),
            ArtNodeMut::Node48(n48) => n48.remove_child(valid_key),
            ArtNodeMut::Node256(n256) => n256.remove_child(valid_key),
            ArtNodeMut::Leaf(_) => Some(std::mem::take(self)),
            _ => unreachable!(),
        };

        self.shrink_to_fit();
        removed_child
    }

    /// shrink_to_fit
    fn shrink_to_fit(&mut self) {
        if !self.is_few() {
            return;
        }

        // let mut taken_node = std::mem::take(self);
        let mut shrink_node = match self.get_mut() {
            ArtNodeMut::Node4(n4) => n4.shrink_to_fit(), // This fucking ugly.
            ArtNodeMut::Node16(n16) => ArtNode::node4(n16.shrink_to_fit()),
            ArtNodeMut::Node48(n48) => ArtNode::node16(n48.shrink_to_fit()),
            ArtNodeMut::Node256(n256) => ArtNode::node48(n256.shrink_to_fit()),
            _ => unreachable!(),
        };

        std::mem::swap(self, &mut shrink_node);
    }

    fn is_few(&self) -> bool {
        match self.get_ref() {
            ArtNodeRef::Node4(n4) => n4.is_few(),
            ArtNodeRef::Node16(n16) => n16.is_few(),
            ArtNodeRef::Node48(n48) => n48.is_few(),
            ArtNodeRef::Node256(n256) => n256.is_few(),
            ArtNodeRef::Leaf(_) | ArtNodeRef::None => false,
            // _ => unreachable!(),
        }
    }

    /// none actually no memory allocation.
    pub(crate) fn none() -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        ArtNode(NODE_TYPE_NONE, PhantomData, PhantomData)
    }

    pub(crate) fn node4(n4: Box<Node4<K, V, MAX_PARTIAL_LEN>>) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let ptr: *mut Node4<K, V, MAX_PARTIAL_LEN> = Box::into_raw(n4);
        let ptr_usize = ptr as usize;
        ArtNode(ptr_usize | NODE_TYPE_N4, PhantomData, PhantomData)
    }

    pub(crate) fn node16(
        n16: Box<Node16<K, V, MAX_PARTIAL_LEN>>,
    ) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let ptr: *mut Node16<K, V, MAX_PARTIAL_LEN> = Box::into_raw(n16);
        let ptr_usize = ptr as usize;
        ArtNode(ptr_usize | NODE_TYPE_N16, PhantomData, PhantomData)
    }

    pub(crate) fn node48(
        n48: Box<Node48<K, V, MAX_PARTIAL_LEN>>,
    ) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let ptr: *mut Node48<K, V, MAX_PARTIAL_LEN> = Box::into_raw(n48);
        let ptr_usize = ptr as usize;
        ArtNode(ptr_usize | NODE_TYPE_N48, PhantomData, PhantomData)
    }

    pub(crate) fn node256(
        n256: Box<Node256<K, V, MAX_PARTIAL_LEN>>,
    ) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let ptr: *mut Node256<K, V, MAX_PARTIAL_LEN> = Box::into_raw(n256);
        let ptr_usize = ptr as usize;
        ArtNode(ptr_usize | NODE_TYPE_N256, PhantomData, PhantomData)
    }

    pub(crate) fn leaf(key: K, val: V) -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        let leaf_ptr: *mut Leaf<K, V> = Box::into_raw(Box::new(Leaf::new(key, val)));
        let leaf_ptr_usize = leaf_ptr as usize;
        ArtNode(leaf_ptr_usize | NODE_TYPE_LEAF, PhantomData, PhantomData)
    }

    /// Safety: node never is leaf and none
    fn minimum_child(node: &ArtNode<K, V, MAX_PARTIAL_LEN>) -> Option<&Leaf<K, V>> {
        assert!(!node.is_none() && !node.is_leaf());
        let mut node = &*node;
        while !node.is_none() && !node.is_leaf() {
            let child = match node.get_ref() {
                ArtNodeRef::Node4(n4) => n4.minimum_child(),
                ArtNodeRef::Node16(n16) => n16.minimum_child(),
                ArtNodeRef::Node48(n48) => n48.minimum_child(),
                ArtNodeRef::Node256(n256) => n256.minimum_child(),
                _ => unreachable!(),
            }?;
            node = child;
        }

        match node.get_ref() {
            ArtNodeRef::None => None,
            ArtNodeRef::Leaf(leaf) => Some(leaf),
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_mut_child(
        &mut self,
        valid_key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        match self.get_mut() {
            ArtNodeMut::None | ArtNodeMut::Leaf(_) => None,
            ArtNodeMut::Node4(n4) => n4.get_mut_child(valid_key),
            ArtNodeMut::Node16(n16) => n16.get_mut_child(valid_key),
            ArtNodeMut::Node48(n48) => n48.get_mut_child(valid_key),
            ArtNodeMut::Node256(n256) => n256.get_mut_child(valid_key),
        }
    }
    pub(crate) const fn is_none(&self) -> bool {
        self.0 == NODE_TYPE_NONE
    }

    pub(crate) const fn is_leaf(&self) -> bool {
        self.0 & NODE_TYPE_MASK == NODE_TYPE_LEAF
    }

    fn insert_child(&mut self, valid_key: (u8, bool), new_child: ArtNode<K, V, MAX_PARTIAL_LEN>) {
        if self.is_full() {
            self.grow()
        }

        match self.get_mut() {
            ArtNodeMut::Node4(n4) => n4.insert_child(valid_key, new_child),
            ArtNodeMut::Node16(n16) => n16.insert_child(valid_key, new_child),
            ArtNodeMut::Node48(n48) => n48.insert_child(valid_key, new_child),
            ArtNodeMut::Node256(n256) => n256.insert_child(valid_key, new_child),
            _ => unreachable!(),
        }
    }

    fn assert_size(&self) {
        debug_assert_eq!(
            {
                let children: &[ArtNode<K, V, MAX_PARTIAL_LEN>] = match self.get_ref() {
                    ArtNodeRef::Node4(n4) => &n4.children,
                    ArtNodeRef::Node16(n16) => &n16.children,
                    ArtNodeRef::Node48(n48) => &n48.children,
                    ArtNodeRef::Node256(n256) => &n256.children,
                    _ => &[],
                };

                children.iter().filter(|child| !child.is_none()).count()
            },
            self.len()
        )
    }

    fn len(&self) -> usize {
        match self.get_ref() {
            ArtNodeRef::Node4(n4) => n4.header.non_null_children as usize,
            ArtNodeRef::Node16(n16) => n16.header.non_null_children as usize,
            ArtNodeRef::Node48(n48) => n48.header.non_null_children as usize,
            ArtNodeRef::Node256(n256) => n256.header.non_null_children as usize,
            _ => 0,
        }
    }

    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        match self.get_ref() {
            ArtNodeRef::Node4(n4) => n4.is_full(),
            ArtNodeRef::Node16(n16) => n16.is_full(),
            ArtNodeRef::Node48(n48) => n48.is_full(),
            ArtNodeRef::Node256(n256) => n256.is_full(),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn grow(&mut self) {
        // save old node header and taken it.
        let mut taken_node = std::mem::take(self);
        // start growing.
        *self = match taken_node.get_mut() {
            ArtNodeMut::Node4(n4) => ArtNode::node16(n4.grow()),
            ArtNodeMut::Node16(n16) => ArtNode::node48(n16.grow()),
            ArtNodeMut::Node48(n48) => ArtNode::node256(n48.grow()),
            _ => unreachable!(),
        };
        // Note: taken_node drop here.
    }

    pub(crate) fn header(&self) -> &Header<MAX_PARTIAL_LEN> {
        match self.get_ref() {
            ArtNodeRef::Node4(n4) => &n4.header,
            ArtNodeRef::Node16(n16) => &n16.header,
            ArtNodeRef::Node48(n48) => &n48.header,
            ArtNodeRef::Node256(n256) => &n256.header,
            _ => unreachable!(),
        }
    }

    pub(crate) fn header_mut(&mut self) -> &mut Header<MAX_PARTIAL_LEN> {
        match self.get_mut() {
            ArtNodeMut::Node4(n4) => &mut n4.header,
            ArtNodeMut::Node16(n16) => &mut n16.header,
            ArtNodeMut::Node48(n48) => &mut n48.header,
            ArtNodeMut::Node256(n256) => &mut n256.header,
            _ => unreachable!(),
        }
    }

    /// Convert node to ref leaf type. It's similar to the `static_cast` in C++ and
    /// equivalent asm following:
    /// ```asm
    /// example::ArtNode<K,V,_>::static_cast_ref_leaf:
    ///         push    rax
    ///         mov     qword ptr [rsp], rdi
    ///         mov     rax, qword ptr [rdi]
    ///         and     rax, 7
    ///         cmp     rax, 5
    ///         jne     .LBB58_2 # jmp unreachable!() if rax equal to five.
    ///         mov     rax, qword ptr [rsp]
    ///         mov     rax, qword ptr [rax]
    ///         and     rax, -8
    ///         pop     rcx
    ///         ret
    /// ```
    #[inline]
    const fn static_cast_ref_leaf(&self) -> &Leaf<K, V> {
        match self.0 & NODE_TYPE_MASK {
            NODE_TYPE_LEAF => {
                // mov rax, qword ptr [rax]
                let leaf_ptr: *const Leaf<K, V> = (self.0 & NODE_PTR_MASK) as *const Leaf<K, V>;
                let leaf_ref: &Leaf<K, V> = unsafe { &*leaf_ptr };
                leaf_ref
            }
            _ => unreachable!(),
        }
    }

    fn take_leaf(&mut self) -> Option<V> {
        let ptr = self.0;
        self.0 = 0;
        match ptr & NODE_TYPE_MASK {
            NODE_TYPE_LEAF => {
                let leaf_ptr = (ptr & NODE_PTR_MASK) as *mut Leaf<K, V>;
                let boxed = unsafe { Box::from_raw(leaf_ptr) };
                Some(boxed.val)
            }
            _ => unreachable!(),
        }
    }

    /// convert ArtNode to ArtNodeRef by type.
    ///
    /// If `self` it an inner node, it first convert the usize to a ptr and
    /// then get a const ref through the ptr.
    pub(crate) fn get_ref(&self) -> ArtNodeRef<'_, K, V, MAX_PARTIAL_LEN> {
        match self.0 & NODE_TYPE_MASK {
            NODE_TYPE_NONE => ArtNodeRef::None,
            NODE_TYPE_N4 => {
                let node_ptr: *const Node4<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *const Node4<K, V, MAX_PARTIAL_LEN>;
                let node_ref: &Node4<K, V, MAX_PARTIAL_LEN> = unsafe { &*node_ptr };
                ArtNodeRef::Node4(node_ref)
            }
            NODE_TYPE_N16 => {
                let node_ptr: *const Node16<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *const Node16<K, V, MAX_PARTIAL_LEN>;
                let node_ref: &Node16<K, V, MAX_PARTIAL_LEN> = unsafe { &*node_ptr };
                ArtNodeRef::Node16(node_ref)
            }
            NODE_TYPE_N48 => {
                let node_ptr: *const Node48<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *const Node48<K, V, MAX_PARTIAL_LEN>;
                let node_ref: &Node48<K, V, MAX_PARTIAL_LEN> = unsafe { &*node_ptr };
                ArtNodeRef::Node48(node_ref)
            }
            NODE_TYPE_N256 => {
                let node_ptr: *const Node256<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *const Node256<K, V, MAX_PARTIAL_LEN>;
                let node_ref: &Node256<K, V, MAX_PARTIAL_LEN> = unsafe { &*node_ptr };
                ArtNodeRef::Node256(node_ref)
            }
            NODE_TYPE_LEAF => {
                let leaf_ptr: *const Leaf<K, V> = (self.0 & NODE_PTR_MASK) as *const Leaf<K, V>;
                let leaf_ref: &Leaf<K, V> = unsafe { &*leaf_ptr };
                ArtNodeRef::Leaf(leaf_ref)
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn get_mut(&mut self) -> ArtNodeMut<'_, K, V, MAX_PARTIAL_LEN> {
        match self.0 & NODE_TYPE_MASK {
            NODE_TYPE_NONE => ArtNodeMut::None,
            NODE_TYPE_N4 => {
                let node_ptr: *mut Node4<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *mut Node4<K, V, MAX_PARTIAL_LEN>;
                let node_mut: &mut Node4<K, V, MAX_PARTIAL_LEN> = unsafe { &mut *node_ptr };
                ArtNodeMut::Node4(node_mut)
            }
            NODE_TYPE_N16 => {
                let node_ptr: *mut Node16<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *mut Node16<K, V, MAX_PARTIAL_LEN>;
                let node_mut: &mut Node16<K, V, MAX_PARTIAL_LEN> = unsafe { &mut *node_ptr };
                ArtNodeMut::Node16(node_mut)
            }
            NODE_TYPE_N48 => {
                let node_ptr: *mut Node48<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *mut Node48<K, V, MAX_PARTIAL_LEN>;
                let node_mut: &mut Node48<K, V, MAX_PARTIAL_LEN> = unsafe { &mut *node_ptr };
                ArtNodeMut::Node48(node_mut)
            }
            NODE_TYPE_N256 => {
                let node_ptr: *mut Node256<K, V, MAX_PARTIAL_LEN> =
                    (self.0 & NODE_PTR_MASK) as *mut Node256<K, V, MAX_PARTIAL_LEN>;
                let node_mut: &mut Node256<K, V, MAX_PARTIAL_LEN> = unsafe { &mut *node_ptr };
                ArtNodeMut::Node256(node_mut)
            }
            NODE_TYPE_LEAF => {
                let leaf_ptr: *mut Leaf<K, V> = (self.0 & NODE_PTR_MASK) as *mut Leaf<K, V>;
                let leaf_mut: &mut Leaf<K, V> = unsafe { &mut *leaf_ptr };
                ArtNodeMut::Leaf(leaf_mut)
            }
            _ => unreachable!(),
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Drop for ArtNode<K, V, MAX_PARTIAL_LEN> {
    fn drop(&mut self) {
        match self.0 & NODE_TYPE_MASK {
            NODE_TYPE_NONE => {}
            NODE_TYPE_N4 => {
                let ptr = (self.0 & NODE_PTR_MASK) as *mut Node4<K, V, MAX_PARTIAL_LEN>;
                drop(unsafe { Box::from_raw(ptr) });
            }
            NODE_TYPE_N16 => {
                let ptr = (self.0 & NODE_PTR_MASK) as *mut Node16<K, V, MAX_PARTIAL_LEN>;
                drop(unsafe { Box::from_raw(ptr) });
            }
            NODE_TYPE_N48 => {
                let ptr = (self.0 & NODE_PTR_MASK) as *mut Node48<K, V, MAX_PARTIAL_LEN>;
                drop(unsafe { Box::from_raw(ptr) });
            }
            NODE_TYPE_N256 => {
                let ptr = (self.0 & NODE_PTR_MASK) as *mut Node256<K, V, MAX_PARTIAL_LEN>;
                drop(unsafe { Box::from_raw(ptr) });
            }
            NODE_TYPE_LEAF => {
                let ptr = (self.0 & NODE_PTR_MASK) as *mut Leaf<K, V>;
                drop(unsafe { Box::from_raw(ptr) });
            }
            _ => unreachable!(),
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for ArtNode<K, V, MAX_PARTIAL_LEN> {
    fn default() -> ArtNode<K, V, MAX_PARTIAL_LEN> {
        ArtNode::none()
    }
}
