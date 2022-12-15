macro_rules! define_types {
    () => {
        use rkyv::{Archived, Archive, Serialize, Deserialize};
        use protoss::rkyv::PadToAlign;

        #[derive(Debug, Archive, Serialize, Deserialize)]
        #[archive(as = "ArchivedTestV0")]
        pub struct TestV0 {
            pub a: u32,
            pub b: u8,
        }

        #[derive(Debug, PartialEq)]
        #[repr(C)]
        pub struct ArchivedTestV0 {
            pub a: u32,
            pub b: u8,
            pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
        }

        #[derive(Debug, Archive, Serialize, Deserialize)]
        #[archive(as = "ArchivedTestV1")]
        pub struct TestV1 {
            pub a: u32,
            pub b: u8,
            pub c: u32,
        }

        #[derive(Debug, PartialEq)]
        #[repr(C)]
        pub struct ArchivedTestV1 {
            pub a: Archived<u32>,
            pub b: Archived<u8>,
            pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
            pub c: Archived<u32>,
            pub _pad1: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>)>,
        }

        #[derive(Debug, Archive, Serialize, Deserialize)]
        #[archive(as = "ArchivedTestV2")]
        pub struct TestV2 {
            pub a: u32,
            pub b: u8,
            pub c: u32,
            pub d: u8
        }

        #[derive(Debug, PartialEq)]
        #[repr(C)]
        pub struct ArchivedTestV2 {
            pub a: Archived<u32>,
            pub b: Archived<u8>,
            pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
            pub c: Archived<u32>,
            pub _pad1: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>)>,
            pub d: Archived<u8>,
            pub _pad2: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>, Archived<u8>)>,
        }
    }
}

mod v1 {
    use protoss::{Evolution, Evolving, AnyProbe, Probe, Version, ProbeMetadata};
    use ptr_meta::Pointee;

    define_types!();

    // #[derive(Evolving)]
    // #[evolving(current_version = 0.1)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(as = "<<Self as Evolving>::LatestEvolution as Archive>::Archived")]
    pub struct Test {
        //#[field(id = 0, since_minor_version = 0)]
        pub a: u32,
        //#[field(id = 1, since_minor_version = 0)]
        pub b: u8,
        //#[field(id = 2, since_minor_version = 1)]
        pub c: u32,
    }

