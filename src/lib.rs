pub mod art;
mod leaf;
mod node;
mod node16;
mod node256;
mod node4;
mod node48;
mod simd;
pub use art::Art;

use std::alloc::alloc;
use std::alloc::dealloc;
use std::alloc::handle_alloc_error;
use std::ptr::copy;
use std::ptr::copy_nonoverlapping;
use std::ptr::NonNull;
use std::vec;

use crate::node::ArtNode;

/// A trait some constraints on the key of art.
///
/// Artful implements this trait for most of the built-in types. If you want to
/// customize the type as an artful key, you will need to implement the trait.
pub trait ArtKey: Default {
    /// Returns a reference to a byte slice from a particular type.
    fn get_bytes(&self) -> &[u8];

    /// Returns a mutable reference to a byte slice from a particular type.
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

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Header<const MAX_PARTIAL_LEN: usize> {
    pub(crate) partial: Partial<MAX_PARTIAL_LEN>,
    pub(crate) non_null_children: u16,
}
