use std::ops::Deref;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ShortBStr {
    /// # Safety len <=12
    len: u32,
    content: [u8; 12],
}

impl ShortBStr {
    pub fn new(content: &[u8]) -> Self {
        assert!(content.len() <= 12);
        let mut ret = ShortBStr {
            len: content.len() as u32,
            content: [0; 12],
        };
        ret.content[..content.len()].copy_from_slice(content);
        ret
    }
}

impl Deref for ShortBStr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { self.content.get_unchecked(..self.len as usize) }
    }
}
