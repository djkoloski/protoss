//! `protoss` implements a protocol for [schema evolution] of
//! binary serialized data, designed to be used with [`rkyv`][::rkyv].
//! 
//! It offers **full** compatibilty* (forward and backward) and **zero-copy deserialization** among **minor versions**
//! (under a restrictive set of allowed changes), and **backward** compatibility* among **major versions**
//! (allow arbitrary changes).
//! 
//! \* *A note on compatibility types:*
//! * **Backward** compatibility means that consumers (readers of serialized data) can read data
//! *produced by* an **older** version of the schema.
//! * **Forward** compatiblity means that consumers (readers of serialized data) can read data
//! *produced by* a **newer** version of the schema.
//! 
//! **Minor version** upgrades may:
//! * Add new fields
//!     - These new fields are always treated as optional
//!     - Fields may only be added to the end of an existing type
//!         - *but if you use ids you can define them in any order in code as long as the ids dont change*
//! 
//! You can think of this as a similar type of schema evolution as what Protocol Buffers, Flatbuffers, and Cap'n Proto
//! offer. Existing consumers expecting a previous version may still read data produced with the new version as
//! the old version, and consumers expecting the new version will still be able to read all the fields that were defined
//! by the older producer.
//! 
//! **Major version** upgrades may:
//! * Do anything they want to the type
//! 
//! After a major version change, existing consumers (readers of serialized data) expecting a *previous version*
//! will no longer be able to read data produced with the newer major version.
//! 
//! New consumers which have updated to the latest major version that expect the latest major version
//! will no longer have *zero copy* access to data produced with a previous version (unless they specifically
//! choose to ask for the data as the older major version). However, they *can* still get access to a new copy
//! of the data in the latest major version which has been upgraded (via a best-effort upgrade function
//! chain).**
//! 
//! \*\* *TODO: This not actually implemented at all yet ;p*
//! 
//! For more on how this works, see the documentation of the [`Evolving`] trait, which is the centerpiece of the `protoss`
//! model, for more.
//! 
//! Also, see the crate-level documentation of [`protoss_derive`] for info on how this system is intended to be
//! implemented/used by the end user.
//! 
//! [schema evolution]: https://martin.kleppmann.com/2012/12/05/schema-evolution-in-avro-protocol-buffers-thrift.html
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod pylon;
pub mod rkyv;
mod test_util;

use core::fmt;

use ::ptr_meta::Pointee;
pub use crate::rkyv::ArchivedEvolution;
pub use crate::rkyv::AnyProbe;
pub use crate::rkyv::Evolve;
// pub use pylon::Pylon;
// pub use protoss_derive::protoss;

use ::rkyv::Archive;

/// A common error type for all errors that could occur in `protoss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// Tried to get [Probe][ProbeOf] metadata for a non-existent version of an [`Evolving`] type.
    TriedToGetProbeMetadataForNonExistentVersion,
    /// Tried to create a [`Pylon<E, StorageV>`] by a [`VersionOf<E>`] that is from a
    /// different **major version** than the pylon's `StorageV`.
    CreatePylonWithUnmatchedMajorVersions,
    /// Tried to create a [`Pylon<E, StorageV>`] by a [`VersionOf<E>`] that has a newer (larger)
    /// **minor version** than the pylon's `StorageV`.
    CreatePylonWithNewerMinorVersionThanStorage,
    /// Tried to build a major version builder with an invalid combination of underlying fields,
    /// which does not match any existing minor version.
    InvalidBuilderFields,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TriedToGetProbeMetadataForNonExistentVersion => {
                write!(f, "tried to get probe metadata for a non-existent version of an Evolving type")
            }
            Self::CreatePylonWithUnmatchedMajorVersions => {
                write!(f, "tried to create a Pylon<E, StorageV> from a version of E that has different major version than StorageV")
            }
            Self::CreatePylonWithNewerMinorVersionThanStorage => {
                write!(f, "tried to create a Pylon<E, StorageV> from a version of E that has newer minor version than StorageV")
            }
            Self::InvalidBuilderFields => {
                write!(f, "tried to build a major version builder with an invalid combination of underlying fields that did not match any minor version")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// A version identifier containing a major and minor version.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Version {
    /// The **major version**.
    /// 
    /// Major versions are **not** binary-compatible with each other.
    pub major: u16,

    /// The **minor version**.
    /// 
    /// Minor versions of the same major version **are** binary-compatible with each other
    /// and may be probed by a [Probe][ProbeOf] compatible with the major version they are
    /// part of.
    pub minor: u16,
}

