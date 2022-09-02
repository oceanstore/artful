use crate::node::ArtNode;
use crate::node4::Node4;
use crate::node48::Node48;
use crate::Header;
use crate::{Art, ArtKey};
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::ptr::copy_nonoverlapping;

pub(crate) struct Node16<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> {
    pub(crate) header: Header<MAX_PARTIAL_LEN>,
    pub(crate) key: [u8; 16],
    pub(crate) children: [ArtNode<K, V, MAX_PARTIAL_LEN>; 16],
    pub(crate) prefixed_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Default for Node16<K, V, MAX_PARTIAL_LEN> {
    fn default() -> Node16<K, V, MAX_PARTIAL_LEN> {
        // Why dont' i use macro `vec![]` initialize the children?
        // just like, you know `vec![ArtNode::none(); 16].try_into()...`.
        // because it need clone and our initialization with occur on
        // the insert critical performance path. so, we manual do it.
        Node16 {
            header: Default::default(),
            key: [0; 16],
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
            ],
        }
    }
}

impl<K: ArtKey, V, const MAX_PARTIAL_LEN: usize> Node16<K, V, MAX_PARTIAL_LEN> {
    #[inline(always)]
    pub(crate) fn is_full(&self) -> bool {
        self.header.non_null_children == 16
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
    fn find_less_than_index(&mut self, key: u8) -> u16 {
        let mask = (1 << self.header.non_null_children) - 1;
        #[cfg(feature = "simd")]
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_x86_feature_detected!("sse2") {
            unsafe {
                let lt = _mm_cmplt_epi8(
                    _mm_set1_epi8(std::mem::transmute::<u8, i8>(key)),
                    _mm_loadu_si128(self.key.as_ptr() as *const __m128i),
                );
                let bit_fields = _mm_movemask_epi8(lt) & mask;
                if bit_fields != 0 {
                    return bit_fields.trailing_zeros() as u16;
                }

                return self.header.non_null_children;
            }
        }

        #[cfg(feature = "simd")]
        #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe {
                let lt = vcltq_u8(vdupq_n_u8(key), vld1q_u8(self.key.as_ptr()));
                let bit_fields = _mm_movemask_epi8(lt) & mask;
                if bit_fields != 0 {
                    return bit_fields.trailing_zeros() as u16;
                }

                return self.header.non_null_children;
            }
        }

        let mut index = 0;
        while (index < self.header.non_null_children) && self.key[index as usize] < key {
            index += 1;
        }

