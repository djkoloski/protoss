//! Things related to actually implementing `rkyv` for `protoss`.

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
use crate::Probe;
use crate::RawProbe;
use crate::Version;
use crate::Evolution;

/// The archived type of [`Version`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchivedVersion(pub Archived<u16>);

impl ArchivedVersion {
    /// Get the unarchived [`Version`] of `self`.
    pub fn unarchived(&self) -> Version {
        Version(from_archived!(self.0))
    }
}

impl Archive for Version {
    type Archived = ArchivedVersion;
    type Resolver = ();

    unsafe fn resolve(&self, _: usize, _: Self::Resolver, out: *mut Self::Archived) {
        unsafe {
            out.write(ArchivedVersion(to_archived!(self.0)));
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
/// **major version** of `E`, or a specific [`Evolution`] directly, and upon success, access the data
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
}

impl<E: Evolving + ?Sized> Drop for ArchivedEvolution<E> {
    fn drop(&mut self) {
        panic!("dropped an ArchivedEvolution! This should not be possible, since they should never be constructed as an owned value.");
    }
}

impl<E: Evolving + ?Sized> ArchivedEvolution<E> {
    /// Get the [`Version`] identifier of the contained [`Evolution`] in `self`, if known.
    /// 
    /// The actual version may not be known if the contained version was created from a later versioned "producer"
    /// and consumed by an earlier-versioned "consumer" binary which does not have knowledge of the latest version(s).
    pub fn version(&self) -> Option<Version> {
        self.as_probe().version()
    }

    /// Downcast `self` as the latest known (to the compiled binary) [`ProbeOf<E>`] ([`E::Probe`][Evolving::Probe]).
    #[inline(always)]
    pub fn as_probe(&self) -> &E::Probe {
        self.as_specific_probe::<E::Probe>()
    }

    /// Downcast `self` as the given concrete [`ProbeOf<E>`]. You probably want just [`as_probe`][ArchivedEvolution::as_probe] instead.
    #[inline(always)]
    pub fn as_specific_probe<P: Probe<Base = E> + ?Sized>(&self) -> &P {
        unsafe { self.probe.as_probe_unchecked() }
    }

    /// Attempt to downcast `self` as the archived version of the given concrete [`Evolution`] directly.
    /// 
    /// For this to succeed, the actual contained version in `self` must be the same or later [`Version`] as `V`.
    #[inline]
    pub fn probe_as_version<V: Evolution<Base = E>>(&self) -> Option<&V::Archived> {
        self.as_probe().probe_as::<V>()
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
    /// (via [`serialize_with_version_serializer`][ArchivedEvolution::serialize_with_version_serializer]) the same [`Evolution`], `V`.
    pub unsafe fn resolve_from_evolution<EV>(pos: usize, resolver: ArchivedEvolutionResolver<EV>, out: *mut Self)
    where
        EV: Evolution<Base = E>,
    {
        let (fp, fo) = out_field!(out.probe);

        // SAFETY: 
        let box_resolver = unsafe {
            BoxResolver::<Archived<ProbeMetadata>>::from_raw_parts(
                resolver.pos,
                core::mem::size_of::<EV::Archived>() as Archived<ProbeMetadata>,
            )
        };

        // SAFETY:
        // - pos + fp is the position of fo within the archive
        // - resolver is the result of serializing the inner value in the archive and contains valid metadata for an AnyProbe<E>
        // containing the archived version
        unsafe {
            ArchivedBox::resolve_from_raw_parts(pos + fp, box_resolver, fo);
        }
    }
    
    /// Serializes an archived evolution from a "`version_serializer: &VS`", where `VS` is a type that implements [`rkyv::Serialize`] with an
    /// [`Archived`][rkyv::Archive::Archived] type `V` that is some [`Evolution`].
    /// 
    /// The main example of such a "version serializer" type is the base `E: Evolving` type, which should implement [`Serialize`] & [`Archive`] with an
    /// [`Archive::Archived`] type that is [`<E as Evolving>::LatesteVersion`][Evolving::LatestVersion].
    /// 
    /// You won't need to use this method unless you're manually implementing [`Serialize`]/[`Archive`] for an [`Evolving`] type,
    /// in which case it might be useful. It's used to help implement the provided derive macros.
    pub fn serialize_with_evolution_serializer<EV, EVS, S>(evolution_serializer: &EVS, serializer: &mut S) -> Result<ArchivedEvolutionResolver<EV>, S::Error>
    where
        EV: Evolution<Base = E>,
        EVS: Serialize<S, Archived = <EV as Archive>::Archived>,
        S: Serializer + ?Sized,
    {
        let pos = serializer.serialize_value(evolution_serializer)?;
        // SAFETY: `pos` is indeed the position of the given version within the archive since we just serialized it ourselves.
        Ok(unsafe { ArchivedEvolutionResolver::from_archived_version_pos(pos) })
    }
}

/// The [`Archive::Resolver`] for [`ArchivedEvolution`].
pub struct ArchivedEvolutionResolver<EV: Evolution> {
    _phantom: PhantomData<fn(EV) -> ()>,
    pos: usize
}

impl<EV: Evolution> ArchivedEvolutionResolver<EV> {
    /// Create a new [`ArchivedEvolutionResolver<EV>`] from the given position.
    /// 
    /// Usually you wouldn't need to create this type directly and can rather obtain it from
    /// [`ArchivedEvlution::serialize_with_version_serializer`].
    /// 
    /// # Safety
    /// 
    /// Technically you can't directly cause bad behavior here, but marked as unsafe because
    /// caution needs to be taken. `pos` must be the position of an archived (serialized + resolved)
    /// `EV` within the same archive that this [`ArchivedEvolutionResolver`] will be used to resolve
    /// an [`ArchivedEvolution`].
    pub unsafe fn from_archived_version_pos(pos: usize) -> Self {
        Self {
            _phantom: PhantomData,
            pos: pos,
        }
    }
}

/// An [`ArchiveWith`] modifier that serializes an [`Evolving`] type into an [`ArchivedEvolution`]. Without using this
/// modifier, an [`Evolving`] type will serialize as its [`Evolving::LatestEvolution`] directly, which does not give the
/// compatibility guarantees and helpers that an [`ArchivedEvolution`] does. See the documentation
/// of [`ArchivedEvolution`] for more.
/// 
/// # Example
/// 
/// ```rust,no_run
/// # protoss::fake_evolving_struct!(MyEvolvingStruct);
/// # use rkyv::{Archive, Serialize, Deserialize};
/// use protoss::Evolve;
/// 
/// #[derive(Archive, Serialize, Deserialize)]
/// struct Container {
///     #[with(Evolve)]
///     my_evolving_field: MyEvolvingStruct,
/// }
/// ```
pub struct Evolve;

impl<E> ArchiveWith<E> for Evolve
where
    E: Evolving + Archive<Archived = <E::LatestEvolution as Archive>::Archived>
{
    type Archived = ArchivedEvolution<E>;
    type Resolver = ArchivedEvolutionResolver<E::LatestEvolution>;

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
            ArchivedEvolution::resolve_from_evolution(pos, resolver, out);
        }
    }
}

impl<S, E> SerializeWith<E, S> for Evolve
where
    S: Serializer + ?Sized,
    E: Evolving + Serialize<S, Archived = <E::LatestEvolution as Archive>::Archived>,
{
    fn serialize_with(field: &E, serializer: &mut S) -> Result<Self::Resolver, <S as Fallible>::Error> {
        ArchivedEvolution::serialize_with_evolution_serializer(field, serializer)
    }
}

/// This is used to help obey the layout rules imposed for archived [`Evolution`]s. You likely won't need to use
/// it yourself unless you're manually implementing [`Evolving`] for your type.
/// 
/// After each minor version's added fields, a dummy field with `PadToAlign<(...)>` should be added, where
/// `...` is a tuple of the types of each previous field,
/// in order. Putting this type in a zero-size array causes the compiler to automatically compute the needed alignment for the previous fields
/// and force the next field out of that alignment, thus preventing "niching" in the padding at the end of the previous version, and therefore
/// guaranteeing each minor version has monotonically increasing size.
/// 
/// You could also calculate the necessary padding manually and add a `[u8; size]` field, but this way is both
/// less easy to mess up and easier for a proc-macro to implement.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct PadToAlign<T>([T; 0]);

impl<T> Default for PadToAlign<T> {
    fn default() -> Self {
        Self([])
    }
}

impl<T> PartialEq for PadToAlign<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for PadToAlign<T> {}

impl<T> PartialOrd for PadToAlign<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}

impl<T> Ord for PadToAlign<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}

impl<T> core::hash::Hash for PadToAlign<T> {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

/// This function's name is a bit odd, it is just a short alias for [`PadToAlign::default()`]
pub fn pad<T>() -> PadToAlign<T> {
    Default::default()
}

impl<T> Archive for PadToAlign<T> {
    type Archived = Self;
    type Resolver = ();

    #[inline(always)]
    unsafe fn resolve(&self, _: usize, _: Self::Resolver, _: *mut Self::Archived) { }
}

impl<T, S: Fallible> Serialize<S> for PadToAlign<T> {
    #[inline(always)]
    fn serialize(&self, _: &mut S) -> Result<Self::Resolver, S::Error> {
        Ok(())
    }
}