impl Version {
    /// Create a new [`Version`] from a given `major` and `minor` version
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Get a tuple `(major, minor)` of the versions contained in `self`
    pub const fn major_minor(self) -> (u16, u16) {
        (self.major, self.minor)
    }
}

/// A type that has multiple versions that may be changed over time. 
/// 
/// An [`Evolving`] type may have one or more **major versions** which are binary incompatible,
/// and each **major version** may have one or more **minor versions** which *are* binary compatible,
/// following a "schema evolution" process.
/// 
/// Each unique version has a concrete backing type which defines its exact set of fields and which implements
/// [`Archive`] such that its [`Archived`][Archive::Archived] type defines that specific version's exact archived layout.
/// Each of these concrete version types should implement [`VersionOf<Self>`].
/// 
/// For example, say we have a type which we want to evolve over time, called `MyType`. Let's say that right now, it has
/// one major version (0) and two minor versions (0 and 1). The core `MyType` should have all the latest fields and impl [`Evolving`],
/// and there should be two version structs, `MyTypeV0_0` and `MyTypeV0_1`, which each implement [`VersionOf<MyType>`], as well as
/// [`Archive`] with [`Archived`][Archive::Archived] types of `ArchivedMyTypeV0_0` and `ArchivedMyTypeV0_1`, respectively. When
/// using the derive macros, these version and archived version structs will be generated for you.
/// 
/// Each **major version** also has a concrete "[Probe][ProbeOf]" type, which is
/// able to "poke at" or "[probe][ProbeOf::probe_as]" serialized binary data which
/// contains an **unknown** *minor version* within some known *major version* of an `Self`.
/// Through "probing", we are able to determine which actual [version][`VersionOf`] it contains,
/// and therefore access it as the specific [version of `Self`][`VersionOf`] we have determined it to be.
/// 
/// [`Evolving`] types may be [serialized][::rkyv::Serialize] into an [`ArchivedEvolution`], which may hold *any* version of that type,
/// along with its version. When accessing it as an [`rkyv::Archive`][::rkyv::Archive]d type (i.e. zero-copy), you can then
/// attempt to downcast it to a desired major version's [Probe][ProbeOf] type which can then be used to get the
/// fully-compatible behavior of minor versions in a zero-copy fashion. If the accessed data has an outdated major version,
/// you can still fully [deserialize][::rkyv::Deserialize] it as the latest major version through upgrade functions,
/// though of course this will no longer be zero-copy. See the docs of [`ArchivedEvolution`] for more.
///
/// # Safety
///
/// - `probe_metadata` must return valid metadata to construct a `Probe` when combined with
/// a pointer to a type `V` that implements [`VersionOf<Self>`] where `V::VERSION == version`.
/// See the documentation of [`VersionOf`] for more.
/// - `LatestVersion` must be the newest version of `Self`
/// - `LatestProbe` must be a probe capable of handling all existing minor versions of the latest
/// major version of `Self`.
pub unsafe trait Evolving {
    /// The latest [`VersionOf<Self>`]
    type LatestVersion: VersionOf<Self>;

    /// The latest [`ProbeOf<Self>`]
    type LatestProbe: ProbeOf<Self> + ?Sized;

    /// Returns the [`Pointee::Metadata`] that can be used to construct a [`ProbeOf<Self>`]
    /// which contains a [`VersionOf<Self>`] with the given `version`. In practical terms, this means
    /// the function returns the size in bytes of the [`VersionOf<Self>`] for the specified [`Version`].
    /// 
    /// For more information on what this means in-depth, see the Safety section in the documentation
    /// of [`VersionOf`].
    fn probe_metadata(version: Version) -> Result<ProbeMetadata, crate::Error>;
}

