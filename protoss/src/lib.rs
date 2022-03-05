#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod proto;

use ::ptr_meta::Pointee;
pub use proto::*;
pub use protoss_derive::protoss;

/// A type that has multiple versions that may be changed over time.
///
/// # Safety
///
/// `accessor_metadata` must return valid metadata to construct an `Accessor` using a pointer to
/// the given version of this type.
pub unsafe trait Versioned {
    /// The type that can be used to access the versioned data.
    type Accessor: Pointee + ?Sized;

    /// The type used to store the version of the data.
    type Version: Copy + PartialEq;

    /// The latest version of the type.
    const LATEST: Self::Version;

    /// Returns the metadata of an `Accessor` for the given version.
    fn accessor_metadata(version: Self::Version) -> <Self::Accessor as Pointee>::Metadata;
}
