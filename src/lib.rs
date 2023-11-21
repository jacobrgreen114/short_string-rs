//
// Copyright (c) 2023. Jacob R. Green
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

use std::ops::Add;

// 32-byte ShortString
#[cfg(not(feature = "compact"))]
pub const INLINE_LENGTH: usize = 30;

// 24-byte ShortString
#[cfg(feature = "compact")]
pub const INLINE_LENGTH: usize = 15;

#[repr(packed)]
#[derive(Default, Clone, Copy)]
struct InlineString {
    len: u8,
    buf: [u8; INLINE_LENGTH],
}

#[derive(Clone)]
enum ShortStringInner {
    Inline(InlineString),
    Heap(String),
}

impl Default for ShortStringInner {
    fn default() -> Self {
        Self::Inline(InlineString::default())
    }
}

#[derive(Default, Clone)]
pub struct ShortString {
    inner: ShortStringInner,
}

impl ShortString {
    pub fn new() -> Self {
        Self {
            inner: ShortStringInner::Inline(InlineString {
                buf: [0; INLINE_LENGTH],
                len: 0,
            }),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match &self.inner {
            ShortStringInner::Inline(inlined) => &inlined.buf[..(inlined.len as usize)],
            ShortStringInner::Heap(buf) => buf.as_bytes(),
        }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        match &mut self.inner {
            ShortStringInner::Inline(inlined) => unsafe {
                std::str::from_utf8_unchecked_mut(&mut inlined.buf[..(inlined.len as usize)])
            },
            ShortStringInner::Heap(buf) => unsafe { buf.as_mut_str() },
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.inner {
            ShortStringInner::Inline(inlined) => unsafe {
                std::str::from_utf8_unchecked(&inlined.buf[..(inlined.len as usize)])
            },
            ShortStringInner::Heap(buf) => unsafe { buf.as_str() },
        }
    }

    pub fn capacity(&self) -> usize {
        match &self.inner {
            ShortStringInner::Inline(inlined) => inlined.buf.len(),
            ShortStringInner::Heap(buf) => buf.capacity(),
        }
    }

    pub fn clear(&mut self) {
        match &mut self.inner {
            ShortStringInner::Inline(inlined) => {
                inlined.len = 0;
            }
            ShortStringInner::Heap(buf) => {
                buf.clear();
            }
        }
    }

    pub fn into_string(self) -> String {
        match self.inner {
            ShortStringInner::Inline(inlined) => unsafe {
                String::from_utf8_unchecked(Vec::from(&inlined.buf[..(inlined.len as usize)]))
            },
            ShortStringInner::Heap(buf) => buf,
        }
    }

    pub fn push(&mut self, ch: char) {
        match &mut self.inner {
            ShortStringInner::Inline(inlined) => {
                let mut bytes = [0u8; 4];
                let ch_bytes = ch.encode_utf8(&mut bytes).as_bytes();

                let length = inlined.len as usize;

                if length + ch_bytes.len() > inlined.buf.len() {
                    self.inner = ShortStringInner::Heap(unsafe {
                        String::from_utf8_unchecked(Vec::from(&inlined.buf[..(length as usize)]))
                    });
                    self.push(ch);
                } else {
                    inlined.buf[length..length + ch_bytes.len()].copy_from_slice(ch_bytes);
                    inlined.len += ch_bytes.len() as u8;
                }
            }
            ShortStringInner::Heap(string) => string.push(ch),
        }
    }
}

impl Add<&str> for ShortString {
    type Output = Self;

    fn add(mut self, rhs: &str) -> Self::Output {
        self.push_str(rhs);
        self
    }
}

impl std::fmt::Debug for ShortString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Display for ShortString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for ShortString {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ShortString {}

impl PartialOrd for ShortString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for ShortString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl std::hash::Hash for ShortString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl std::ops::Deref for ShortString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<&str> for ShortString {
    fn from(s: &str) -> Self {
        let len = s.len();
        if len <= INLINE_LENGTH {
            let mut buf = [0; INLINE_LENGTH];
            buf[..len].copy_from_slice(s.as_bytes());
            Self {
                inner: ShortStringInner::Inline(InlineString {
                    buf,
                    len: len as u8,
                }),
            }
        } else {
            Self {
                inner: ShortStringInner::Heap(s.into()),
            }
        }
    }
}

impl From<String> for ShortString {
    fn from(s: String) -> Self {
        Self {
            inner: ShortStringInner::Heap(s),
        }
    }
}

impl From<&String> for ShortString {
    fn from(s: &String) -> Self {
        Self::from(s.as_str())
    }
}

impl Into<String> for ShortString {
    fn into(self) -> String {
        self.into_string()
    }
}

impl AsRef<str> for ShortString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