/// Implemented by a specific concrete version of an [`Evolving`] type `E`.
/// 
/// # Safety
/// 
/// Implementing this trait means that it must be valid to construct (via [`ptr_meta::from_raw_parts`])
/// a [`Self::ProbedBy`] [Probe][ProbeOf] with a data pointer to a [`<Self as Archive>::Archived`][Archive::Archived]
/// and the metadata returned by [`E::probe_metadata(Self::VERSION)`]. This implies
/// the following requirements, as well as the ones discussed in the
/// documentation for both [`ptr_meta::Pointee`] and [`ptr_meta::from_raw_parts`]:
/// 
/// - For some `E`, the [`Archived`] type of all [`VersionOf<E>`] within the same **major version** must have exactly the same
/// memory layout as the previous version until the end of the previous version's size. In plain speak
/// this means each version must only add new fields after the end of the previous version, and never change
/// the layout of fields that have already been used in a previous version. This also implies the following:
///     - The [`Archived`] type of all [`VersionOf<E>`] within the same **major version** must have size > (not >=) the version that came
/// before it (each version's size must be [*monotonically increasing*])
///     - The [`Archived`] type of all [`VersionOf<E>`] within the same **major version** must have an alignment >= the version that came before it
///     - The [`Archived`] type of `Self` must have no padding after its final field, i.e. the end of the memory that the final field
/// occupies must also be the end of the whole struct.
/// 
/// [`Archived`]: Archive::Archived
/// [`Self::ProbedBy`]: VersionOf::ProbedBy
/// [`E::probe_metadata(Self::VERSION)`]: Evolving::probe_metadata
/// [`ptr_meta` documentation]: ptr_meta::
/// [*monotonically increasing*]: https://mathworld.wolfram.com/MonotoneIncreasing.html
pub unsafe trait VersionOf<E> 
where
    E: Evolving + ?Sized,
    Self: Archive,
{
    /// The version number of `E` for which `Self` is the concrete definition
    const VERSION: Version;

    /// The [`ProbeOf<E>`] type that is able to probe this version of `E` (this will be the same
    /// for all [`VersionOf<E>`] with the same major version)
    type ProbedBy: ProbeOf<E> + ?Sized;

    /// Cast `&self` as its [Probe][ProbeOf] type ([`Self::ProbedBy`][VersionOf::ProbedBy]).
    fn archived_as_probe(archived: &Self::Archived) -> &Self::ProbedBy {
        unsafe {
            &*::ptr_meta::from_raw_parts(
                (archived as *const Self::Archived).cast(),
                core::mem::size_of::<Self::Archived>() as ProbeMetadata,
            )
        }
    }
}

/// All probe types must have this as their [`<Self as Pointee>::Metadata`][Pointee::Metadata]
pub type ProbeMetadata = <[u8] as Pointee>::Metadata;

