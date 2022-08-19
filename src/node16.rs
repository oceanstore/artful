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

const FULL_NODE_SIZE: u16 = 16;

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
        // because it need clone and our intilization with occur on
        // the insert critial performacne path. so, we manual do it.
        Node16 {
            header: Default::default(),
            key: [255; 16],
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

    /// Safety: grow.
    pub(crate) fn insert_child(
        &mut self,
        valid_key: (u8, bool),
        mut new_child: ArtNode<K, V, MAX_PARTIAL_LEN>,
    ) {
        assert!(self.header.non_null_children < 16);
        // #[cfg(feature = "simd")]
        // #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        // {
        //     let idx = unsafe {
        //         // vectorized node16 keys.
        //         let simd_keys = _mm_loadu_si128(self.key.as_ptr() as *const __m128i);
        //         // create selection mask by search byte
        //         let simd_mask = _mm_set1_epi8(std::mem::transmute::<u8, i8>(key_byte));
        //         let vcmp = _mm_cmplt_epi8(simd_keys, simd_mask);
        //         let vresult = _mm_cmpeq_epi8(vcmp, _mm_set1_epi8(1));
        //         let mut result: [u8; 16] = [0; 16];
        //         _mm_store_si128(result.as_mut_ptr() as *mut __m128i, vresult);
        //         println!(
        //             "{:?} {}",
        //             result,
        //             _mm_movemask_epi8(vresult).trailing_zeros()
        //         );
        //         _mm_movemask_epi8(vresult).trailing_zeros() as usize
        //     };
        //
        //     let idx = idx.min(self.header.non_null_children as usize);
        //     println!("keys = {:?} idx = {}", self.key, idx);
        //     if !self.children[idx].is_none() {
        //         let mut i = self.header.non_null_children as usize;
        //         while i > idx {
        //             self.children[i as usize] = std::mem::take(&mut self.children[i as usize - 1]);
        //             self.key[i as usize] = self.key[i as usize - 1];
        //             i -= 1;
        //         }
        //     }
        //     self.key[idx] = key_byte;
        //     self.children[idx] = new_child;
        //     self.header.non_null_children += 1;
        //     return;
        // }

        // if prefixed {
        //     assert!(self.prefixed_child.is_none());
        //     swap(&mut self.prefixed_child, &mut new_child);
        //     return;
        // }
        //

        if !valid_key.1 {
            assert!(self.prefixed_child.is_none());
            std::mem::swap(&mut self.prefixed_child, &mut new_child);
            return;
        }

        // find first index greater than or equal to key_byte
        // TODO: use simd?
        let mut index = 0;
        while (index < self.header.non_null_children) && self.key[index as usize] < valid_key.0 {
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

        self.key[index as usize] = valid_key.0;
        self.children[index as usize] = new_child;
        self.header.non_null_children += 1;
    }

    #[inline]
    pub(crate) fn get_child(&self, key: (u8, bool)) -> Option<&ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&self.prefixed_child);
        }

        let index = self.find_child_index(key.0)?;
        return Some(&self.children[index]);
    }

    pub(crate) fn get_mut_child(
        &mut self,
        key: (u8, bool),
    ) -> Option<&mut ArtNode<K, V, MAX_PARTIAL_LEN>> {
        if !key.1 {
            return Some(&mut self.prefixed_child);
        }

        let index = self.find_child_index(key.0)?;
        Some(&mut self.children[index])
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
        // copy header
        // node48.header.partial.len = self.header.partial.len;
        // node48.header.non_null_children = self.header.non_null_children;
        // unsafe {
        //     copy_nonoverlapping(
        //         self.header.partial.data.as_ptr(),
        //         node48.header.partial.data.as_mut_ptr(),
        //         self.header.partial.len as usize,
        //     )
        // }

        // copy the old node header to the new grown node.
        node48.header = self.header;
        node48
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

#[test]
fn basic_node16_get_child() {
    // none child case.
    let mut n6: Node16<String, i32, 8> = Default::default();
    assert_eq!(n6.get_child((255, true)).is_none(), true);
    assert_eq!(n6.get_child((179, true)).is_none(), true);

    // has child case.
    n6.key[10] = 255;
    n6.children[10] = ArtNode::leaf(255.to_string(), 255 as i32);
    n6.header.non_null_children = 1;
    assert_eq!(n6.get_child((255, true)).is_none(), false);

    n6.header.non_null_children = 0;
    for i in 0..16 {
        n6.key[i] = (i * 4) as u8;
        n6.children[i] = ArtNode::leaf(i.to_string(), i as i32);
        n6.header.non_null_children += 1;
    }

    assert_eq!(n6.get_child((255, true)).is_none(), true);
    assert_eq!(n6.get_child((179, true)).is_none(), true);

    for i in (0..16).rev() {
        assert_eq!(n6.get_child(((i * 4) as u8, true)).is_some(), true);
    }
}

// 0 .. 15
#[test]
fn basic_sse2_add() {
    let mut n6: Node16<String, i32, 8> = Default::default();
    for i in 8..16 {
        let kb = i * 10 as u8;
        // n6.key[i] = kb;
        n6.insert_child((kb, true), ArtNode::leaf(kb.to_string(), kb as i32));
    }
    println!("{:?}", n6.key);

    for i in 0..8 {
        let kb = i * 4 as u8;
        // n6.key[i] = i as u8;
        n6.insert_child((kb, true), ArtNode::leaf(kb.to_string(), kb as i32));
    }

    println!("{:?}", n6.key)
}
