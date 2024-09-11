//
// Copyright (c) 2023 Jacob R. Green
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use static_assertions as sa;
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::*;
use std::path::Path;

const INLINE_CHAR_COUNT: usize = size_of::<usize>() * 4 - 1;
const SENTINAL: u8 = 0xFF;

const INLINE_AGAIN_LENGTH: usize = INLINE_CHAR_COUNT / 2;

const unsafe fn uninitialized<T>() -> T {
    unsafe { MaybeUninit::uninit().assume_init() }
}

#[repr(packed)]
#[derive(Default, Copy, Clone)]
struct Inline {
    chars: [u8; INLINE_CHAR_COUNT],
    len: u8,
}

impl Inline {
    pub const fn new() -> Self {
        unsafe { std::mem::zeroed() }
    }

    pub const fn can_inline(s: &str) -> bool {
        s.len() <= INLINE_CHAR_COUNT
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.chars.as_ptr(),
                self.len as _,
            ))
        }
    }

    pub fn capacity(&self) -> usize {
        self.chars.len()
    }

    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    pub fn clear(&mut self) {
        self.len = 0
    }

    fn can_push_str(&self, string: &str) -> bool {
        self.len() + string.len() <= self.capacity()
    }

    fn push_str(&mut self, string: &str) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                string.as_ptr(),
                self.chars.as_mut_ptr().add(self.len()),
                string.len(),
            )
        }
        self.len += string.len() as u8;
    }

    pub fn try_push_str(&mut self, string: &str) -> Result<(), ()> {
        if self.can_push_str(string) {
            self.push_str(string);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl From<&str> for Inline {
    fn from(value: &str) -> Self {
        unsafe {
            let mut s: Inline = uninitialized();
            s.len = u8::try_from(value.len()).unwrap();
            std::ptr::copy_nonoverlapping(value.as_ptr(), s.chars.as_mut_ptr(), value.len());
            s
        }
    }
}

impl std::fmt::Debug for Inline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <str as std::fmt::Debug>::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for Inline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <str as std::fmt::Display>::fmt(self.as_str(), f)
    }
}

const PADDING: usize = size_of::<usize>() - 1;

#[derive(Clone)]
struct Heap {
    vec: String,
    #[allow(unused)]
    pad: [u8; PADDING],
    #[allow(unused)]
    flag: u8,
}

impl Heap {
    pub fn as_str(&self) -> &str {
        self.vec.as_str()
    }

    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn clear(&mut self) {
        self.vec.clear()
    }

    pub fn push_str(&mut self, string: &str) {
        self.vec.push_str(string);
    }
}

impl From<String> for Heap {
    fn from(value: String) -> Self {
        Self {
            vec: value,
            pad: unsafe { uninitialized() },
            flag: SENTINAL,
        }
    }
}

impl std::fmt::Debug for Heap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as std::fmt::Debug>::fmt(&self.vec, f)
    }
}

impl std::fmt::Display for Heap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <String as std::fmt::Display>::fmt(&self.vec, f)
    }
}

sa::assert_eq_size!(Inline, Heap);

pub union ShortString {
    inline: Inline,
    heap: ManuallyDrop<Heap>,
}