/// Implemented by a concrete [Probe][ProbeOf] for a specific *major version* of an [`Evolving`] type.
/// 
/// "[Probe][ProbeOf]" types are able to "poke at" or "[probe][ProbeOf::probe_as]" binary data
/// which contains an **unknown** *minor version* within some known *major version* of an [`Evolving`]
/// type in order to determine which actual version it contains. Probes will often use this ability to
/// also implement helper accessor methods that attempt to access each individual field contained in
/// any (known) minor version of that type.
/// 
/// # Safety
/// 
/// - See [`VersionOf`]
/// - TODO: describe the actual requirements here
pub unsafe trait ProbeOf<E>
where
    E: Evolving + ?Sized,
    Self: Pointee<Metadata = ProbeMetadata>,
    Self: 'static,
{
    /// The major version of `E` that `Self` probes
    const PROBES_MAJOR_VERSION: u16;

    /// "Probes" `self` as the given [`VersionOf<E>`].
    /// 
    /// Returns `Some(&V::Archived)` if `self` is a >= minor version and `None` if `self` is an earlier minor version.
    fn probe_as<V: VersionOf<E, ProbedBy = Self>>(&self) -> Option<&V::Archived>;

    /// Assumes `self` is the given [`VersionOf<E>`] and casts self as that version.
    /// 
    /// # Safety
    /// 
    /// This probe must have been created with data that is binary-compatible with the given
    /// version: it must be the same major version as [`Self::PROBES_MAJOR_VERSION`][ProbeOf::PROBES_MAJOR_VERSION]
    /// and an equal or later minor version.
    unsafe fn as_version_unchecked<V: VersionOf<E, ProbedBy = Self>>(&self) -> &V::Archived;

    /// Cast `&self` into a `&AnyProbe<E>`.
    /// 
    /// This is safe because you can't actually do anything (safely) with a `&AnyProbe<E>` and
    /// the [`Pointee::Metadata`] types are the same and valid between each other.
    fn as_any_probe(&self) -> &AnyProbe<E> {
        // SAFETY: This is safe because
        // - you can't actually do anything (safely) with a `&AnyProbe<E>` an
        // - the [`Pointee::Metadata`] types are the same and valid between each other
        // - the alignment requirements of Self are always more strict than `AnyProbe`
        unsafe {
            &*::ptr_meta::from_raw_parts(
                (self as *const Self).cast(),
                ptr_meta::metadata(self),
            )
        }
    }

    /// Cast a boxed version of `Self` into a `Box<AnyProbe<E>>`.
    /// 
    /// This is safe because the [`Pointee::Metadata`] for both is the same and
    /// you can't actually do anything (safely) with a `Box<AnyProbe<E>>` besides `Drop` it,
    /// and since it's still a `Box`, it will then deallocate the memory properly so long
    /// as it was allocated properly in the first place.
    fn into_boxed_any_probe(self: Box<Self>) -> Box<AnyProbe<E>> {
        let ptr = Box::into_raw(self);
        // SAFETY: 
        // This is safe because the [`Pointee::Metadata`] for both is the same and
        // you can't actually do anything (safely) with a `Box<AnyProbe<E>>` besides `Drop` it,
        // and since it's still a `Box`, it will then deallocate the memory properly so long
        // as it was allocated properly in the first place.
        unsafe {
            Box::from_raw(ptr_meta::from_raw_parts_mut(
                ptr.cast(),
                ptr_meta::metadata(ptr)
            ))
        }
    }
}

/// This is a trait that all [Probe][ProbeOf] types as well as [`AnyProbe`] can implement
/// which provides raw, unsafe helper interfaces.
/// 
/// Implementing this trait is not unsafe in and of itself, but using the
/// [`as_probe_unchecked`][RawProbe::as_probe_unchecked] method is extremely unsafe.
/// 
/// It's unlikely you want to work with this trait directly, but is used internally in the implementation
/// of [`ArchivedEvolution`], which you can then work with safely.
pub trait RawProbe<E>
where
    E: Evolving + ?Sized,
    Self: Pointee<Metadata = ProbeMetadata>,
{
    /// Unsafely "casts" `Self` as a concrete [Probe][ProbeOf] type.
    /// 
    /// # Safety
    /// 
    /// This method is extremely unsafe because it allows you to construct a `P`,
    /// which then has safe interfaces with very particular requirements.
    /// 
    /// In order for this to be valid, `Self` must have originally been a valid `P`,
    /// meaning a `P` backed by a [`VersionOf<E>`] that can be [`ProbedBy`][VersionOf::ProbedBy] `P`:
    /// 
    /// Specifically, `self` must have been created from properly aligned memory of the correct size, and
    /// [`ptr_meta::metadata(self)`] must give valid [`Pointee::Metadata`] for a `P` created from a data
    /// pointer to `self` and that metadata using [`ptr_meta::from_raw_parts`].
    unsafe fn as_probe_unchecked<P: ProbeOf<E> + ?Sized>(&self) -> &P {
        unsafe {
            &*::ptr_meta::from_raw_parts(
                (self as *const Self).cast(),
                ptr_meta::metadata(self),
            )
        }
    }
}
