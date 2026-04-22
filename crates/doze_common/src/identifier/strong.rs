use core::{
    borrow::Borrow,
    ffi::CStr,
    fmt::{Display, Error, Formatter, Result},
    hash::{Hash, Hasher},
};

use alloc::{boxed::Box, ffi::CString, string::String};

use super::weak::WeakIdentifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrongIdentifier(Box<CStr>);

impl Hash for StrongIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bytes().hash(state);
    }
}

impl Borrow<[u8]> for StrongIdentifier {
    #[inline]
    fn borrow(&self) -> &[u8] {
        self.0.to_bytes()
    }
}

impl AsRef<CStr> for StrongIdentifier {
    #[inline]
    fn as_ref(&self) -> &CStr {
        &self.0
    }
}

impl PartialEq<str> for StrongIdentifier {
    fn eq(&self, other: &str) -> bool {
        let this_bytes = self.0.to_bytes();
        let other_bytes = other.as_bytes();
        if this_bytes.len() != other_bytes.len() {
            return false;
        }
        this_bytes.iter().zip(other_bytes).all(|(c, s)| c == s)
    }
}

impl From<WeakIdentifier<'_>> for StrongIdentifier {
    #[inline]
    fn from(identifier: WeakIdentifier<'_>) -> Self {
        let boxed: Box<[u8]> = Box::from(identifier.borrow());
        // SAFETY: Copied directly from a valid CStr, null terminator preserved.
        unsafe { StrongIdentifier(Box::from_raw(Box::into_raw(boxed) as *mut CStr)) }
    }
}

impl From<String> for StrongIdentifier {
    #[inline]
    fn from(value: String) -> Self {
        Self(
            CString::new(value)
                .expect("null byte in string")
                .into_boxed_c_str(),
        )
    }
}

impl From<&str> for StrongIdentifier {
    #[inline]
    fn from(value: &str) -> Self {
        Self(
            CString::new(value)
                .expect("null byte in string")
                .into_boxed_c_str(),
        )
    }
}

impl From<CString> for StrongIdentifier {
    #[inline]
    fn from(value: CString) -> Self {
        Self(value.into_boxed_c_str())
    }
}

impl Display for StrongIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(self.0.to_str().map_err(|_| Error)?)
    }
}
