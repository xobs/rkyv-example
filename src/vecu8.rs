use rkyv::{ArchiveUnsized, SerializeUnsized};

#[derive(Clone, Default)]
pub struct VecU8<T>
where
    T: heapless::ArrayLength<u8>,
{
    inner: heapless::Vec<u8, T>,
}

impl<T> VecU8<T>
where
    T: heapless::ArrayLength<u8>,
{
    pub fn new() -> Self {
        Default::default()
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }
}

impl<T> core::ops::Deref for VecU8<T>
where
    T: heapless::ArrayLength<u8>,
{
    type Target = heapless::Vec<u8, T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for VecU8<T>
where
    T: heapless::ArrayLength<u8>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ArchivedVecU8 {
    myvec_ptr: rkyv::RelPtr<[u8]>,
}

impl ArchivedVecU8 {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { &*self.myvec_ptr.as_ptr() }
    }
}

pub struct VecU8Resolver {
    bytes_pos: usize,
    _metadata_resolver: rkyv::MetadataResolver<[u8]>,
}

impl<S: rkyv::ser::Serializer + ?Sized, T: heapless::ArrayLength<u8>> rkyv::Serialize<S> for VecU8<T> {
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        println!("In Vecu8::Serialize::serialize()");
        Ok(VecU8Resolver {
            bytes_pos: self.as_slice().serialize_unsized(serializer)?,
            _metadata_resolver: self.as_slice().serialize_metadata(serializer)?,
        })
    }
}

impl<T> rkyv::Archive for VecU8<T>
where
    T: heapless::ArrayLength<u8>,
{
    type Archived = ArchivedVecU8;
    type Resolver = VecU8Resolver;
    fn resolve(&self, pos: usize, resolver: Self::Resolver) -> Self::Archived {
        println!("In Vecu8::Archive::resolve() (pos: {})", pos);
        Self::Archived {
            myvec_ptr: unsafe {
                self.as_slice().resolve_unsized(
                    pos + rkyv::offset_of!(Self::Archived, myvec_ptr),
                    resolver.bytes_pos,
                    (),
                )
            },
        }
    }
}

impl AsRef<[u8]> for ArchivedVecU8 {
    fn as_ref(&self) -> &[u8] {
        unsafe { &*self.myvec_ptr.as_ptr() }
    }
}
