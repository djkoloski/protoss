use crate::{Composite, Partial};
use rkyv::{
    boxed::{ArchivedBox, BoxResolver},
    Archive,
    ArchiveUnsized,
    Deserialize,
    Fallible,
    MetadataResolver,
    Serialize,
    SerializeUnsized,
};

impl<T: Composite> Archive for Partial<T>
where
    T::Parts: ArchiveUnsized,
{
    type Archived = ArchivedBox<<T::Parts as ArchiveUnsized>::Archived>;
    type Resolver = BoxResolver<MetadataResolver<T::Parts>>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        ArchivedBox::resolve_from_ref(self.parts(), pos, resolver, out);
    }
}

impl<T: Composite, S: Fallible> Serialize<S> for Partial<T>
where
    T::Parts: SerializeUnsized<S>,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        ArchivedBox::serialize_from_ref(self.parts(), serializer)
    }
}

impl<T: Composite, D: Fallible> Deserialize<Partial<T>, D> for ArchivedBox<<T::Parts as ArchiveUnsized>::Archived>
where
    T::Parts: ArchiveUnsized,
    <T::Parts as ArchiveUnsized>::Archived: Deserialize<Partial<T>, D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<Partial<T>, D::Error> {
        self.as_ref().deserialize(deserializer)
    }
}
