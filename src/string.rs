use rkyv::{ArchiveUnsized, SerializeUnsized};

#[derive(Debug)]
pub enum Error {
    OutOfMemory,
}

#[derive(Copy, Clone)]
pub struct String<const N: usize> {
    bytes: [u8; N],
    len: u32, // length in bytes, not characters
}

impl<const N: usize> Default for String<N> {
    fn default() -> String<N> {
        String {
            bytes: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> String<N> {
    pub fn new() -> String<N> {
        String {
            bytes: [0; N],
            len: 0,
        }
    }

    pub fn from_str(src: &str) -> String<N> {
        let mut s = Self::new();
        // Copy the string into our backing store.
        for (&src_byte, dest_byte) in src.as_bytes().iter().zip(&mut s.bytes) {
            *dest_byte = src_byte;
        }
        // Set the string length to the length of the passed-in String,
        // or the maximum possible length. Which ever is smaller.
        s.len = s.bytes.len().min(src.as_bytes().len()) as u32;

        // If the string is not valid, set its length to 0.
        if s.as_str().is_err() {
            s.len = 0;
        }

        s
    }

    pub fn as_bytes(&self) -> [u8; N] {
        self.bytes
    }

    pub fn as_str(&self) -> core::result::Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.bytes[0..self.len as usize])
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear the contents of this String and set the length to 0
    pub fn clear(&mut self) {
        self.len = 0;
        self.bytes = [0; N];
    }

    pub fn to_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.bytes[0..self.len()]) }
    }

    /// awful, O(N) implementation because we have to iterate through the entire string
    /// and decode variable-length utf8 characters, until we can't.
    pub fn pop(&mut self) -> Option<char> {
        if self.is_empty() {
            return None;
        }
        // first, make a copy of the string
        let tempbytes: [u8; N] = self.bytes;
        let tempstr = unsafe { core::str::from_utf8_unchecked(&tempbytes[0..self.len()]) }.clone();
        // clear our own string
        self.len = 0;
        self.bytes = [0; N];

        // now copy over character by character, until just before the last character
        let mut char_iter = tempstr.chars();
        let mut maybe_c = char_iter.next();
        loop {
            match maybe_c {
                Some(c) => {
                    let next_c = char_iter.next();
                    match next_c {
                        Some(_thing) => {
                            self.push(c).unwrap(); // always succeeds because we're re-encoding our string
                            maybe_c = next_c;
                        }
                        None => return next_c,
                    }
                }
                None => {
                    return None; // we should actually never get here because len() == 0 case already covered
                }
            }
        }
    }

    pub fn push(&mut self, ch: char) -> core::result::Result<usize, Error> {
        match ch.len_utf8() {
            1 => {
                if self.len() < self.bytes.len() {
                    self.bytes[self.len()] = ch as u8;
                    self.len += 1;
                    Ok(1)
                } else {
                    Err(Error::OutOfMemory)
                }
            }
            _ => {
                let mut bytes: usize = 0;
                let mut data: [u8; 4] = [0; 4];
                let subslice = ch.encode_utf8(&mut data);
                if self.len() + subslice.len() < self.bytes.len() {
                    for c in subslice.bytes() {
                        self.bytes[self.len()] = c;
                        self.len += 1;
                        bytes += 1;
                    }
                    Ok(bytes)
                } else {
                    Err(Error::OutOfMemory)
                }
            }
        }
    }

    pub fn append(&mut self, s: &str) -> core::result::Result<usize, Error> {
        let mut bytes_added = 0;
        for ch in s.chars() {
            if let Ok(bytes) = self.push(ch) {
                bytes_added += bytes;
            } else {
                return Err(Error::OutOfMemory);
            }
        }
        Ok(bytes_added)
    }

    pub fn push_byte(&mut self, b: u8) -> core::result::Result<(), Error> {
        if self.len() < self.bytes.len() {
            self.bytes[self.len()] = b;
            self.len += 1;
            Ok(())
        } else {
            Err(Error::OutOfMemory)
        }
    }
}

impl<const N: usize> core::fmt::Display for String<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<const N: usize> core::fmt::Write for String<N> {
    fn write_str(&mut self, s: &str) -> core::result::Result<(), core::fmt::Error> {
        for c in s.bytes() {
            if self.len() < self.bytes.len() {
                self.bytes[self.len()] = c;
                self.len += 1;
            }
        }
        Ok(())
    }
}

impl<const N: usize> core::fmt::Debug for String<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

#[repr(C)]
pub struct ArchivedString {
    ptr: rkyv::RelPtr<str>,
}

impl ArchivedString {
    // Provide a `str` view of an `ArchivedString`.
    pub fn as_str(&self) -> &str {
        unsafe {
            // The as_ptr() function of RelPtr will get a pointer
            // to its memory.
            &*self.ptr.as_ptr()
        }
    }
}

pub struct StringResolver {
    bytes_pos: usize,
    _metadata_resolver: rkyv::MetadataResolver<str>,
}

/// Turn a `String` into an archived object
impl<const N: usize> rkyv::Archive for String<N> {
    type Archived = ArchivedString;
    type Resolver = StringResolver;

    fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
        println!("In String::Archive::resolve() (pos: {})", pos);
        Self::Archived {
            ptr: unsafe {
                self.as_str().unwrap().resolve_unsized(
                    pos + rkyv::offset_of!(Self::Archived, ptr),
                    resolver.bytes_pos,
                    (),
                )
            },
        }
    }
}

// Turn a stream of bytes into an `ArchivedString`.
impl<S: rkyv::ser::Serializer + ?Sized, const N: usize> rkyv::Serialize<S> for String<N> {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        println!("In String::Serialize::serialize()");
        // This is where we want to write the bytes of our string and return
        // a resolver that knows where those bytes were written.
        // We also need to serialize the metadata for our str.
        Ok(StringResolver {
            bytes_pos: self.as_str().unwrap().serialize_unsized(serializer)?,
            _metadata_resolver: self.as_str().unwrap().serialize_metadata(serializer)?,
        })
    }
}

impl<D: rkyv::de::Deserializer + ?Sized, const N: usize> rkyv::Deserialize<String<N>, D>
    for ArchivedString
{
    fn deserialize(&self, mut _deserializer: &mut D) -> Result<String<N>, D::Error> {
        Ok(String::from_str(self.as_str()))
    }
}
