#![deny(unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// mod pylon;

use core::fmt;
use core::marker::PhantomData;

use ::ptr_meta::Pointee;
// pub use pylon::*;
// pub use protoss_derive::protoss;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    TriedToGetProbeMetadataForNonExistentVersion
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TriedToGetProbeMetadataForNonExistentVersion => {
                write!(f, "tried to get probe metadata for a non-existent version of an Evolving type")
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
}

impl Version {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub const fn major_minor(self) -> (u16, u16) {
        (self.major, self.minor)
    }
}

/// A type-erased Probe. All concrete Probe types should have the same
/// layout and [`Metadata`][ptr_meta::Pointee::Metadata] as [`AnyProbe`].
#[repr(transparent)]
pub struct AnyProbe<E: Evolving + ?Sized> {
    _phantom: PhantomData<E>,
    data: [u8]
}


impl<E: Evolving + ?Sized> AnyProbe<E> {
    pub unsafe fn as_probe_unchecked<P: ProbeOf<E> + ?Sized>(&self) -> &P {
        unsafe {
            &*::ptr_meta::from_raw_parts(
                self.data.as_ptr().cast(),
                ptr_meta::metadata(self as *const Self),
            )
        }
    }
}

impl<E: Evolving + ?Sized> Pointee for AnyProbe<E> {
    type Metadata = <[u8] as Pointee>::Metadata;
}

/// An archived version of a Probe for some [`Evolving`] type `E`, containing a boxed [`AnyProbe`] and
/// a [`Version`].
/// 
/// Can be downcast into a specific [`ProbeOf<E>`], i.e. a Probe for some specific *major version* of `E`.
#[repr(C)]
pub struct ArchivedProbe<E: Evolving + ?Sized> {
    probe: Box<AnyProbe<E>>,
    version: Version,
}

impl<E: Evolving + ?Sized> ArchivedProbe<E> {
    pub fn try_as_probe<P: ProbeOf<E> + ?Sized>(&self) -> Option<&P> {
        if self.version.major == P::PROBES_MAJOR_VERSION {
            Some(unsafe { self.probe.as_probe_unchecked() })
        } else {
            None
        }
    }

    pub fn try_as_latest(&self) -> Option<&E::LatestProbe> {
        self.try_as_probe()
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn probe_as<V: VersionOf<E>>(&self) -> Option<&V> {
        if let Some(probe) = self.try_as_probe::<V::ProbedBy>() {
            probe.probe_as()
        } else {
            None
        }
    }
}

/// A type that has multiple versions that may be changed over time.
///
/// # Safety
///
/// - `probe_metadata` must return valid metadata to construct a `Probe` when combined with
/// a pointer to a type `V` that implements [`VersionOf<Self>`] where `V::VERSION == version`
/// - `Latest` must be the newest version of `Self`
pub unsafe trait Evolving {
    type LatestVersion: VersionOf<Self>;
    type LatestProbe: ProbeOf<Self> + ?Sized;
    fn probe_metadata(version: Version) -> Result<<AnyProbe<Self> as Pointee>::Metadata, crate::Error>;
}

/// A specific concrete version of an [`Evolving`] type.
/// 
/// # Safety
/// 
/// - It must be valid to construct an [`E::Probe`][Evolving::Probe] with a pointer to a `Self`
/// and the metadata returned by [`E::probe_metadata(Self::VERSION)`][Evolving::probe_metadata].
/// - For some `E`, all [`VersionOf<E>`] must have an alignment >= the version that came before it
/// - TODO: describe the actual requirements here
pub unsafe trait VersionOf<E: Evolving + ?Sized> {
    /// The Probe type that can probe this version of `E`
    type ProbedBy: ProbeOf<E> + ?Sized;

    const VERSION: Version;
}

/// A probe for a specific major version of an [`Evolving`] type.
/// 
/// # Safety
/// 
/// - See [`VersionOf`]
/// - TODO: describe the actual requirements here
pub unsafe trait ProbeOf<E>
where
    E: Evolving + ?Sized,
    Self: Pointee<Metadata = <AnyProbe<E> as Pointee>::Metadata>,
{
    /// The major version of `E` that `Self` probes
    const PROBES_MAJOR_VERSION: u16;

    /// "Probes" `self` as the given [`VersionOf<E>`].
    /// 
    /// Returns `Some(&V)` if `self` is a >= minor version and `None` if `self` is an earlier minor version.
    fn probe_as<V: VersionOf<E, ProbedBy = Self>>(&self) -> Option<&V>;

    unsafe fn as_version_unchecked<V: VersionOf<E, ProbedBy = Self>>(&self) -> &V;
}
