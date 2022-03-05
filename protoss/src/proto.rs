use core::{
    fmt,
    mem::{self, MaybeUninit},
    ptr,
};
use crate::Versioned;

/// Some version of a versioned type.
pub struct Proto<T: Versioned> {
    value: MaybeUninit<T>,
    version: T::Version,
}

impl<T: Versioned> Drop for Proto<T> {
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

impl<T: Versioned> Proto<T> {
    /// Creates a new proto from a partially-initialized versioned value and its version.
    ///
    /// # Safety
    ///
    /// `value` must have the fields specified by `version` initialized.
    #[inline]
    pub unsafe fn new_unchecked(value: MaybeUninit<T>, version: T::Version) -> Self {
        Self {
            value,
            version,
        }
    }

    /// Creates a new proto from a value of the latest version.
    #[inline]
    pub fn latest(value: T) -> Self {
        Self {
            value: MaybeUninit::new(value),
            version: T::LATEST,
        }
    }

    #[inline]
    fn access(&self) -> &T::Accessor {
        unsafe {
            // SAFETY:
            // - self.value.as_ptr() is a valid pointer to T::Accesor
            // - T::accessor_metadata returns valid metadata for a T::Accessor
            &*::ptr_meta::from_raw_parts(
                self.value.as_ptr().cast(),
                T::accessor_metadata(self.version),
            )
        }
    }

    #[inline]
    fn access_mut(&mut self) -> &mut T::Accessor {
        unsafe {
            // SAFETY:
            // - self.value.as_ptr() is a valid pointer to T::Accesor
            // - T::accessor_metadata returns valid metadata for a T::Accessor
            &mut *::ptr_meta::from_raw_parts_mut(
                self.value.as_mut_ptr().cast(),
                T::accessor_metadata(self.version),
            )
        }
    }

    /// Returns whether the data is the latest version.
    pub fn is_latest(&self) -> bool {
        self.version == T::LATEST
    }

    /// Unwraps the versioned type if the data is the latest version.
    ///
    /// If the data is not the latest version, `Err` is returned with the original value.
    pub fn try_unwrap(mut self) -> Result<T, Self> {
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
    pub fn unwrap(self) -> T
    where
        T::Accessor: fmt::Debug,
        T::Version: fmt::Debug,
    {
        self.try_unwrap().expect("attempted to unwrap a Version that was not the latest version")
    }

    /// Converts the versioned type into a boxed accessor.
    pub fn into_boxed_accessor(mut self) -> Box<T::Accessor> {
        unsafe {
            #[cfg(feature = "std")]
            use ::std::alloc::alloc;
            #[cfg(not(feature = "std"))]
            use ::alloc::alloc::alloc;

            use ::core::{alloc::Layout, mem::{size_of_val, align_of_val, forget}};

            let accessor = self.access_mut();
            // SAFETY: align is a non-zero power of two which does not exceed usize::MAX when
            // rounded up to the nearest multiple of align
            let layout = Layout::from_size_align_unchecked(size_of_val(accessor), align_of_val(accessor));
            let ptr = if layout.size() == 0 {
                // SAFETY: layout.align() is non-zero
                ptr::NonNull::new_unchecked(layout.align() as *mut u8).as_ptr()
            } else {
                // SAFETY: layout has non-zero size
                let ptr = alloc(layout);
                // SAFETY:
                // - accessor is valid for reads
                // - ptr is valid for writes
                ptr::copy_nonoverlapping(
                    accessor as *const _ as *const u8,
                    ptr.cast::<u8>(),
                    layout.size(),
                );
                ptr
            };
            let accessor_ptr = ::ptr_meta::from_raw_parts_mut(ptr.cast(), T::accessor_metadata(self.version));
            forget(self);
            // SAFETY: accessor_ptr conforms to the memory layout required by Box
            Box::from_raw(accessor_ptr)
        }
    }
}

impl<T: Versioned> fmt::Debug for Proto<T>
where
    T::Accessor: fmt::Debug,
    T::Version: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Proto")
            .field("value", &self.access())
            .field("version", &self.version)
            .finish()
    }
}
