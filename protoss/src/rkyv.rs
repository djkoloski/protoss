// TODO:
// - Serialize a Pylon<E> as a ArchivedEvolution<E>
// - Serialize an Rc/Arc<Pylon<E>> as an Archived[Arc/Rc]Evolution<E>

use core::marker::PhantomData;

use ptr_meta::Pointee;
use rkyv::Archive;
use rkyv::ArchivePointee;
use rkyv::Archived;
use rkyv::Fallible;
use rkyv::Serialize;
use rkyv::boxed::ArchivedBox;
use rkyv::boxed::BoxResolver;
use rkyv::from_archived;
use rkyv::out_field;
use rkyv::ser::Serializer;
use rkyv::to_archived;
use rkyv::with::ArchiveWith;
use rkyv::with::SerializeWith;

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

impl ArchivedVersion {
    /// Get the unarchived [`Version`] of `self`.
    pub fn unarchived(&self) -> Version {
        Version::new(from_archived!(self.major), from_archived!(self.minor))
    }
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

impl<S: Fallible + ?Sized> Serialize<S> for Version {
    #[inline]
    fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
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

    /// Resolves an archived evolution from the given parameters.
    /// 
    /// You won't need to use this method unless you're manually implementing [`Serialize`]/[`Archive`] for an [`Evolving`] type,
    /// in which case it might be useful. It's used to help implement the provided derive macros.
    /// 
    /// # Safety
    /// 
    /// - `pos` must be the position of `out` within the archive
    /// - `resolver` must be the result of serializing
    /// (via [`serialize_with_version_serializer`][ArchivedEvolution::serialize_with_version_serializer]) the same [`VersionOf<E>`], `V`.
    pub unsafe fn resolve_from_version<V>(pos: usize, resolver: ArchivedEvolutionResolver<E, V>, out: *mut Self)
    where
        V: VersionOf<E>,
    {
        // first resolve the boxed anyprobe
        let (fp, fo) = out_field!(out.probe);

        // SAFETY: 
        let box_resolver = unsafe {
            BoxResolver::<Archived<ProbeMetadata>>::from_raw_parts(
                resolver.pos,
                core::mem::size_of::<V>() as Archived<ProbeMetadata>,
            )
        };

        // SAFETY:
        // - pos + fp is the position of fo within the archive
        // - resolver is the result of serializing the inner value in the archive and contains valid metadata for an AnyProbe<E>
        // containing the archived version
        unsafe {
            ArchivedBox::resolve_from_raw_parts(pos + fp, box_resolver, fo);
        }

        // next resolve the version number field
        let (fp, fo) = out_field!(out.version);

        let version = V::VERSION;

        // SAFETY:
        // - pos + fp is the position of fo within the archive
        // - doesn't need a resolver
        unsafe {
            version.resolve(pos + fp, (), fo);
        }
    }
    
    /// Serializes an archived evolution from a "`version_serializer: &VS`", where `VS` is a type that implements [`rkyv::Serialize`] with an
    /// [`Archived`][rkyv::Archive::Archived] type `V` that is some [`VersionOf<E>`].
    /// 
    /// The main example of such a "version serializer" type is the base `E: Evolving` type, which should implement [`Serialize`] & [`Archive`] with an
    /// [`Archive::Archived`] type that is [`<E as Evolving>::LatesteVersion`][Evolving::LatestVersion].
    /// 
    /// You won't need to use this method unless you're manually implementing [`Serialize`]/[`Archive`] for an [`Evolving`] type,
    /// in which case it might be useful. It's used to help implement the provided derive macros.
    pub fn serialize_with_version_serializer<V, VS, S>(version_serializer: &VS, serializer: &mut S) -> Result<ArchivedEvolutionResolver<E, V>, S::Error>
    where
        V: VersionOf<E>,
        VS: Serialize<S, Archived = V>,
        S: Serializer + ?Sized,
    {
        let pos = serializer.serialize_value(version_serializer)?;
        // SAFETY: `pos` is indeed the position of the given version within the archive since we just serialized it ourselves.
        Ok(unsafe { ArchivedEvolutionResolver::from_archived_version_pos(pos) })
    }
}

pub struct ArchivedEvolutionResolver<E: Evolving + ?Sized, V: VersionOf<E>> {
    _phantom: PhantomData<fn(E, V) -> ()>,
    pos: usize
}

impl<E: Evolving + ?Sized, V: VersionOf<E>> ArchivedEvolutionResolver<E, V> {
    /// Create a new [`ArchivedEvolutionResolver<E, V>`] from the given position.
    /// 
    /// Usually you wouldn't need to create this type directly and can rather obtain it from
    /// [`ArchivedEvlution::serialize_with_version_serializer`].
    /// 
    /// # Safety
    /// 
    /// Technically you can't directly cause bad behavior here, but marked as unsafe because
    /// caution needs to be taken. `pos` must be the position of an archived (serialized + resolved)
    /// `V` within the same archive that this [`ArchivedEvolutionResolver`] will be used to resolve
    /// an [`ArchivedEvolution`].
    pub unsafe fn from_archived_version_pos(pos: usize) -> Self {
        Self {
            _phantom: PhantomData,
            pos: pos,
        }
    }
}

/// An [`ArchiveWith`] modifier that serializes an [`Evolving`] type into an [`ArchivedEvolution`]. Without using this
/// modifier, an [`Evolving`] type will serialize as its [`Evolving::LatestVersion`] directly, which does not give the
/// compatibility guarantees and helpers that an [`ArchivedEvolution`] does. See the documentation
/// of [`ArchivedEvolution`] for more.
/// 
/// # Example
/// 
/// ```rust,no_run
/// #[derive(Archive, Serialize, Deserialize)]
/// struct Container {
///     #[with(Evolve)]
///     my_evolving_field: MyEvolvingStruct,
/// }
/// ```
pub struct Evolve;

impl<E> ArchiveWith<E> for Evolve
where
    E: Evolving + Archive<Archived = E::LatestVersion>
{
    type Archived = ArchivedEvolution<E>;
    type Resolver = ArchivedEvolutionResolver<E, E::LatestVersion>;

    /// # Safety
    ///
    /// - `pos` must be the position of `out` within the archive
    /// - `resolver` must be the result of serializing `field`
    /// with `Evolve` (`serialize_with`)
    unsafe fn resolve_with(
            _field: &E,
            pos: usize,
            resolver: Self::Resolver,
            out: *mut Self::Archived,
    ) {
        // SAFETY:
        // - pos is the position of `out` within the archive as long as function-level safety is upheld
        // - resolver is the result of serializing the field which serialized into an archived E::LatestVersion
        // as long as function-level safety is upheld
        unsafe {
            ArchivedEvolution::resolve_from_version(pos, resolver, out);
        }
    }
}

impl<S, E> SerializeWith<E, S> for Evolve
where
    S: Serializer + ?Sized,
    E: Evolving + Serialize<S, Archived = E::LatestVersion>,
{
    fn serialize_with(field: &E, serializer: &mut S) -> Result<Self::Resolver, <S as Fallible>::Error> {
        ArchivedEvolution::serialize_with_version_serializer(field, serializer)
    }
}
