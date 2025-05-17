use crate::StrAllocator;
use bumpalo::Bump;

unsafe impl<'a> StrAllocator for &'a Bump {
    fn allocate(self, data: &[u8]) -> &'a [u8] {
        self.alloc_slice_copy(data)
    }
}
