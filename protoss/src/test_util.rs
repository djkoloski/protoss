//! Things to help with testing protoss.

/// A macro to create a "fake" [`Evolving`][crate::Evolving] struct for use
/// in doctests so that it will compile but not work.
#[macro_export]
macro_rules! fake_evolving_struct {
    ($name:ident) => {
        #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
        #[archive(as = "<<Self as ::protoss::Evolving>::LatestEvolution as ::rkyv::Archive>::Archived")]
        struct $name {}
        #[derive(::rkyv::Archive, ::rkyv::Serialize)]
        struct FakeEvolution {}
        #[derive(::ptr_meta::Pointee)]
        struct FakeProbe {
            _data: [u8]
        }
        unsafe impl ::protoss::Evolution for FakeEvolution {
            type Base = $name;
            const VERSION: ::protoss::Version = ::protoss::Version::new(0);
            const METADATA: ::protoss::ProbeMetadata = core::mem::size_of::<<Self as ::rkyv::Archive>::Archived>() as ::protoss::ProbeMetadata;
        }
        unsafe impl ::protoss::Probe for FakeProbe {
            type Base = $name;
            fn probe_as<EV: ::protoss::Evolution<Base = $name>>(&self) -> Option<&EV::Archived> {
                unimplemented!()
            }
            unsafe fn as_version_unchecked<EV: ::protoss::Evolution<Base = $name>>(&self) -> &EV::Archived {
                unimplemented!()
            }
            fn version(&self) -> Option<::protoss::Version> {
                unimplemented!()
            }
        }
        unsafe impl ::protoss::Evolving for $name {
            type LatestEvolution = FakeEvolution;
            type Probe = FakeProbe;
            fn probe_metadata(v: ::protoss::Version) -> Result<::protoss::ProbeMetadata, ::protoss::Error> {
                unimplemented!()
            }
        }
    }
}
    