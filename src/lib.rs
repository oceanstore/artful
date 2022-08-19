extern crate core;

use std::alloc::alloc;
use std::alloc::dealloc;
use std::alloc::handle_alloc_error;
use std::ptr::copy;
use std::ptr::copy_nonoverlapping;
use std::ptr::NonNull;
use std::vec;

use crate::node::ArtNode;

pub trait ArtKey {
    fn get_bytes(&self) -> &[u8];

    fn get_mut_bytes(&mut self) -> &mut [u8];
}

impl ArtKey for i32 {
    fn get_bytes(&self) -> &[u8] {
        let ptr = self as *const i32 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, 4) }
    }

    fn get_mut_bytes(&mut self) -> &mut [u8] {
        let ptr = self as *mut i32 as *mut u8;
        unsafe { std::slice::from_raw_parts_mut(ptr, 4) }
    }
}

impl ArtKey for i64 {
    fn get_bytes(&self) -> &[u8] {
        let ptr = self as *const i64 as *const u8;
        unsafe { std::slice::from_raw_parts(ptr, 8) }
    }

    fn get_mut_bytes(&mut self) -> &mut [u8] {
        let ptr = self as *mut i64 as *mut u8;
        unsafe { std::slice::from_raw_parts_mut(ptr, 8) }
    }
}

impl ArtKey for std::string::String {
    fn get_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn get_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { self.as_bytes_mut() }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Partial<const MAX_PARTIAL_LEN: usize> {
    pub(crate) data: [u8; MAX_PARTIAL_LEN],
    pub(crate) len: u32,
}

impl<const MAX_PARTIAL_LEN: usize> Default for Partial<MAX_PARTIAL_LEN> {
    fn default() -> Partial<MAX_PARTIAL_LEN> {
        Partial {
            data: [0_u8; MAX_PARTIAL_LEN],
            len: 0,
        }
    }
}

// impl<const MAX_PARTIAL_LEN: usize> From<&Partial<MAX_PARTIAL_LEN>> for Partial<MAX_PARTIAL_LEN> {
//     fn from(other: &Partial<MAX_PARTIAL_LEN>) -> Partial<MAX_PARTIAL_LEN> {
//         Partial {
//             data: other.data,
//             len: other.len,
//         }
//     }
// }

// impl<const MAX_PARTIAL_LEN: usize> From<&[u8; MAX_PARTIAL_LEN]> for Partial<MAX_PARTIAL_LEN> {
//     fn from(other: &[u8; MAX_PARTIAL_LEN]) -> Partial<MAX_PARTIAL_LEN> {
//         Partial {
//             data: other.clone(),
//             len: MAX_PARTIAL_LEN as u32,
//         }
//     }
// }

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Header<const MAX_PARTIAL_LEN: usize> {
    // pub(crate) prefix: NonNull<u8>,
    // pub(crate) prefix_len: u32,
    pub(crate) partial: Partial<MAX_PARTIAL_LEN>,
    pub(crate) non_null_children: u16, // numebr of none-null children
                                       // padding 3
}

// impl Clone for Header {
//     fn clone(&self) -> Header {
//         let mut other = Header {
//             prefix: HeaderPrefix::default()
//             non_null_children: self.non_null_children,
//         };

//         if self.prefix_len != 0 {
//             other.prefix = Header::alloc_prefix(self.prefix_len as usize);
//             unsafe {
//                 copy_nonoverlapping(
//                     self.prefix.as_ptr(),
//                     other.prefix.as_ptr() as *mut u8,
//                     self.prefix_len as usize,
//                 );
//             }
//         };

//         other
//     }
// }

impl<const MAX_PARTIAL_LEN: usize> Header<MAX_PARTIAL_LEN> {
    // #[inline]
    // pub(crate) fn init_prefix(&mut self, src: *const u8, count: usize) {
    //     assert!(self.prefix_len == 0, "prefix alreay initialize");

    //     if count == 0 {
    //         return;
    //     }

    //     self.prefix = Header::alloc_prefix(count);
    //     unsafe { copy_nonoverlapping(src, self.prefix.as_ptr() as *mut u8, count) }
    //     self.prefix_len = count as u32;
    // }

    // pub(crate) fn cut_prefix(&mut self, pos: usize) {
    //     let old_header = std::mem::take(self);
    //     let prefix = Header::alloc_prefix(old_header.prefix_len as usize - pos);
    //     unsafe {
    //         copy_nonoverlapping(
    //             old_header.prefix.as_ptr().offset(pos as isize + 1),
    //             prefix.as_ptr(),
    //             old_header.prefix_len as usize - pos - 1,
    //         );
    //     }
    //     println!("{}", self.prefix_len)
    // }

    // fn alloc_prefix(len: usize) -> NonNull<u8> {
    //     let layout = std::alloc::Layout::from_size_align(len, std::mem::align_of::<u8>())
    //         .expect("Bac layout for header prefix");

    //     unsafe {
    //         let ptr = alloc(layout);
    //         match NonNull::new(ptr as *mut u8) {
    //             Some(p) => return p,
    //             None => handle_alloc_error(layout),
    //         };
    //     }
    // }
}

// impl Drop for Header {
//     fn drop(&mut self) {
//         if self.prefix_len != 0 {
//             unsafe {
//                 dealloc(
//                     self.prefix.as_ptr() as *mut u8,
//                     std::alloc::Layout::from_size_align(
//                         self.prefix_len as usize,
//                         std::mem::align_of::<u8>(),
//                     )
//                     .expect("Bac layout for header prefix"),
//                 )
//             }
//         }
//     }
// }

mod leaf;
mod node;
mod node16;
mod node256;
mod node4;
mod node48;

pub mod art;
pub use art::Art;
