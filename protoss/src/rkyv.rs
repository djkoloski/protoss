// TODO:
// - Serialize a Pylon<E> as a ArchivedEvolution<E>
// - Serialize an Rc/Arc<Pylon<E>> as an Archived[Arc/Rc]Evolution<E>

use core::marker::PhantomData;

use ptr_meta::Pointee;
use rkyv::Archive;
use rkyv::ArchivePointee;
use rkyv::Archived;
use rkyv::boxed::ArchivedBox;
use rkyv::from_archived;
use rkyv::to_archived;

use crate::Evolving;
use crate::ProbeMetadata;
use crate::ProbeOf;
use crate::RawProbe;
use crate::Version;
use crate::VersionOf;

/// The archived type of [`Version`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchivedVersion {
    /// The major version
    pub major: Archived<u16>,
    /// The minor version
    pub minor: Archived<u16>,
}

impl Archive for Version {
    type Archived = ArchivedVersion;
    type Resolver = ();

    unsafe fn resolve(&self, _: usize, _: Self::Resolver, out: *mut Self::Archived) {
        unsafe {
            out.write(ArchivedVersion {
                major: to_archived!(self.major),
                minor: to_archived!(self.minor),
            })
        }
    }
}

impl ArchivedVersion {
    /// Get the unarchived [`Version`] of `self`.
    pub fn unarchived(&self) -> Version {
        Version::new(from_archived!(self.major), from_archived!(self.minor))
    }
}

/// A type-erased [Probe][ProbeOf] for some `E`. This could contain any concrete [`ProbeOf<E>`]
/// (there should be one [Probe][ProbeOf] for each major version of `E`).
/// 
/// Constructing and otherwise using this type is extremely fraught, it's unlikely you'll need or
/// want to interact with this type directly unless you want to create your own alternative to
/// [`ArchivedEvolution`].
#[repr(transparent)]
pub struct AnyProbe<E: Evolving + ?Sized> {
    _phantom: PhantomData<E>,
    #[allow(dead_code)]
    data: [u8]
}

impl<E: Evolving + ?Sized> Pointee for AnyProbe<E> {
    type Metadata = ProbeMetadata;
}

impl<E: Evolving + ?Sized> RawProbe<E> for AnyProbe<E> {}

impl<E: Evolving + ?Sized> ArchivePointee for AnyProbe<E> {
    type ArchivedMetadata = Archived<ProbeMetadata>;

    fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
        from_archived!(*archived) as usize
    }
}

/// The archived version of some [`Evolving`] type `E`, containing the data for *some version* of that
/// `E` as well as a version descriptor of which version is contained.
/// 
/// We can attempt to downcast into a concrete [`ProbeOf<E>`], i.e. a [Probe][ProbeOf] for some specific
/// **major version** of `E`, or a specific [`VersionOf<E>`] directly, and upon success, access the data
/// contained inside in a zero-copy fashion.
/// 
/// If the accessed data has an outdated major version, you can still fully [deserialize][::rkyv::Deserialize]
/// it as the latest major version through upgrade functions, though of course this will no longer be zero-copy.**
/// 
/// \*\* TODO: this is not actually implemented yet.
/// 
/// # Safety
/// 
/// Constructing this type is extremely fraught! It should only be constructed by casting existing data
/// and not constructed directly as an owned value.
#[repr(C)]
pub struct ArchivedEvolution<E: Evolving + ?Sized> {
    probe: ArchivedBox<AnyProbe<E>>,
    version: ArchivedVersion,
}

impl<E: Evolving + ?Sized> Drop for ArchivedEvolution<E> {
    fn drop(&mut self) {
        panic!("dropped an ArchivedEvolution! This should not be possible, since they should never be constructed as an owned value.");
    }
}

impl<E: Evolving + ?Sized> ArchivedEvolution<E> {
    /// Get the [`Version`] identifier of the contained [`VersionOf<E>`] in `self`.
    pub fn version(&self) -> Version {
        self.version.unarchived()
    }

    /// Try to downcast `self` as the given concrete [`ProbeOf<E>`].
    /// 
    /// For this to succeed, the contained version in `self` must be:
    /// - the **major version** that `P` is able to probe.
    pub fn try_as_probe<P: ProbeOf<E> + ?Sized>(&self) -> Option<&P> {
        if self.version.major == P::PROBES_MAJOR_VERSION {
            Some(unsafe { self.probe.as_probe_unchecked() })
        } else {
            None
        }
    }

    /// Try to downcast `self` as the [`ProbeOf<E>`] corresponding to the latest
    /// known (to the compiled binary) major version of `E` ([`Evolving::LatestProbe`]).
    pub fn try_as_latest_probe(&self) -> Option<&E::LatestProbe> {
        self.try_as_probe()
    }

    /// Attempt to downcast `self` as the given concrete [`VersionOf<E>`] directly.
    /// 
    /// For this to succeed, the contained version in `self` must be:
    /// - the same **major version** as `V`
    /// - the same or later **minor version** as `V`
    pub fn probe_as_version<V: VersionOf<E>>(&self) -> Option<&V> {
        if let Some(probe) = self.try_as_probe::<V::ProbedBy>() {
            probe.probe_as()
        } else {
            None
        }
    }
}