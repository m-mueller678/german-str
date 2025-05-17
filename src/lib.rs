use crate::long_str::LongBStr;
use crate::short_str::ShortBStr;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::mem;
use std::mem::ManuallyDrop;
use std::ops::Deref;

#[repr(transparent)]
pub struct GermanBStr<'a>(BStrInner<'a>);
pub use long_str::StrAllocator;

impl Clone for GermanBStr<'_> {
    fn clone(&self) -> Self {
        unsafe {
            if self.len() <= 12 {
                GermanBStr(BStrInner {
                    short: self.0.short,
                })
            } else {
                GermanBStr(BStrInner {
                    long: self.0.long.clone(),
                })
            }
        }
    }
}

impl Default for GermanBStr<'_> {
    fn default() -> Self {
        Self::new_static(&[])
    }
}

#[cfg(feature = "bumpalo")]
mod bumpalo;
mod long_str;
mod short_str;

#[repr(C)]
union BStrInner<'a> {
    head: BStrHead,
    short: ShortBStr,
    long: ManuallyDrop<LongBStr<'a>>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(C)]
struct BStrHead {
    len: u32,
    head: [u8; 4],
}

impl GermanBStr<'_> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.head().len as usize
    }

    fn head_bytes(&self) -> &[u8; 4] {
        &self.head().head
    }

    fn head(&self) -> &BStrHead {
        unsafe { &self.0.head }
    }
}

impl Deref for GermanBStr<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        if self.len() <= 12 {
            unsafe { &self.0.short }
        } else {
            unsafe { &self.0.long }
        }
    }
}

macro_rules! define_constructor {
    ($name:ident,$ty:ty) => {
        pub fn $name(data: $ty) -> Self {
            if data.len() <= 12 {
                GermanBStr(BStrInner {
                    short: ShortBStr::new(data),
                })
            } else {
                GermanBStr(BStrInner {
                    long: ManuallyDrop::new(LongBStr::$name(data)),
                })
            }
        }
    };
}

impl<'a> GermanBStr<'a> {
    define_constructor!(new_static, &'static [u8]);
    define_constructor!(new_borrowed, &'a [u8]);
    define_constructor!(new_boxed, &[u8]);

    pub fn reallocate_borrowed<'b>(mut self, dst: impl StrAllocator<'b>) -> GermanBStr<'b> {
        unsafe {
            if self.len() <= 12 {
                GermanBStr(BStrInner {
                    short: self.0.short,
                })
            } else {
                let long = ManuallyDrop::take(&mut self.0.long);
                mem::forget(self);
                GermanBStr(BStrInner {
                    long: ManuallyDrop::new(long.reallocate_borrowed(dst)),
                })
            }
        }
    }
}

impl PartialOrd for GermanBStr<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GermanBStr<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.head_bytes()
            .cmp(other.head_bytes())
            .then_with(|| <[u8]>::cmp(self, other))
    }
}

impl PartialEq for GermanBStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.head() == other.head() && <[u8]>::eq(self, &**other)
    }
}

impl Eq for GermanBStr<'_> {}

impl Hash for GermanBStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl Drop for GermanBStr<'_> {
    fn drop(&mut self) {
        if self.len() > 12 {
            unsafe { ManuallyDrop::drop(&mut self.0.long) }
        }
    }
}
