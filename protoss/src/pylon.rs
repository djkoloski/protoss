//! A stack-allocated container for an archived version of an evolving type.
use core::{
    fmt,
    mem::{self, MaybeUninit},
    ptr, marker::PhantomData,
};
use crate::{Evolving, Version, Evolution};

/// An owned, stack-allocated container for an archived version of an [`Evolving`] type `E`.
/// 
/// It is backed by the `Archived` type of some `StorageEV` which is a [`Evolution<Base = E>`], meaning it can store
/// any version of `E` with the same **major version** as `StorageEV` and a **minor version**
/// less than or equal to `StorageEV`.
/// 
/// Note that this type is only rarely useful and it requires there's a trivial way to construct the [`Archived`][::rkyv::Archive::Archived]
/// type of your evolving type, which may not always be the case.
pub struct Pylon<E: Evolving, StorageEV: Evolution<Base = E> = <E as Evolving>::LatestEvolution> {
    _phantom: PhantomData<E>,
    storage: MaybeUninit<StorageEV::Archived>,
    contained_version: Version,
}

impl<E: Evolving, StorageEV: Evolution<Base = E>> Drop for Pylon<E, StorageEV> {
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

impl<E: Evolving, StorageEV: Evolution<Base = E>> Pylon<E, StorageEV> {
    /// Creates a new [`Pylon`] from a partially-initialized versioned value and its version.
    ///
    /// # Safety
    ///
    /// `stored_value` must have the fields defined by `contained_version` initialized.
    #[inline]
    pub unsafe fn new_unchecked(stored_value: MaybeUninit<StorageEV::Archived>, contained_version: Version) -> Self {
        Self {
            _phantom: PhantomData,
            storage: stored_value,
            contained_version,
        }
    }

    /// Creates a new [`Pylon`] using the data of some version `V` of `E`.
    /// 
    /// In order for this to succeed, `V` must be from the same major version
    /// as `StorageEV` and be a minor version less than or equal to `StorageV`.
    pub fn new<EV: Evolution<Base = E>>(version_value: EV::Archived) -> Result<Self, crate::Error> {
        if EV::VERSION.0 > StorageEV::VERSION.0 {
            return Err(crate::Error::CreatePylonWithNewerMinorVersionThanStorage)
        }

        let mut storage = MaybeUninit::uninit();
        // TODO: safety comment
        unsafe {
            *(&mut storage as *mut MaybeUninit<StorageEV::Archived>).cast::<EV::Archived>() = version_value;
        }
        Ok(Self {
            _phantom: PhantomData,
            storage,
            contained_version: EV::VERSION,
        })
    }

    #[inline]
    fn probe(&self) -> &E::Probe {
        unsafe {
            // SAFETY:
            // - self.storage.as_ptr() is a valid pointer to a `StorageEV::ProbedBy` because
            // it contains a vlue of the same major version
            // - E::probe_metadata returns valid metadata for a `ProbeOf<E>` of the correct
            // version, which `StorageEV::ProbedBy` is.
            &*::ptr_meta::from_raw_parts(
                self.storage.as_ptr().cast(),
                E::probe_metadata(self.contained_version)
                    .expect("malformed Pylon created with version that does not exist"),
            )
        }
    }

    #[inline]
    fn probe_mut(&mut self) -> &mut E::Probe {
        unsafe {
            // SAFETY:
            // - self.storage.as_ptr() is a valid pointer to a `StorageEV::ProbedBy` because
            // it contains a vlue of the same major version
            // - E::probe_metadata returns valid metadata for a `ProbeOf<E>` of the correct
            // version, which `StorageEV::ProbedBy` is.
            &mut *::ptr_meta::from_raw_parts_mut(
                self.storage.as_mut_ptr().cast(),
                E::probe_metadata(self.contained_version)
                    .expect("malformed Pylon created with version that does not exist"),
            )
        }
    }

    /// Returns whether the data contained "completes" the storage, i.e.
    /// whether the contained version is a full (contains all fields of) `StorageEV`.
    pub fn is_complete(&self) -> bool {
        self.contained_version == StorageEV::VERSION
    }

    /// Unwraps the versioned type if the contained data is a `StorageEV`
    ///
    /// If the data is not a `StorageEV` version, `Err` is returned with the original value.
    pub fn try_unwrap(mut self) -> Result<StorageEV::Archived, Self> {
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
    pub fn unwrap(self) -> StorageEV::Archived
    where
        E::Probe: core::fmt::Debug
    {
        self.try_unwrap().expect("attempted to unwrap a Pylon that did not contain the StorageEV version")
    }

    /// Converts the versioned type into a boxed [Probe][crate::ProbeOf] that is able to
    /// probe the contained data.
    pub fn into_boxed_probe(mut self) -> Box<E::Probe> {
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

impl<E: Evolving, StorageEV: Evolution<Base = E>> fmt::Debug for Pylon<E, StorageEV>
where
    E::Probe: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pylon")
            .field("value", &self.probe())
            .field("version", &self.contained_version)
            .finish()
    }
}
