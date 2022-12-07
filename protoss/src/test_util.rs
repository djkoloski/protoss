//! Things to help with testing protoss.

/// A macro to create a "fake" [`Evolving`][crate::Evolving] struct for use
/// in doctests so that it will compile but not work.
#[macro_export]
macro_rules! fake_evolving_struct {
    ($name:ident) => {
        #[derive(::rkyv::Archive, ::rkyv::Serialize, ::rkyv::Deserialize)]
        #[archive(as = "FakeVersion")]
        struct $name {}
        struct FakeVersion {}
        #[derive(::ptr_meta::Pointee)]
        struct FakeProbe {
            _data: [u8]
        }
        unsafe impl ::protoss::VersionOf<$name> for FakeVersion {
            type ProbedBy = FakeProbe;
            const VERSION: ::protoss::Version = ::protoss::Version::new(0, 0);
        }
        unsafe impl ::protoss::ProbeOf<$name> for FakeProbe {
            const PROBES_MAJOR_VERSION: u16 = 0;
            fn probe_as<V: ::protoss::VersionOf<$name, ProbedBy = Self>>(&self) -> Option<&V> {
                unimplemented!()
            }
            unsafe fn as_version_unchecked<V: ::protoss::VersionOf<$name, ProbedBy = Self>>(&self) -> &V {
                unimplemented!()
            }
        }
        unsafe impl ::protoss::Evolving for $name {
            type LatestVersion = FakeVersion;
            type LatestProbe = FakeProbe;
            fn probe_metadata(v: ::protoss::Version) -> Result<::protoss::ProbeMetadata, ::protoss::Error> {
                unimplemented!()
            }
        }
    }
}
    