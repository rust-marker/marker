//! This module will hold types used for communication over FFI boundaries.
//!
//! There are some existing libraries and structures, like [`std::ffi`], which
//! provide similar functionality. These tend to focus on creating a stable
//! representation for calling C code, while we can expect both sides of the
//! FFI interface is written in Rust. These representations will therefore
//! focus on ABI safety, conversion, and simplicity by expecting that both
//! sides use these types.
//!
//! All of these types are naturally not part of the stable API

use std::{marker::PhantomData, slice};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Str<'a> {
    _lifetime: PhantomData<&'a ()>,
    data: *const u8,
    len: usize,
}

impl<'a> From<&'a str> for Str<'a> {
    fn from(source: &'a str) -> Self {
        Self {
            _lifetime: PhantomData,
            data: source.as_ptr(),
            len: source.len(),
        }
    }
}

impl<'a> From<&Str<'a>> for &'a str {
    fn from(src: &Str<'a>) -> Self {
        unsafe {
            let data = slice::from_raw_parts(src.data, src.len);

            std::str::from_utf8_unchecked(data)
        }
    }
}

impl<'a> ToString for Str<'a> {
    fn to_string(&self) -> String {
        let base: &str = self.into();
        base.to_string()
    }
}

/// This is an FFI save option. In most cases it's better to pass a pointer and
/// then use `as_ref()` but this doesn't work for owned return values.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FfiOption<T> {
    Some(T),
    None,
}

impl<T> FfiOption<T> {
    pub fn get(&self) -> Option<&T> {
        match self {
            FfiOption::Some(x) => Some(x),
            FfiOption::None => None,
        }
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
pub struct FfiSlice<'a, T> {
    _lifetime: PhantomData<&'a ()>,
    data: *const T,
    len: usize,
}

impl<'a, T> FfiSlice<'a, T> {
    pub fn get(&self) -> &'a [T] {
        self.into()
    }

    pub fn as_slice(&self) -> &'a [T] {
        self.into()
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
