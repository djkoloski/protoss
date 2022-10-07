#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod pylon;

use ::ptr_meta::Pointee;
pub use pylon::*;
pub use protoss_derive::protoss;

/// A type that has multiple versions that may be changed over time.
///
/// # Safety
///
/// - `probe_metadata` must return valid metadata to construct a `Probe` when combined with
/// a pointer to a type `V` that implements [`VersionOf<Self>`] where `V::VERSION == version`
/// - `Latest` must be the newest version of `Self`
pub unsafe trait Evolving: Sized {
    /// The type that can be used to access the versioned data generically
    type Probe: Pointee + ?Sized;

    /// The latest version of `Self`
    type Latest: VersionOf<Self>;

    // TODO: maybe just make this return the size... since that's effectively what
    // it does
    fn probe_metadata(version: u16) -> <Self::Probe as Pointee>::Metadata;
}

/// A version of an [`Evolving`] type.
/// 
/// # Safety
/// 
/// - It must be valid to construct an [`E::Probe`][Evolving::Probe] with a pointer to a `Self`
/// and the metadata returned by [`E::probe_metadata(Self::VERSION)`][Evolving::probe_metadata].
/// - For some `E`, all [`VersionOf<E>`] must have an alignment >= the version that came before it
/// - TODO: describe the actual requirements here
pub unsafe trait VersionOf<E: Evolving> {
    const VERSION: u16;
}