    // imagine this as Serialize
    impl From<Test> for ArchivedTestV1 {
        fn from(Test { a, b, c}: Test) -> Self {
            ArchivedTestV1 {
                a,
                b,
                c,
                _pad0: Default::default(),
                _pad1: Default::default(),
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbe {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type Probe = TestProbe;
        type LatestEvolution = TestV1;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            match version {
                TestV0::VERSION => Ok(TestV0::METADATA),
                TestV1::VERSION => Ok(TestV1::METADATA),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl Evolution for TestV0 {
        type Base = Test;
        const VERSION: Version = Version::new(0);
        const METADATA: ProbeMetadata = core::mem::size_of::<Self::Archived>() as ProbeMetadata;
    }
    unsafe impl Evolution for TestV1 {
        type Base = Test;
        const VERSION: Version = Version::new(1);
        const METADATA: ProbeMetadata = core::mem::size_of::<Self::Archived>() as ProbeMetadata;
    }

    unsafe impl Probe for TestProbe {
        type Base = Test;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: Evolution<Base = Test>>(&self) -> &V::Archived {
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: Evolution<Base = Test>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }

        fn version(&self) -> Option<Version> {
            match core::mem::size_of_val(&self.data) as ProbeMetadata {
                TestV0::METADATA => Some(TestV0::VERSION),
                TestV1::METADATA => Some(TestV1::VERSION),
                _ => None,
            }
        }
    }

    impl TestProbe {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u8> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0>() };
            Some(&v0.b)
        }

        pub fn c(&self) -> Option<&u32> {
            if let Some(v1) = self.probe_as::<TestV1>() {
                Some(&v1.c)
            } else {
                None
            }
        }
    }
}

mod v2 {
    use protoss::{Evolution, Evolving, Version, Probe, AnyProbe, ProbeMetadata};
    use ptr_meta::Pointee;

    define_types!();

    // #[derive(Evolving)]
    // #[evolving(current_version = 0.2)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(as = "<<Self as Evolving>::LatestEvolution as Archive>::Archived")]
    pub struct Test {
        //#[field(id = 0, since_minor_version = 0)]
        pub a: u32,
        //#[field(id = 1, since_minor_version = 0)]
        pub b: u8,
        //#[field(id = 2, since_minor_version = 1)]
        pub c: u32,
        //#[field(id = 3, since_minor_version = 2)]
        pub d: u8,
    }

    // imagine this as Serialize
    impl From<Test> for ArchivedTestV2 {
        fn from(Test { a, b, c, d }: Test) -> Self {
            ArchivedTestV2 {
                a,
                b,
                c,
                d,
                _pad0: Default::default(),
                _pad1: Default::default(),
                _pad2: Default::default(),
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbe {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type Probe = TestProbe;
        type LatestEvolution = TestV2;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            match version {
                TestV0::VERSION => Ok(TestV0::METADATA),
                TestV1::VERSION => Ok(TestV1::METADATA),
                TestV2::VERSION => Ok(TestV2::METADATA),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl Evolution for TestV0 {
        type Base = Test;
        const VERSION: Version = Version::new(0);
        const METADATA: ProbeMetadata = core::mem::size_of::<Self::Archived>() as ProbeMetadata;
    }

    unsafe impl Evolution for TestV1 {
        type Base = Test;
        const VERSION: Version = Version::new(1);
        const METADATA: ProbeMetadata = core::mem::size_of::<Self::Archived>() as ProbeMetadata;
    }

    unsafe impl Evolution for TestV2 {
        type Base = Test;
        const VERSION: Version = Version::new(2);
        const METADATA: ProbeMetadata = core::mem::size_of::<Self::Archived>() as ProbeMetadata;
    }

    unsafe impl Probe for TestProbe {
        type Base = Test;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: Evolution<Base = Test>>(&self) -> &V::Archived {
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: Evolution<Base = Test>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }

        fn version(&self) -> Option<Version> {
            match core::mem::size_of_val(&self.data) as ProbeMetadata {
                TestV0::METADATA => Some(TestV0::VERSION),
                TestV1::METADATA => Some(TestV1::VERSION),
                TestV2::METADATA => Some(TestV2::VERSION),
                _ => None,
            }
        }
    }

    impl TestProbe {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u8> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0>() };
            Some(&v0.b)
        }

        pub fn c(&self) -> Option<&u32> {
            if let Some(v1) = self.probe_as::<TestV1>() {
                Some(&v1.c)
            } else {
                None
            }
        }

        pub fn d(&self) -> Option<&u8> {
            if let Some(v2) = self.probe_as::<TestV2>() {
                Some(&v2.d)
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use protoss::Probe;
    use protoss::pylon::Pylon;
    use protoss::Evolve;
    use protoss::rkyv::pad;
    use rkyv::AlignedVec;
    use rkyv::Archive;
    use rkyv::Archived;
    use rkyv::Serialize;
    use rkyv::archived_root;
    use rkyv::ser::Serializer;
    use rkyv::ser::serializers::AllocSerializer;

    type DefaultSerializer = AllocSerializer<256>;

    #[test]
    fn into_boxed_probe() {
        let v1 = v1::Test {
            a: 1,
            b: 2,
            c: 3,
        };
        let v1_pylon: Pylon<v1::Test> = Pylon::new::<v1::TestV1>(v1::ArchivedTestV1::from(v1)).unwrap();

        let probe_v1 = v1_pylon.into_boxed_probe();

        assert_eq!(probe_v1.probe_as::<v1::TestV0>(), Some(&v1::ArchivedTestV0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe_v1.probe_as::<v1::TestV1>(), Some(&v1::ArchivedTestV1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(probe_v1.a(), Some(&1));
        assert_eq!(probe_v1.b(), Some(&2));
        assert_eq!(probe_v1.c(), Some(&3));
    }

    #[test]
    fn basic_evolution_minor() {
        let v1 = v1::Test {
            a: 1,
            b: 2,
            c: 3,
        };
        let v1_pylon: Pylon<v1::Test> = Pylon::new::<v1::TestV1>(v1::ArchivedTestV1::from(v1)).unwrap();

        let v1_probe = v1_pylon.into_boxed_probe();

        let v2 = v2::Test {
            a: 5,
            b: 6, 
            c: 7,
            d: 8,
        };
        let v2_pylon: Pylon<v2::Test> = Pylon::new::<v2::TestV2>(v2::ArchivedTestV2::from(v2)).unwrap();

        let v2_probe = v2_pylon.into_boxed_probe();

        let v1_from_v2 = unsafe { core::mem::transmute::<&v2::TestProbe, &v1::TestProbe>(&v2_probe) };

        assert_eq!(v1_from_v2.probe_as::<v1::TestV0>(), Some(&v1::ArchivedTestV0 { a: 5, b: 6, _pad0: pad() }));
        assert_eq!(v1_from_v2.probe_as::<v1::TestV1>(), Some(&v1::ArchivedTestV1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));

        let v2_from_v1 = unsafe { core::mem::transmute::<&v1::TestProbe, &v2::TestProbe>(&v1_probe) };

        assert_eq!(v2_from_v1.probe_as::<v2::TestV0>(), Some(&v2::ArchivedTestV0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(v2_from_v1.probe_as::<v2::TestV1>(), Some(&v2::ArchivedTestV1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(v2_from_v1.probe_as::<v2::TestV2>(), None);
        assert_eq!(v2_from_v1.a(), Some(&1));
        assert_eq!(v2_from_v1.b(), Some(&2));
        assert_eq!(v2_from_v1.c(), Some(&3));
        assert_eq!(v2_from_v1.d(), None);
    }

    #[test]
    fn basic_archiving() {
        #[derive(Archive, Serialize)]
        struct Container {
            #[with(Evolve)]
            test: v1::Test,
        }

        let container = Container {
            test: v1::Test {
                a: 1,
                b: 2,
                c: 3,
            }
        };

        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&container).unwrap();
        let buf: AlignedVec = serializer.into_serializer().into_inner();

        let archived_container: &ArchivedContainer = unsafe { archived_root::<Container>(&buf) };
        let archived_test: &protoss::ArchivedEvolution<v1::Test> = &archived_container.test;

        let probe = archived_test.as_probe();

        assert_eq!(probe.probe_as::<v1::TestV0>(), Some(&v1::ArchivedTestV0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe.probe_as::<v1::TestV1>(), Some(&v1::ArchivedTestV1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(probe.a(), Some(&1));
        assert_eq!(probe.b(), Some(&2));
        assert_eq!(probe.c(), Some(&3));
    }

    #[test]
    fn basic_archived_backwards_compat_minor() {
        #[derive(Archive, Serialize)]
        struct ContainerV1 {
            #[with(Evolve)]
            test: v1::Test,
        }

        #[derive(Archive, Serialize)]
        struct ContainerV2 {
            #[with(Evolve)]
            test: v2::Test,
        }

        let container_v1 = ContainerV1 {
            test: v1::Test {
                a: 1,
                b: 2,
                c: 3,
            }
        };

        // producer is on v1, serializes a v1
        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&container_v1).unwrap();
        let buf: AlignedVec = serializer.into_serializer().into_inner();


        // consumer is on v2, accesses v1 archive as v2
        let archived_container: &Archived<ContainerV2> = unsafe { archived_root::<ContainerV2>(&buf) };
        let archived_test: &protoss::ArchivedEvolution<v2::Test> = &archived_container.test;

        // v2 probe from v1 archived data
        let probe: &v2::TestProbe = archived_test.as_probe();

        assert_eq!(probe.probe_as::<v2::TestV0>(), Some(&v2::ArchivedTestV0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe.probe_as::<v2::TestV1>(), Some(&v2::ArchivedTestV1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(probe.probe_as::<v2::TestV2>(), None);
        assert_eq!(probe.a(), Some(&1));
        assert_eq!(probe.b(), Some(&2));
        assert_eq!(probe.c(), Some(&3));
        assert_eq!(probe.d(), None);
    }

    #[test]
    fn basic_archived_forwards_compat_minor() {
        #[derive(Archive, Serialize)]
        struct ContainerV1 {
            #[with(Evolve)]
            test: v1::Test,
        }

        #[derive(Archive, Serialize)]
        struct ContainerV2 {
            #[with(Evolve)]
            test: v2::Test,
        }

        let container_v2 = ContainerV2 {
            test: v2::Test {
                a: 5,
                b: 6,
                c: 7,
                d: 8,
            }
        };

        // producer is on v2, serializes v2
        let mut serializer = DefaultSerializer::default();
        serializer.serialize_value(&container_v2).unwrap();
        let buf: AlignedVec = serializer.into_serializer().into_inner();


        // consumer is on v1, accesses v2-serialized archive as v1
        let archived_container: &Archived<ContainerV1> = unsafe { archived_root::<ContainerV1>(&buf) };
        let archived_test: &protoss::ArchivedEvolution<v1::Test> = &archived_container.test;

        // v1 probe from v2 archived data
        let probe: &v1::TestProbe = archived_test.as_probe();

        assert_eq!(probe.probe_as::<v1::TestV0>(), Some(&v1::ArchivedTestV0 { a: 5, b: 6, _pad0: pad() }));
        assert_eq!(probe.probe_as::<v1::TestV1>(), Some(&v1::ArchivedTestV1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));
        // compile fails because v1 doesn't know about V0_2!
        // assert_eq!(probe.probe_as::<TestV0_2>(), Some(&ArchivedTestV0_2 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad(), d: 8, _pad2: pad() }));
        assert_eq!(probe.a(), Some(&5));
        assert_eq!(probe.b(), Some(&6));
        assert_eq!(probe.c(), Some(&7));
        // compile fails because v1 doesn't know about field d on V0_2!
        // assert_eq!(probe.d(), Some(&8));
    }
}
