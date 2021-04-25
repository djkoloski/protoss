use rkyv::{
    Archive,
    ArchivedMetadata,
    ArchivedUsize,
    ArchivePointee,
    ArchiveUnsized,
    DeserializeUnsized,
    Fallible,
    SerializeUnsized,
};

pub trait ArchiveParts: Partite {
    fn archived_partial_size(bytes: &[u8]) -> usize;
    fn unarchived_partial_size(archived_bytes: &[u8]) -> usize;
}

pub trait SerializeParts<S: Fallible>: ArchiveParts {
    fn serialize_parts(bytes: &[u8], serializer: &mut S) -> Result<usize, S::Error>;
}

pub trait DeserializeParts<T: ArchiveParts, D: Fallible>: Partite {
    fn deserialize_parts(bytes: &[u8], deserializer: &mut D) -> Result<*mut (), D::Error>;
}

impl<T: Partite> ArchivePointee for Partial<T> {
    type ArchivedMetadata = ArchivedUsize;

    fn pointer_metadata(archived: &Self::ArchivedMetadata) -> Self::Metadata {
        *archived as usize
    }
}

impl<T: Archive + ArchiveParts> ArchiveUnsized for Partial<T>
where
    T::Archived: Partite,
{
    type Archived = Partial<T::Archived>;
    type MetadataResolver = ();

    fn resolve_metadata(&self, _: usize, _: Self::MetadataResolver) -> ArchivedMetadata<Self> {
        T::archived_partial_size(&self.bytes) as ArchivedUsize
    }
}

impl<S: Fallible, T: Archive + SerializeParts<S>> SerializeUnsized<S> for Partial<T>
where
    T::Archived: Partite,
{
    fn serialize_unsized(&self, serializer: &mut S) -> Result<usize, S::Error> {
        T::serialize_parts(&self.bytes, serializer)
    }

    fn serialize_metadata(&self, _: &mut S) -> Result<Self::MetadataResolver, S::Error> {
        Ok(())
    }
}

impl<D: Fallible, T: Archive + ArchiveParts> DeserializeUnsized<Partial<T>, D> for Partial<T::Archived>
where
    T::Archived: DeserializeParts<T, D> + Partite,
{
    unsafe fn deserialize_unsized(&self, deserializer: &mut D) -> Result<*mut (), D::Error> {
        T::Archived::deserialize_parts(&self.bytes, deserializer)
    }

    fn deserialize_metadata(&self, _: &mut D) -> Result<<Partial<T> as Pointee>::Metadata, D::Error> {
        Ok(T::unarchived_partial_size(&self.bytes))
    }
}