use std::marker::PhantomData;
use std::ops::Deref;
use std::{mem, ptr, slice};

/// # Safety
/// returned slice must have same length as input slice.
pub unsafe trait StrAllocator<'a> {
    fn allocate(self, data: &[u8]) -> &'a [u8];
}

#[repr(C)]
pub struct LongBStr<'a> {
    /// # Safety len >12
    len: u32,
    head: [u8; 4],
    content: *const u8,
    _p: PhantomData<&'a [u8]>,
}

const CLASS_BITS: u32 = 2;
const CLASS_BIT_SHIFT: u32 = 64 - CLASS_BITS;

#[derive(Clone, Copy)]
#[repr(usize)]
enum Class {
    Borrowed,
    Static,
    Owned,
}

impl<'a> LongBStr<'a> {
    unsafe fn new(data: *const [u8], class: Class) -> Self {
        assert!(data.len() > 12);
        LongBStr {
            len: data.len().try_into().unwrap(),
            head: unsafe { (data as *const [u8; 4]).read_unaligned() },
            content: Self::make_tag(data as *const u8, class),
            _p: PhantomData,
        }
    }

    fn make_tag(x: *const u8, tag: Class) -> *const u8 {
        debug_assert!((tag as usize) < (1 << CLASS_BITS));
        x.map_addr(|x| {
            assert_eq!(x >> CLASS_BITS, 0);
            x | ((tag as usize) << CLASS_BIT_SHIFT)
        })
    }

    fn tag(&self) -> Class {
        unsafe { mem::transmute(self.content.addr() >> CLASS_BITS) }
    }

    fn ptr(&self) -> *const u8 {
        self.content.map_addr(|p| p & (usize::MAX >> CLASS_BITS))
    }

    pub fn new_static(data: &'static [u8]) -> Self {
        unsafe { Self::new(data, Class::Static) }
    }

    pub fn new_borrowed(data: &'a [u8]) -> Self {
        unsafe { Self::new(data, Class::Borrowed) }
    }

    pub fn new_boxed(data: &[u8]) -> Self {
        assert!(data.len() > 12);
        let data = Box::into_raw(data.to_vec().into_boxed_slice());
        unsafe { Self::new(data, Class::Owned) }
    }

    pub fn reallocate_borrowed<'b>(self, dst: impl StrAllocator<'b>) -> LongBStr<'b> {
        match self.tag() {
            Class::Borrowed => {
                let new_ptr = dst.allocate(&self).as_ptr();
                LongBStr {
                    content: Self::make_tag(new_ptr, Class::Borrowed),
                    _p: Default::default(),
                    ..self
                }
            }
            Class::Owned | Class::Static => unsafe {
                mem::transmute::<LongBStr<'a>, LongBStr<'b>>(self)
            },
        }
    }
}

impl Drop for LongBStr<'_> {
    fn drop(&mut self) {
        match self.tag() {
            Class::Borrowed | Class::Static => (),
            Class::Owned => unsafe {
                drop(Box::from_raw(ptr::slice_from_raw_parts_mut(
                    self.ptr() as *mut u8,
                    self.len as usize,
                )));
            },
        }
    }
}

impl Deref for LongBStr<'_> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr(), self.len as usize) }
    }
}
