use crate::long_str::LongBStr;
use crate::short_str::ShortBStr;
use bstr::BStr;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GermanBStr<'a>(BStrInner<'a>);
pub use long_str::StrAllocator;

impl Default for GermanBStr<'_> {
    fn default() -> Self {
        Self::new_static(&[])
    }
}

#[cfg(feature = "bumpalo")]
mod bumpalo;
mod long_str;
mod short_str;

#[derive(Clone, Copy)]
#[repr(C)]
union BStrInner<'a> {
    head: BStrHead,
    short: ShortBStr,
    long: LongBStr<'a>,
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
    type Target = BStr;
    fn deref(&self) -> &BStr {
        BStr::new(if self.len() <= 12 {
            unsafe { &*self.0.short }
        } else {
            unsafe { &*self.0.long }
        })
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
                    long: LongBStr::$name(data),
                })
            }
        }
    };
}

impl<'a> GermanBStr<'a> {
    define_constructor!(new_static, &'static [u8]);
    define_constructor!(new_borrowed, &'a [u8]);

    pub fn reallocate_borrowed<'b>(self, dst: impl StrAllocator<'b>) -> GermanBStr<'b> {
        unsafe {
            if self.len() <= 12 {
                GermanBStr(BStrInner {
                    short: self.0.short,
                })
            } else {
                GermanBStr(BStrInner {
                    long: self.0.long.reallocate_borrowed(dst),
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
