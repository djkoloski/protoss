#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use core::{
    fmt,
    mem::{self, MaybeUninit},
    ptr,
};
use ptr_meta::Pointee;

pub use protoss_derive::protoss;

/// A type that can be treated as a collection of its fields.
pub unsafe trait Composite: Sized {
    /// The type used to access the individual fields.
    ///
    /// The metadata for this type **must be** the total number of bytes that make up the type.
    type Parts: Pointee<Metadata = usize> + ?Sized;
}

/// Some or all of the parts of a composite type.
pub struct Partial<T: Composite> {
    value: MaybeUninit<T>,
    size: usize,
}

impl<T: Composite> Drop for Partial<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.parts_mut());
        }
    }
}

impl<T: Composite> Partial<T> {
    /// Creates an empty partial with no set fields.
    pub fn empty() -> Self {
        Self {
            value: MaybeUninit::uninit(),
            size: 0,
        }
    }

    /// Creates a new partial from a composite value.
    pub fn new(value: T) -> Self {
        Self {
            value: MaybeUninit::new(value),
            size: mem::size_of::<T>(),
        }
    }

    /// Creates a new partial from a partially-initialized composite value and size.
    ///
    /// # Safety
    ///
    /// `value` must have the parts specified by `size` initialized.
    pub unsafe fn new_unchecked(value: MaybeUninit<T>, size: usize) -> Self {
        Self {
            value,
            size,
        }
    }

    /// Returns the parts of the partial.
    pub fn parts(&self) -> &T::Parts {
        unsafe { &*ptr_meta::from_raw_parts(self.value.as_ptr().cast(), self.size) }
    }

    /// Returns the mutable parts of the partial.
    pub fn parts_mut(&mut self) -> &mut T::Parts {
        unsafe { &mut *ptr_meta::from_raw_parts_mut(self.value.as_mut_ptr().cast(), self.size) }
    }

    /// Returns whether the partial has all of its fields.
    pub fn is_complete(&self) -> bool {
        self.size == mem::size_of::<T>()
    }

    /// Unwraps the composite type if the partial is complete.
    ///
    /// If the partial is incomplete, `Err` is returned with the original partial.
    pub fn try_unwrap(mut self) -> Result<T, Self> {
        if self.is_complete() {
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            mem::forget(self);
            unsafe {
                Ok(value.assume_init())
            }
        } else {
            Err(self)
        }
    }

    /// Unwraps the composite type and panics if the partial is incomplete.
    pub fn unwrap(self) -> T {
        self.try_unwrap().expect("attempted to unwrap an incomplete Partial")
    }

    /// Converts the partial into a box of its parts.
    pub fn into_boxed_parts(self) -> Box<T::Parts> {
        #[cfg(feature = "std")]
        use std::alloc;
        #[cfg(not(feature = "std"))]
        use alloc::alloc;

        unsafe {
            // Move data into a new pointer
            let data = if self.size != 0 {
                let layout = alloc::Layout::from_size_align_unchecked(self.size, mem::align_of::<T>());
                let data = alloc::alloc(layout);
                ptr::copy_nonoverlapping(self.value.as_ptr().cast(), data, self.size);
                data
            } else {
                ptr::NonNull::<T>::dangling().as_ptr().cast()
            };

            // Make parts from data
            let ptr = ptr_meta::from_raw_parts_mut(data.cast(), self.size);

            // Forget the partial
            mem::forget(self);

            // Wrap the parts in a box
            Box::from_raw(ptr)
        }
    }

    /// Converts a box of parts back into a partial.
    ///
    /// # Safety
    ///
    /// The provided parts must not exceed the size of the composite type. This could happen if, for
    /// example, the parts were from a newer schema for the composite type.
    pub unsafe fn from_boxed_parts_unchecked(parts: Box<T::Parts>) -> Self {
        #[cfg(feature = "std")]
        use std::alloc;
        #[cfg(not(feature = "std"))]
        use alloc::alloc;

        // Unwrap the parts and decompose the pointer
        let ptr = Box::into_raw(parts);
        let (data, size) = ptr_meta::PtrExt::to_raw_parts(ptr);

        // Move the parts into a MaybeUninit
        let mut value = MaybeUninit::<T>::uninit();
        if size != 0 {
            let layout = alloc::Layout::from_size_align_unchecked(size, mem::align_of::<T>());
            ptr::copy_nonoverlapping(data.cast::<u8>(), value.as_mut_ptr().cast::<u8>(), size);
            alloc::dealloc(data.cast(), layout);
        }

        Self {
            value,
            size,
        }
    }

