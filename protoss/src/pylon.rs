use core::{
    fmt,
    mem::{self, MaybeUninit},
    ptr,
};
use crate::{Evolving, VersionOf};

/// Some version of a versioned type.
pub struct Pylon<E: Evolving> {
    value: MaybeUninit<E::Latest>,
    version: u16,
}

impl<E: Evolving> Drop for Pylon<E> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: the inner value
            // - is valid for reads and writes
            // - is properly aligned
            // - points to a value valid for dropping
            // - will not be accessed after being dropped
            ptr::drop_in_place(self.access_mut());
        }
    }
}

impl<E: Evolving> Pylon<E> {
    /// Creates a new proto from a partially-initialized versioned value and its version.
    ///
    /// # Safety
    ///
    /// `value` must have the fields specified by `version` initialized.
    #[inline]
    pub unsafe fn new_unchecked(value: MaybeUninit<E::Latest>, version: u16) -> Self {
        Self {
            value,
            version,
        }
    }

    /// Creates a new [`Pylon`] from some version `V` of `E` less than or equal to the latest.
    pub fn new<V: VersionOf<E>>(version: V) -> Self {
        let mut value = MaybeUninit::uninit();
        // TODO: safety comment
        unsafe {
            *(&mut value as *mut MaybeUninit<E::Latest>).cast::<V>() = version;
        }
        Self {
            value,
            version: V::VERSION,
        }
    }

    #[inline]
    fn access(&self) -> &E::Probe {
        unsafe {
            // SAFETY:
            // - self.value.as_ptr() is a valid pointer to T::Accesor
            // - T::probe_metadata returns valid metadata for a T::Probe
            &*::ptr_meta::from_raw_parts(
                self.value.as_ptr().cast(),
                E::probe_metadata(self.version),
            )
        }
    }

    #[inline]
    fn access_mut(&mut self) -> &mut E::Probe {
        unsafe {
            // SAFETY:
            // - self.value.as_ptr() is a valid pointer to T::Accesor
            // - T::probe_metadata returns valid metadata for a T::Probe
            &mut *::ptr_meta::from_raw_parts_mut(
                self.value.as_mut_ptr().cast(),
                E::probe_metadata(self.version),
            )
        }
    }

    /// Returns whether the data is the latest version.
    pub fn is_latest(&self) -> bool {
        self.version == E::Latest::VERSION
    }

    /// Unwraps the versioned type if the data is the latest version.
    ///
    /// If the data is not the latest version, `Err` is returned with the original value.
    pub fn try_unwrap(mut self) -> Result<E::Latest, Self> {
        if self.is_latest() {
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            mem::forget(self);
            unsafe {
                Ok(value.assume_init())
            }
        } else {
            Err(self)
        }
    }

    /// Unwraps the versioned type and panics if the data is not the latest version.
    pub fn unwrap(self) -> E::Latest
    where
        E::Probe: core::fmt::Debug
    {
        self.try_unwrap().expect("attempted to unwrap a Version that was not the latest version")
    }

    /// Converts the versioned type into a boxed probe.
    pub fn into_boxed_probe(mut self) -> Box<E::Probe> {
        unsafe {
            #[cfg(feature = "std")]
            use ::std::alloc::alloc;
            #[cfg(not(feature = "std"))]
            use ::alloc::alloc::alloc;

            use ::core::{alloc::Layout, mem::{size_of_val, align_of_val, forget}};

            let probe = self.access_mut();
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
            let probe_ptr = ::ptr_meta::from_raw_parts_mut(ptr.cast(), E::probe_metadata(self.version));
            forget(self);
            // SAFETY: probe_ptr conforms to the memory layout required by Box
            Box::from_raw(probe_ptr)
        }
    }
}

impl<E: Evolving> fmt::Debug for Pylon<E>
where
    E::Probe: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pylon")
            .field("value", &self.access())
            .field("version", &self.version)
            .finish()
    }
}
