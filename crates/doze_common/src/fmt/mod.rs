use core::{
    ffi::CStr,
    fmt::{Result, Write},
};

pub struct Message<'b> {
    buf: &'b mut [u8],
    len: usize,
}

impl<'b> Message<'b> {
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }
    pub fn remaining(&self) -> usize {
        self.capacity() - self.len
    }

    pub fn as_str(&self) -> &str {
        // safe: only written via fmt::Write which guarantees valid UTF-8
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

impl Write for Message<'_> {
    fn write_str(&mut self, s: &str) -> Result {
        let len = s.floor_char_boundary(self.remaining());

        self.buf[self.len..self.len + len].copy_from_slice(&s.as_bytes()[..len]);
        self.len += len;

        if len < s.len() {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

pub struct NullTermMessage<'b> {
    inner: Message<'b>,
}

impl<'b> NullTermMessage<'b> {
    pub fn new(buf: &'b mut [u8]) -> Self {
        if let Some(b) = buf.get_mut(0) {
            *b = 0;
        }
        Self {
            inner: Message::new(buf),
        }
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn capacity(&self) -> usize {
        self.inner.capacity().saturating_sub(1)
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub fn is_full(&self) -> bool {
        self.inner.len() == self.capacity()
    }
    pub fn remaining(&self) -> usize {
        self.capacity() - self.inner.len()
    }
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
    pub fn clear(&mut self) {
        self.inner.clear();
        if let Some(b) = self.inner.buf.get_mut(0) {
            *b = 0;
        }
    }
    pub fn as_cstr(&self) -> Option<&CStr> {
        self.inner
            .buf
            .get(..self.inner.len + 1)
            .and_then(|s| CStr::from_bytes_until_nul(s).ok())
    }
}

impl Write for NullTermMessage<'_> {
    fn write_str(&mut self, s: &str) -> Result {
        let len = s.floor_char_boundary(self.remaining());

        self.inner.buf[self.inner.len..self.inner.len + len].copy_from_slice(&s.as_bytes()[..len]);
        self.inner.len += len;
        // maintain null terminator but skip if slice is empty
        if let Some(b) = self.inner.buf.get_mut(self.inner.len) {
            *b = 0;
        }

        if len < s.len() {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}
