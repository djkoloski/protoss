//! A stack-allocated container for an archived version of an evolving type.
use core::{
    fmt,
    mem::{self, MaybeUninit},
    ptr, marker::PhantomData,
};
use crate::{Evolving, VersionOf, Version};

/// An owned, stack-allocated container for some version of an [`Evolving`] type `E`.
/// 
/// It is backed by some `StorageV` which is a [`VersionOf<E>`], meaning it can store
/// any version of `E` with the same **major version** as `StorageV` and a **minor version**
/// less than or equal to `StorageV`.
pub struct Pylon<E: Evolving, StorageV: VersionOf<E> = <E as Evolving>::LatestVersion> {
    _phantom: PhantomData<E>,
    storage: MaybeUninit<StorageV>,
    contained_version: Version,
}

impl<E: Evolving, StorageV: VersionOf<E>> Drop for Pylon<E, StorageV> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: the inner value
            // - is valid for reads and writes
            // - is properly aligned
            // - points to a value valid for dropping
            // - will not be accessed after being dropped
            ptr::drop_in_place(self.probe_mut());
        }
    }
}

impl<E: Evolving, StorageV: VersionOf<E>> Pylon<E, StorageV> {
    /// Creates a new [`Pylon`] from a partially-initialized versioned value and its version.
    ///
    /// # Safety
    ///
    /// `stored_value` must have the fields defined by `contained_version` initialized.
    #[inline]
    pub unsafe fn new_unchecked(stored_value: MaybeUninit<StorageV>, contained_version: Version) -> Self {
        Self {
            _phantom: PhantomData,
            storage: stored_value,
            contained_version,
        }
    }

    /// Creates a new [`Pylon`] using the data of some version `V` of `E`.
    /// 
    /// In order for this to succeed, `V` must be from the same major version
    /// as `StorageV` and be a minor version less than or equal to `StorageV`.
    pub fn new<V: VersionOf<E>>(version_value: V) -> Result<Self, crate::Error> {
        let v_version = V::VERSION;
        let storage_version = StorageV::VERSION;
        if v_version.major != storage_version.major {
            return Err(crate::Error::CreatePylonWithUnmatchedMajorVersions)
        } else if v_version.minor > storage_version.minor {
            return Err(crate::Error::CreatePylonWithNewerMinorVersionThanStorage)
        }

        let mut storage = MaybeUninit::uninit();
        // TODO: safety comment
        unsafe {
            *(&mut storage as *mut MaybeUninit<StorageV>).cast::<V>() = version_value;
        }
        Ok(Self {
            _phantom: PhantomData,
            storage,
            contained_version: v_version,
        })
    }

    #[inline]
    fn probe(&self) -> &StorageV::ProbedBy {
        unsafe {
            // SAFETY:
            // - self.storage.as_ptr() is a valid pointer to a `StorageV::ProbedBy` because
            // it contains a vlue of the same major version
            // - E::probe_metadata returns valid metadata for a `ProbeOf<E>` of the correct
            // version, which `StorageV::ProbedBy` is.
            &*::ptr_meta::from_raw_parts(
                self.storage.as_ptr().cast(),
                E::probe_metadata(self.contained_version)
                    .expect("malformed Pylon created with version that does not exist"),
            )
        }
    }

    #[inline]
    fn probe_mut(&mut self) -> &mut StorageV::ProbedBy {
        unsafe {
            // SAFETY:
            // - self.storage.as_ptr() is a valid pointer to a `StorageV::ProbedBy` because
            // it contains a vlue of the same major version
            // - E::probe_metadata returns valid metadata for a `ProbeOf<E>` of the correct
            // version, which `StorageV::ProbedBy` is.
            &mut *::ptr_meta::from_raw_parts_mut(
                self.storage.as_mut_ptr().cast(),
                E::probe_metadata(self.contained_version)
                    .expect("malformed Pylon created with version that does not exist"),
            )
        }
    }

    /// Returns whether the data contained "completes" the storage, i.e.
    /// whether the contained version is a full (contains all fields of) `StorageV`.
    pub fn is_complete(&self) -> bool {
        self.contained_version == StorageV::VERSION
    }

    /// Unwraps the versioned type if the contained data is a `StorageV`
    ///
    /// If the data is not a `StorageV` version, `Err` is returned with the original value.
    pub fn try_unwrap(mut self) -> Result<StorageV, Self> {
        if self.is_complete() {
            let value = mem::replace(&mut self.storage, MaybeUninit::uninit());
            mem::forget(self);
            unsafe {
                Ok(value.assume_init())
            }
        } else {
            Err(self)
        }
    }

    /// Same as [`try_unwrap`][Pylon::try_unwrap] but panics if it fails.
    pub fn unwrap(self) -> StorageV
    where
        StorageV::ProbedBy: core::fmt::Debug
    {
        self.try_unwrap().expect("attempted to unwrap a Pylon that did not contain the StorageV version")
    }

    /// Converts the versioned type into a boxed [Probe][crate::ProbeOf] that is able to
    /// probe the contained data.
    pub fn into_boxed_probe(mut self) -> Box<StorageV::ProbedBy> {
        unsafe {
            #[cfg(feature = "std")]
            use ::std::alloc::alloc;
            #[cfg(not(feature = "std"))]
            use ::alloc::alloc::alloc;

            use ::core::{alloc::Layout, mem::{size_of_val, align_of_val, forget}};

            let probe = self.probe_mut();
            // SAFETY: align is a non-zero power of two which does not exceed usize::MAX when
            // rounded up to the nearest multiple of align
            let layout = Layout::from_size_align_unchecked(size_of_val(probe), align_of_val(probe));
            let ptr = if layout.size() == 0 {
                // SAFETY: layout.align() is non-zero
                ptr::NonNull::new_unchecked(layout.align() as *mut u8).as_ptr()
            } else {
                // SAFETY: layout has non-zero size
                let ptr = alloc(layout);
                // SAFETY:
                // - probe is valid for reads
                // - ptr is valid for writes
                ptr::copy_nonoverlapping(
                    probe as *const _ as *const u8,
                    ptr.cast::<u8>(),
                    layout.size(),
                );
                ptr
            };
            let probe_ptr = ::ptr_meta::from_raw_parts_mut(
                ptr.cast(),
                E::probe_metadata(self.contained_version)
                    .expect("malformed Pylon created with version that doesn't exist")
            );
            forget(self);
            // SAFETY: probe_ptr conforms to the memory layout required by Box
            Box::from_raw(probe_ptr)
        }
    }
}

impl<E: Evolving, StorageV: VersionOf<E>> fmt::Debug for Pylon<E, StorageV>
where
    StorageV::ProbedBy: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pylon")
            .field("value", &self.probe())
            .field("version", &self.contained_version)
            .finish()
    }
}