enum UnionVariant<'a> {
    Inline(&'a Inline),
    Heap(&'a Heap),
}

enum UnionVariantMut<'a> {
    Inline(&'a mut Inline),
    Heap(&'a mut Heap),
}

impl ShortString {
    pub const fn new() -> Self {
        Self {
            inline: Inline::new(),
        }
    }

    pub const fn is_inline(&self) -> bool {
        unsafe { self.inline.len != SENTINAL }
    }

    #[inline(always)]
    fn variant(&self) -> UnionVariant {
        unsafe {
            if self.is_inline() {
                UnionVariant::Inline(&self.inline)
            } else {
                UnionVariant::Heap(&self.heap)
            }
        }
    }

    #[inline(always)]
    fn variant_mut(&mut self) -> UnionVariantMut {
        unsafe {
            if self.is_inline() {
                UnionVariantMut::Inline(&mut self.inline)
            } else {
                UnionVariantMut::Heap(&mut self.heap)
            }
        }
    }

    pub fn as_str(&self) -> &str {
        match self.variant() {
            UnionVariant::Inline(inline) => inline.as_str(),
            UnionVariant::Heap(heap) => heap.as_str(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_str().as_bytes()
    }

    pub fn capacity(&self) -> usize {
        match self.variant() {
            UnionVariant::Inline(inline) => inline.capacity(),
            UnionVariant::Heap(heap) => heap.capacity(),
        }
    }

    pub fn len(&self) -> usize {
        match self.variant() {
            UnionVariant::Inline(inline) => inline.len(),
            UnionVariant::Heap(heap) => heap.len(),
        }
    }

    pub fn clear(&mut self) {
        match self.variant_mut() {
            UnionVariantMut::Inline(inline) => inline.clear(),
            UnionVariantMut::Heap(heap) => heap.clear(),
        }
    }

    pub fn push_str(&mut self, string: &str) {
        match self.variant_mut() {
            UnionVariantMut::Inline(inline) => {
                if let Err(_) = inline.try_push_str(string) {
                    let mut s = String::new();
                    s.reserve(inline.len() + string.len());
                    s.push_str(inline.as_str());
                    s.push_str(string);
                    self.heap = ManuallyDrop::new(s.into())
                }
            }
            UnionVariantMut::Heap(heap) => {
                heap.push_str(string);
            }
        }
    }

    pub fn push(&mut self, c: char) {
        let mut buffer: [u8; 4] = unsafe { uninitialized() };
        let string = c.encode_utf8(&mut buffer);
        self.push_str(string)
    }
}

impl Default for ShortString {
    fn default() -> Self {
        Self {
            inline: Default::default(),
        }
    }
}

impl Drop for ShortString {
    fn drop(&mut self) {
        unsafe {
            if !self.is_inline() {
                ManuallyDrop::drop(&mut self.heap)
            }
        }
    }
}

impl Clone for ShortString {
    fn clone(&self) -> Self {
        match self.variant() {
            UnionVariant::Inline(inline) => Self {
                inline: inline.clone(),
            },
            UnionVariant::Heap(heap) => Self {
                heap: ManuallyDrop::new(heap.clone()),
            },
        }
    }
}

impl From<&str> for ShortString {
    fn from(value: &str) -> Self {
        Inline::can_inline(value)
            .then(|| Self {
                inline: Inline::from(value),
            })
            .unwrap_or_else(|| Self {
                heap: ManuallyDrop::new(Heap::from(value.to_owned())),
            })
    }
}

impl From<String> for ShortString {
    fn from(value: String) -> Self {
        Inline::can_inline(&value)
            .then(|| Self {
                inline: Inline::from(value.as_str()),
            })
            .unwrap_or_else(|| Self {
                heap: ManuallyDrop::new(Heap::from(value)),
            })
    }
}

impl Deref for ShortString {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<OsStr> for ShortString {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(self.as_str())
    }
}

impl AsRef<Path> for ShortString {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl AsRef<[u8]> for ShortString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Borrow<str> for ShortString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Add<&str> for ShortString {
    type Output = Self;
    fn add(mut self, rhs: &str) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<&str> for ShortString {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl Extend<char> for ShortString {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        iter.into_iter().for_each(|c| self.push(c));
    }
}

impl<'a> Extend<&'a str> for ShortString {
    fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s));
    }
}

impl std::fmt::Debug for ShortString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.variant() {
            UnionVariant::Inline(inline) => <Inline as std::fmt::Debug>::fmt(inline, f),
            UnionVariant::Heap(heap) => <Heap as std::fmt::Debug>::fmt(heap, f),
        }
    }
}

impl std::fmt::Display for ShortString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.variant() {
            UnionVariant::Inline(inline) => <Inline as std::fmt::Display>::fmt(inline, f),
            UnionVariant::Heap(heap) => <Heap as std::fmt::Display>::fmt(heap, f),
        }
    }
}