        index
    }

    /// Safety: grow.
    pub(crate) fn insert_child(
        &mut self,
        valid_key: (u8, bool),
        mut new_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
    ) {
        assert!(self.header.non_null_children < 16);
        if !valid_key.1 {
            assert!(self.prefixed_child.is_none());
            std::mem::swap(&mut self.prefixed_child, &mut new_child);
            return;
        }

        let index = self.find_less_than_index(valid_key.0);
        if !self.children[index as usize].is_none() {
            let mut i = self.header.non_null_children;
            while i > index {
                let mut moved = std::mem::take(&mut self.children[i as usize - 1]);
                std::mem::swap(&mut self.children[i as usize], &mut moved);
                // self.children[i as usize] = std::mem::take(&mut self.children[i as usize - 1]);
                self.key[i as usize] = self.key[i as usize - 1];
                i -= 1;
            }
        }

        self.key[index as usize] = valid_key.0;
        std::mem::swap(&mut self.children[index as usize], &mut new_child);
        // self.children[index as usize] = new_child;
        self.header.non_null_children += 1;
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
        // simd is preferred
        #[cfg(feature = "simd")]
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        if is_x86_feature_detected!("sse2") {
            unsafe {
                // vectorized node16 keys.
                // create selection mask by search byte
                let mask = _mm_set1_epi8(std::mem::transmute::<u8, i8>(key));
                // if byte in node16, where all elem in simd_keys that are equal to simd_mask equal
                // one, else 0.
                // vectorized node16 keys.
                let cmp =
                    _mm_cmpeq_epi8(mask, _mm_loadu_si128(self.key.as_ptr() as *const __m128i));
                return match _mm_movemask_epi8(cmp) & ((1 << self.header.non_null_children) - 1) {
                    0 => None,
                    bit_fields => Some(bit_fields.trailing_zeros() as usize),
                };
            };
        }

        #[cfg(feature = "simd")]
        #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
        if std::arch::is_aarch64_feature_detected!("neon") {
            unsafe {
                let simd_keys = vld1q_u8(self.key.as_ptr());
                let simd_mask = vdupq_n_u8(key);
                let eq = vceqq_u8(simd_mask, simd_keys);
                let mask = (1 << self.header.non_null_children) - 1;
                let bit_fields = _mm_movemask_epi8(eq) & mask;
                if bit_fields == 0 {
                    return None;
                }
                return Some(bit_fields.trailing_zeros() as usize);
            }
        }
        // slow path, binary search used.
        match self.key[0..self.header.non_null_children as usize].binary_search(&key) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        }
    }

    #[inline(always)]
    pub(crate) fn grow(&mut self) -> Box<Node48<K, V, MAX_PARTIAL_LEN>> {
        let mut node48: Box<Node48<K, V, MAX_PARTIAL_LEN>> = Box::default();
        // copy invalid child
        std::mem::swap(&mut self.prefixed_child, &mut node48.prefixed_child);
        // copy children and key
        for i in 0..self.header.non_null_children {
            std::mem::swap(
                &mut self.children[i as usize],
                &mut node48.children[i as usize],
            );
            node48.child_index[self.key[i as usize] as usize] = i as u8;
        }
        // copy the old node header to the new grown node.
        node48.header.partial.clone_from(&self.header.partial);
        node48.header.non_null_children = self.header.non_null_children;
        // node48.header = self.header;
        node48
    }

    #[inline(always)]
    pub fn is_few(&self) -> bool {
        self.header.non_null_children < 5
    }

    pub(crate) fn remove_child(
        &mut self,
        valid_key: (u8, bool),
    ) -> Option<ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !valid_key.1 {
            assert_eq!(self.prefixed_child.is_none(), false);
            return Some(std::mem::take(&mut self.prefixed_child));
        }

        let idx = self.find_child_index(valid_key.0)?;

        // TODO: 这一次的查找可以优化为从外部传递.
        let mut idx = self.find_child_index(valid_key.0)?;
        let child = std::mem::take(&mut self.children[idx]);
        self.key[idx] = 0;
        self.header.non_null_children -= 1;

        // to keep order
        while idx < self.header.non_null_children as usize {
            self.key[idx] = self.key[idx + 1];
            let mut moved = std::mem::take(&mut self.children[idx + 1]);
            std::mem::swap(&mut self.children[idx], &mut moved);
            idx += 1
        }

        // again remaining
        while idx < 16 {
            if !self.children[idx].is_none() {
                break;
            }
            let _ = std::mem::take(&mut self.children[idx]);
            idx += 1
        }

        Some(child)
    }

    pub(crate) fn shrink_to_fit(&mut self) -> Box<Node4<K, V, MAX_PARTIAL_LEN>> {
        let mut node4: Box<Node4<K, V, MAX_PARTIAL_LEN>> = Box::default();
        let mut node4_index = 0;
        for i in 0..self.header.non_null_children as usize {
            std::mem::swap(&mut self.children[i], &mut node4.children[node4_index]);
            node4.key[node4_index] = self.key[i];
            node4_index += 1;
        }

        std::mem::swap(&mut self.prefixed_child, &mut node4.prefixed_child);
        // node4.header = self.header;
        node4.header.partial.clone_from(&self.header.partial);
        node4.header.non_null_children = node4_index as u16;
        node4
    }
}

#[inline]
#[cfg(feature = "simd")]
#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
unsafe fn _mm_movemask_epi8(input: uint8x16_t) -> i32 {
    // Example input (half scale):
    // 0x89 FF 1D C0 00 10 99 33
    // Shift out everything but the sign bits
    // 0x01 01 00 01 00 00 01 00
    let high_bits = vreinterpretq_u16_u8(vshrq_n_u8::<7>(input));
    // Merge the even lanes together with vsra. The '??' bytes are garbage.
    // vsri could also be used, but it is slightly slower on aarch64.
    // 0x??03 ??02 ??00 ??01
    let paired16 = vreinterpretq_u32_u16(vsraq_n_u16::<7>(high_bits, high_bits));
    // Repeat with wider lanes.
    // 0x??????0B ??????04
    let paired32 = vreinterpretq_u64_u32(vsraq_n_u32::<14>(paired16, paired16));
    // 0x??????????????4B
    let paired64 = vreinterpretq_u8_u64(vsraq_n_u64::<28>(paired32, paired32));
    // Extract the low 8 bits from each lane and join.
    // 0x4B
    vgetq_lane_u8::<0>(paired64) as i32 | (vgetq_lane_u8::<8>(paired64) as i32) << 8
}
