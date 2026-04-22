use core::{
    borrow::Borrow,
    ffi::CStr,
    fmt::{Display, Error, Formatter, Result},
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeakIdentifier<'i>(&'i CStr);

impl Hash for WeakIdentifier<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.borrow());
    }
}

impl Borrow<[u8]> for WeakIdentifier<'_> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.0.to_bytes()
    }
}

impl Borrow<str> for WeakIdentifier<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.0.to_str().expect("invalid UTF-8")
    }
}

impl AsRef<CStr> for WeakIdentifier<'_> {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.0
    }
}

impl PartialEq<str> for WeakIdentifier<'_> {
    fn eq(&self, other: &str) -> bool {
        let this_bytes = self.0.to_bytes();
        let other_bytes = other.as_bytes();
        if this_bytes.len() != other_bytes.len() {
            return false;
        }
        this_bytes.iter().zip(other_bytes).all(|(c, s)| c == s)
    }
}

impl<'i> From<&'i CStr> for WeakIdentifier<'i> {
    #[inline]
    fn from(value: &'i CStr) -> Self {
        Self::from_cstr(value)
    }
}

impl Display for WeakIdentifier<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.0.to_str().map_err(|_| Error)?)
    }
}

impl<'i> WeakIdentifier<'i> {
    #[inline]
    pub const fn as_cstr(self) -> &'i CStr {
        self.0
    }

    #[inline]
    pub const fn from_cstr(value: &'i CStr) -> Self {
        Self(value)
    }
}

#[cfg(feature = "alloc")]
impl super::strong::StrongIdentifier {
    #[inline]
    pub fn downgrade<'i>(&'i self) -> WeakIdentifier<'i> {
        WeakIdentifier(self.as_ref())
    }
}
