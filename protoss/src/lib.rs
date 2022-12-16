//! `protoss` implements a protocol for [schema evolution] of
//! binary serialized data, designed to be used with [`rkyv`][::rkyv].
//! 
//! It offers **full** compatibilty\* (forward and backward) and **zero-copy deserialization** among "[`Evolution`]s"
//! of a base "[`Evolving`]" type, under a restrictive set of allowed changes, similar to Flatbuffers and Cap'n Proto.
//!
//! \* *A note on compatibility types:*
//! * **Backward** compatibility means that consumers (readers of serialized data) can read data
//! *produced by* an **older** version of the schema.
//! * **Forward** compatiblity means that consumers (readers of serialized data) can read data
//! *produced by* a **newer** version of the schema.
//! 
//! [`Evolution`]s of an [`Evolving`] type are allowed to:
//! * Add new fields
//!     - These new fields are always treated as optional
//!     - Fields may only be added to the end of an existing type
//!         - *but when using the provided derive macros, if you use field `id`s, you can define them in any order in code as long as the `id`s dont change*
//! * Rename existing fields (*but not change the type*)
//!
//! They are not allowed to:
//! * Remove existing fields entirely
//! * Change the type of existing fields
//! * Re-order existing fields
//! * Add new fields to the middle of an existing type
//! 
//! You can think of this as a similar type of schema evolution as what Protocol Buffers, Flatbuffers, and Cap'n Proto
//! offer. Existing consumers expecting a previous version may still read data produced with the new version as
//! the old version, and consumers expecting the new version will still be able to read all the fields that were defined
//! by the older producer.
//! 
//! For more on how this works, see the documentation of the [`Evolving`] trait, which is the centerpiece of the `protoss`
//! model.
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
    /// Tried to get [Probe] metadata for a non-existent version of an [`Evolving`] type.
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

/// A version identifier containing which "minor" version.
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
/// An [`Evolving`] type may have one or more [`Evolution`]s, which can also be thought of as *minor versions* or
/// *collections of binary-compatible changes* to their schema.
///
/// Each unique [`Evolution`] has a concrete backing type which defines its exact set of fields, and which implements
/// [`Archive`] such that its [`Archived`][Archive::Archived] type follows a specific set of rules that guarantee binary
/// compatibility between such archived [`Evolution`]s.
/// 
/// For example, say we have a type which we want to evolve over time, called `MyType`. Let's say that right now, it has
/// two "evolutions", i.e. two compatible versions of its schema (0 and 1). The core `MyType` should have all the latest fields
/// (from evolution 1) and implement [`Evolving`], and there should be two version structs, `MyTypeV0` and `MyTypeV1`, which each
/// implement [`Evolution`] with [`Base = MyType`][Evolution::Base], as well as
/// [`Archive`] with [`Archived`][Archive::Archived] types `ArchivedMyTypeV0` and `ArchivedMyTypeV1`, respectively. When
/// using the derive macros, these version and archived version structs will be generated for you.
/// 
/// Each [`Evolving`] type also has a concrete "[`Probe`]" type, which is
/// able to "poke at" or "probe" serialized binary data which
/// contains an **unknown** [`Evolution`] of that base [`Evolving`] type.
/// Through "probing", we are able to determine which actual [evolution][`Evolution`] it contains,
/// and therefore access it as the specific [evolution of `Self`][`Evolution`] we have determined it to be,
/// or alternatively attempt to access any known individual field directly.
/// 
/// [`Evolving`] types may be [serialized][::rkyv::Serialize] into an [`ArchivedEvolution`], which may hold *any* [`Evolution`] of that type.
/// From the [`ArchivedEvolution<E>`] you can obtain a reference to the base [`Evolving`] type's [`Probe`], and from that [`Probe`] attempt to
/// access any of its known fields individually, or attempt to access it as a specific archived [`Evolution`] direclty (and therefore get access
/// to all the fields included in that [`Evolution`] at zero cost if it succeeds).
/// 
/// # Safety
///
/// - `probe_metadata` must return valid metadata to construct a [`Probe`][Evolving::Probe] when combined with
/// a pointer to the archived type of an [`Evolution`] where that [`Evolution`]'s [VERSION][Evolution::VERSION] is equal to the
/// passed-in `version` parameter. See the documentation of [`Evolution`] for more.
/// - `LatestEvolution` must be the newest [`Evolution`] of `Self`
/// - `Probe` must be a [`Probe`] type capable of handling all [`Evolution`]s of `Self`
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
/// [`ptr_meta` documentation]: ptr_meta::
/// [*monotonically increasing*]: https://mathworld.wolfram.com/MonotoneIncreasing.html
pub unsafe trait Evolution: Archive {
    /// The [`Evolving`] type that this type is an evolution of.
    type Base: Evolving + ?Sized;

    /// The version identifier of this evolution of `Self::Base`
    const VERSION: Version;

    /// The [`Pointee::Metadata`] that can be used to construct a [`Probe<Base = Self::Base>`]
    /// which contains this verion's archived data ([`<Self as Archive>::Archived`][Archive::Archived]).
    /// In practical terms, this is the size in bytes of [`Self::Archived`][Archive::Archived].
    const METADATA: ProbeMetadata;
}

/// All probe types must have this as their [`<Self as Pointee>::Metadata`][Pointee::Metadata]
pub type ProbeMetadata = <[u8] as Pointee>::Metadata;

/// Implemented by a concrete [Probe] type for a specific [`Evolving`] type.
/// 
/// "[`Probe`]" types are able to "poke at" or "[probe][Probe::probe_as]" binary data
/// which contains an **unknown** [`Evolution`] of an [`Evolving`]
/// type in order to determine which actual version it contains and access the contained fields.
/// Probes will often use this ability to also implement helper accessor methods that attempt to access
/// each individual field contained in any (known) minor version of that type.
/// 
/// The key method of [`Probe`] is [`probe_as`][Probe::probe_as], through which richer functionality can
/// then be built.
/// 
/// # Safety
/// 
/// - See [`Evolution`]
/// - TODO: describe more actual requirements here
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
    /// Returns `Some(&EV::Archived)` if `self` is a >= minor version and `None` if `self` is an earlier minor version.
    /// 
    /// You can think of this as conceptually similar to `Any::downcast_ref`.
    fn probe_as<EV: Evolution<Base = Self::Base>>(&self) -> Option<&EV::Archived>;

    /// Assumes `self` is the given [`Evolution`] and casts self as that version.
    /// 
    /// # Safety
    /// 
    /// This probe must have been created with data that is binary-compatible with the given
    /// version: it must be an equal or later minor version of the same [`Evolving`] type (i.e. same 'major' version).
    /// 
    /// You can think of this as conceptually similar to `Any::downcast_ref_unchecked`.
    unsafe fn as_version_unchecked<EV: Evolution<Base = Self::Base>>(&self) -> &EV::Archived;

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
    /// Unsafely "casts" `Self` as a concrete [Probe][Probe] type.
    /// 
    /// # Safety
    /// 
    /// This method is extremely unsafe because it allows you to construct a `P`,
    /// which then has safe interfaces with very particular requirements.
    /// 
    /// In order for this to be valid, `Self` must have originally been a valid `P`,
    /// meaning a `P` backed by a [`Evolution`] of `<P as Probe>::Base`:
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
