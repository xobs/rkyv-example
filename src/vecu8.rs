#[derive(Clone, Default)]
pub struct VecU8<T> where T: heapless::ArrayLength<u8> {
    inner: heapless::Vec<u8, T>,
}

impl<T> VecU8<T>  where T: heapless::ArrayLength<u8> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }
}

impl<T> core::ops::Deref for VecU8<T>  where T: heapless::ArrayLength<u8> {
    type Target = heapless::Vec<u8, T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for VecU8<T> where T: heapless::ArrayLength<u8> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct ArchivedVecU8 {
    myvec_ptr: rkyv::RelPtr,
    myvec_len: u32,
}

impl ArchivedVecU8 {
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.myvec_ptr.as_ptr(), self.myvec_len as usize)}
    }
}

pub struct VecU8Resolver {
    start: usize,
}

impl<T> rkyv::Resolve<VecU8<T>> for VecU8Resolver where T: heapless::ArrayLength<u8> {
    type Archived = ArchivedVecU8;

    fn resolve(self, pos: usize, value: &VecU8<T>) -> Self::Archived {
        println!("In myvec_resolve(pos: {})", pos);
        Self::Archived {
            myvec_ptr: unsafe {
                rkyv::RelPtr::new(pos + rkyv::offset_of!(ArchivedVecU8, myvec_ptr), self.start)
            },
            myvec_len: value.inner.len() as u32,
        }
    }
}

impl<T> rkyv::Archive for VecU8<T> where T: heapless::ArrayLength<u8> {
    type Archived = ArchivedVecU8;
    type Resolver = VecU8Resolver;
    fn archive<W: rkyv::Write + ?Sized>(
        &self,
        writer: &mut W,
    ) -> core::result::Result<Self::Resolver, W::Error> {
        println!("In myvec_archive() (pos: {})", writer.pos());
        let start = writer.pos();
        writer.write(&self.inner[0..self.inner.len()])?;
        Ok(Self::Resolver { start })
    }
}

impl AsRef<[u8]> for ArchivedVecU8 {
    fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.myvec_ptr.as_ptr(), self.myvec_len as usize)}
    }
}
