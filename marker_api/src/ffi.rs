//! This module will hold types used for communication over FFI boundaries.
//!
//! There are some existing libraries and structures, like [`std::ffi`], which
//! provide similar functionality. These tend to focus on creating a stable
//! representation for calling C code, while we can expect both sides of the
//! FFI interface is written in Rust. These representations will therefore
//! focus on ABI safety, conversion, and simplicity by expecting that both
//! sides use these types.
//!
//! All of these types are naturally not a part of the stable API
#![allow(clippy::exhaustive_enums)]

use std::{marker::PhantomData, slice};

#[repr(C)]
#[derive(Clone, Copy, Eq)]
pub struct FfiStr<'a> {
    _lifetime: PhantomData<&'a ()>,
    /// Not really *const, but it should have the lifetime of at least `'a`
    data: *const u8,
    len: usize,
}

impl<'a> PartialEq for FfiStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other.get())
    }
}

impl<'a> From<&'a str> for FfiStr<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            _lifetime: PhantomData,
            data: source.as_ptr(),
            len: source.len(),
        }
    }
}

impl<'a> From<&'a String> for FfiStr<'a> {
    fn from(source: &'a String) -> Self {
        source.as_str().into()
    }
}

impl<'a> FfiStr<'a> {
    pub fn get(&self) -> &'a str {
        unsafe {
            let data = slice::from_raw_parts(self.data, self.len);
            std::str::from_utf8_unchecked(data)
        }
    }
}

impl<'a> From<&FfiStr<'a>> for &'a str {
    fn from(src: &FfiStr<'a>) -> Self {
        unsafe {
            let data = slice::from_raw_parts(src.data, src.len);

            std::str::from_utf8_unchecked(data)
        }
    }
}

impl<'a> ToString for FfiStr<'a> {
    fn to_string(&self) -> String {
        let base: &str = self.into();
        base.to_string()
    }
}

impl std::fmt::Debug for FfiStr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

impl std::hash::Hash for FfiStr<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

/// This is an FFI safe option. In most cases it's better to pass a pointer and
/// then use `as_ref()` but this doesn't work for owned return values.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub enum FfiOption<T> {
    Some(T),
    #[default]
    None,
}

impl<T> FfiOption<T> {
    pub fn get(&self) -> Option<&T> {
        match self {
            FfiOption::Some(x) => Some(x),
            FfiOption::None => None,
        }
    }

    pub fn copy(self) -> Option<T> {
        match self {
            FfiOption::Some(x) => Some(x),
            FfiOption::None => None,
        }
    }

    pub fn is_some(&self) -> bool {
        matches!(self, FfiOption::Some(_))
    }
}

impl<T> From<FfiOption<T>> for Option<T> {
    fn from(src: FfiOption<T>) -> Self {
        match src {
            FfiOption::Some(t) => Option::Some(t),
            FfiOption::None => Option::None,
        }
    }
}

impl<T> From<Option<T>> for FfiOption<T> {
    fn from(src: Option<T>) -> Self {
        match src {
            Option::Some(t) => FfiOption::Some(t),
            Option::None => FfiOption::None,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FfiSlice<'a, T> {
    _lifetime: PhantomData<&'a ()>,
    /// Not really *const, but it should have the lifetime of at least `'a`
    data: *const T,
    len: usize,
}

impl<'a, T: Eq> Eq for FfiSlice<'a, T> {}

impl<'a, T: PartialEq> PartialEq for FfiSlice<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other.get())
    }
}

impl<'a, T: std::hash::Hash> std::hash::Hash for FfiSlice<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<'a, T> FfiSlice<'a, T> {
    pub fn get(&self) -> &'a [T] {
        self.into()
    }

    pub fn as_slice(&self) -> &'a [T] {
        self.into()
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<'a, T> From<&'a [T]> for FfiSlice<'a, T> {
    fn from(src_data: &'a [T]) -> Self {
        Self {
            _lifetime: PhantomData,
            data: src_data.as_ptr(),
            len: src_data.len(),
        }
    }
}

impl<'a, T> From<&FfiSlice<'a, T>> for &'a [T] {
    fn from(src: &FfiSlice<'a, T>) -> Self {
        unsafe { slice::from_raw_parts(src.data, src.len) }
    }
}

impl<'a, T: std::fmt::Debug> std::fmt::Debug for FfiSlice<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data: &[T] = self.into();
        f.debug_list().entries(data.iter()).finish()
    }
}
