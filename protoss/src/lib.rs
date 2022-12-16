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

pub mod rkyv;
mod test_util;

use core::fmt;

use ::ptr_meta::Pointee;
pub use crate::rkyv::ArchivedEvolution;
pub use crate::rkyv::AnyProbe;
pub use crate::rkyv::Evolve;
// pub use protoss_derive::protoss;

use ::rkyv::Archive;

/// A common error type for all errors that could occur in `protoss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    /// Tried to get [Probe][ProbeOf] metadata for a non-existent version of an [`Evolving`] type.
    TriedToGetProbeMetadataForNonExistentVersion,
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
            Self::InvalidBuilderFields => {
                write!(f, "tried to build a major version builder with an invalid combination of underlying fields that did not match any minor version")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// A version identifier containing the "minor" version.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Version(pub u16);

impl Version {
    /// Create a new [`Version`] from a given `minor` version
    pub const fn new(minor: u16) -> Self {
        Self(minor)
    }
}

/// A type that has multiple versions that may be changed over time. 
/// 
/// An [`Evolving`] type may have one or more **minor versions** which are *binary compatible*,
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
    /// The latest [`Evolution`] of `Self`
    type LatestEvolution: Evolution<Base = Self>;

    /// The latest [`Probe`] of `Self`
    type Probe: Probe<Base = Self> + ?Sized;

    /// Returns the [`Pointee::Metadata`] that can be used to construct a [`Probe`]
    /// which contains an [`Evolution`] of `Self` with the given `version`. In practical terms, this means
    /// the function returns the size in bytes of the [`Evolution`]'s [`Archived`][Archive::Archived] type for the specified [`Version`].
    /// This should be the same as the associated const [`Evolution::METADATA`] for that [`Evolution`].
    /// 
    /// For more information on what this means in-depth, see the Safety section in the trait-level documentation
    /// of [`Evolution`].
    fn probe_metadata(version: Version) -> Result<ProbeMetadata, crate::Error>;
}

/// Implemented by a specific concrete "evolution" (minor version) of an [`Evolving`] type.
/// 
/// # Safety
/// 
/// Implementing this trait means that it must be valid to construct (via [`ptr_meta::from_raw_parts`])
/// a [`Probe`] of `Self`'s assotiated [`Evolving`] type ([`Self::Base::Probe`][Evolution::Base])
/// with a data pointer to a [`<Self as Archive>::Archived`][Archive::Archived]
/// and the metadata returned by [`<Self::Base as Evolving>::probe_metadata(Self::VERSION)`][Evolving::probe_metadata]. This implies
/// the following requirements, as well as the ones discussed in the
/// documentation for both [`ptr_meta::Pointee`] and [`ptr_meta::from_raw_parts`]:
/// 
/// - For some [`Evolving`] type `E`, the [`Archived`] type of all [`Evolution`]s of `E` must have exactly the same
/// memory layout as the previous version until the end of the previous version's size. In plain speak
/// this means each version must only add new fields after the end of the previous version, and never change
/// the layout of fields that have already been used in a previous version. This also implies the following:
///     - The [`Archived`] type of all [`Evolution`] within the same **major version** must have size > (not >=) the version that came
/// before it (each version's size must be [*monotonically increasing*])
///     - The [`Archived`] type of all [`Evolution`] within the same **major version** must have an alignment >= the version that came before it
///     - The [`Archived`] type of `Self` must have no padding after its final field, i.e. the end of the memory that the final field
/// occupies must also be the end of the whole struct.
/// 
/// [`Archived`]: Archive::Archived
/// [`Self::ProbedBy`]: VersionOf::ProbedBy
/// [`E::probe_metadata(Self::VERSION)`]: Evolving::probe_metadata
/// [`ptr_meta` documentation]: ptr_meta::
/// [*monotonically increasing*]: https://mathworld.wolfram.com/MonotoneIncreasing.html
pub unsafe trait Evolution: Archive {
    /// The [`Evolving`] type that this type is an evolution of.
    type Base: Evolving + ?Sized;

    /// The version of `Self::Of` for which `Self` is the concrete definition
    const VERSION: Version;

    /// The [`Pointee::Metadata`] that can be used to construct a [`ProbeOf<Self>`]
    /// which contains this verion's archived data ([`<Self as Archive>::Archived`][Archive::Archived]).
    /// In practical terms, this is the size in bytes of [`Self::Archived`][Archive::Archived].
    const METADATA: ProbeMetadata;
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
pub unsafe trait Probe
where
    Self: Pointee<Metadata = ProbeMetadata>,
    Self: 'static,
{
    /// The [`Evolving`] type that this type is a probe of.
    type Base: Evolving + ?Sized;

    /// Returns the [`Version`] specifier of the actual contained version, if known.
    /// 
    /// The actual version may not be known if the contained version was created from a later versioned "producer"
    /// and consumed by an earlier-versioned "consumer" binary which does not have knowledge of the latest version(s).
    /// 
    /// You can think of this as conceptually similar to [`Any::type_id`][std::any::Any::type_id].
    fn version(&self) -> Option<Version>;

    /// "Probes" `self` as the given [`Evolution`].
    /// 
    /// Returns `Some(&V::Archived)` if `self` is a >= minor version and `None` if `self` is an earlier minor version.
    /// 
    /// You can think of this as conceptually similar to [`Any::downcast_ref`][std::any::Any::downcast_ref].
    fn probe_as<V: Evolution<Base = Self::Base>>(&self) -> Option<&V::Archived>;

    /// Assumes `self` is the given [`Evolution`] and casts self as that version.
    /// 
    /// # Safety
    /// 
    /// This probe must have been created with data that is binary-compatible with the given
    /// version: it must be an equal or later minor version of the same [`Evolving`] type (i.e. same 'major' version).
    /// 
    /// You can think of this as conceptually similar to [`Any::downcast_ref_unchecked`][std::any::Any::downcast_ref_unchecked].
    unsafe fn as_version_unchecked<V: Evolution<Base = Self::Base>>(&self) -> &V::Archived;

    /// Cast `&self` into a `&AnyProbe<E>`.
    /// 
    /// This is safe because you can't actually do anything (safely) with a `&AnyProbe<E>` and
    /// the [`Pointee::Metadata`] types are the same and valid between each other.
    fn as_any_probe(&self) -> &AnyProbe<Self::Base> {
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
    fn into_boxed_any_probe(self: Box<Self>) -> Box<AnyProbe<Self::Base>> {
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

/// This is a trait that all [`Probe`] types as well as [`AnyProbe`] can implement
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
    /// meaning a `P` backed by a [`Evolution`] that can be [`ProbedBy`][VersionOf::ProbedBy] `P`:
    /// 
    /// Specifically, `self` must have been created from properly aligned memory of the correct size, and
    /// [`ptr_meta::metadata(self)`] must give valid [`Pointee::Metadata`] for a `P` created from a data
    /// pointer to `self` and that metadata using [`ptr_meta::from_raw_parts`].
    unsafe fn as_probe_unchecked<P: Probe<Base = E> + ?Sized>(&self) -> &P {
        unsafe {
            &*::ptr_meta::from_raw_parts(
                (self as *const Self).cast(),
                ptr_meta::metadata(self),
            )
        }
    }
}