    /// Attempts to convert a box of parts back into a partial.
    ///
    /// If the parts exceed the maximum size of the partial, `Err` is returned with the original
    /// parts. THis could happen if, for example, the parts were from a newer schema for the
    /// composite type.
    pub fn from_boxed_parts(parts: Box<T::Parts>) -> Result<Self, Box<T::Parts>> {
        #[cfg(feature = "std")]
        use std::alloc;
        #[cfg(not(feature = "std"))]
        use alloc::alloc;

        unsafe {
            // Unwrap the parts and decompose the pointer
            let ptr = Box::into_raw(parts);
            let (data, size) = ptr_meta::PtrExt::to_raw_parts(ptr);

            if size > mem::size_of::<T>() {
                // Put it back in a box
                let ptr = ptr_meta::from_raw_parts_mut(data, size);
                Err(Box::from_raw(ptr))
            } else {
                // Move the parts into a MaybeUninit
                let mut value = MaybeUninit::<T>::uninit();
                if size != 0 {
                    let layout = alloc::Layout::from_size_align_unchecked(size, mem::align_of::<T>());
                    ptr::copy_nonoverlapping(data.cast::<u8>(), value.as_mut_ptr().cast::<u8>(), size);
                    alloc::dealloc(data.cast(), layout);
                }

                Ok(Self {
                    value,
                    size,
                })
            }
        }
    }
}

impl<T: Composite> AsRef<T::Parts> for Partial<T> {
    fn as_ref(&self) -> &T::Parts {
        self.parts()
    }
}

impl<T: Composite> AsMut<T::Parts> for Partial<T> {
    fn as_mut(&mut self) -> &mut T::Parts {
        self.parts_mut()
    }
}

impl<T: Composite> fmt::Debug for Partial<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Partial")
            .field("value", &self.value)
            .field("size", &self.size)
            .finish()
    }
}

#[cfg(feature = "rkyv")]
const _: () = {
    use ::rkyv::{
        boxed::{ArchivedBox, BoxResolver},
        Archive,
        ArchiveUnsized,
        Deserialize,
        DeserializeUnsized,
        Fallible,
        Serialize,
        SerializeUnsized,
    };

    impl<T: Composite> Archive for Partial<T>
    where
        T::Parts: ArchiveUnsized,
    {
        type Archived = ArchivedBox<<T::Parts as ArchiveUnsized>::Archived>;
        type Resolver = BoxResolver<<T::Parts as ArchiveUnsized>::MetadataResolver>;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            ArchivedBox::resolve_from_ref(self.parts(), pos, resolver, out);
        }
    }

    impl<T: Composite, S: Fallible + ?Sized> Serialize<S> for Partial<T>
    where
        T::Parts: SerializeUnsized<S>,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            ArchivedBox::serialize_from_ref(self.parts(), serializer)
        }
    }

    impl<T: Composite, D: Fallible + ?Sized> Deserialize<Partial<T>, D> for ArchivedBox<<T::Parts as ArchiveUnsized>::Archived>
    where
        T::Parts: ArchiveUnsized,
        <T::Parts as ArchiveUnsized>::Archived: DeserializeUnsized<T::Parts, D>,
    {
        fn deserialize(&self, deserializer: &mut D) -> Result<Partial<T>, D::Error> {
            unsafe {
                let mut value = ::core::mem::MaybeUninit::<T>::uninit();
                let mut size = 0;
                self.as_ref().deserialize_unsized(deserializer, |layout| {
                    size = layout.size();
                    value.as_mut_ptr().cast()
                })?;

                Ok(Partial::new_unchecked(value, size))
            }
        }
    }
};
