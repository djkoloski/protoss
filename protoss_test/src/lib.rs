mod shared {
    use rkyv::{Archived, Archive, Serialize, Deserialize};
    use protoss::rkyv::PadToAlign;

    #[derive(Debug, Archive, Serialize, Deserialize)]
    #[archive(as = "ArchivedTestV0_0")]
    pub struct TestV0_0 {
        pub a: u32,
        pub b: u8,
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct ArchivedTestV0_0 {
        pub a: u32,
        pub b: u8,
        pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    #[archive(as = "ArchivedTestV0_1")]
    pub struct TestV0_1 {
        pub a: u32,
        pub b: u8,
        pub c: u32,
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct ArchivedTestV0_1 {
        pub a: Archived<u32>,
        pub b: Archived<u8>,
        pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
        pub c: Archived<u32>,
        pub _pad1: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>)>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    #[archive(as = "ArchivedTestV0_2")]
    pub struct TestV0_2 {
        pub a: u32,
        pub b: u8,
        pub c: u32,
        pub d: u8
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct ArchivedTestV0_2 {
        pub a: Archived<u32>,
        pub b: Archived<u8>,
        pub _pad0: PadToAlign<(Archived<u32>, Archived<u8>)>,
        pub c: Archived<u32>,
        pub _pad1: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>)>,
        pub d: Archived<u8>,
        pub _pad2: PadToAlign<(Archived<u32>, Archived<u8>, Archived<u32>, Archived<u8>)>,
    }

    #[derive(Debug, Archive, Serialize, Deserialize)]
    #[archive(as = "ArchivedTestV1_0")]
    pub struct TestV1_0 {
        pub a: u32,
        pub b: u32,
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct ArchivedTestV1_0 {
        pub a: Archived<u32>,
        pub b: Archived<u32>,
        pub _pad0: PadToAlign<(Archived<u32>, Archived<u32>)>,
    }
}

mod v1 {
    use protoss::{VersionOf, Evolving, AnyProbe, ProbeOf, Version};
    use ptr_meta::Pointee;

    use super::shared::*;

    // #[derive(Evolving)]
    // #[evolving(current_version = 0.1)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(as = "<<Self as Evolving>::LatestVersion as Archive>::Archived")]
    pub struct Test {
        //#[field(id = 0, since_minor_version = 0)]
        pub a: u32,
        //#[field(id = 1, since_minor_version = 0)]
        pub b: u8,
        //#[field(id = 2, since_minor_version = 1)]
        pub c: u32,
    }

    // imagine this as Serialize
    impl From<Test> for ArchivedTestV0_1 {
        fn from(Test { a, b, c}: Test) -> Self {
            ArchivedTestV0_1 {
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
    pub struct TestProbeMajor0 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeMajor0;
        type LatestVersion = TestV0_1;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match version.major_minor() {
                (0, 0) => Ok(size_of::<ArchivedTestV0_0>()),
                (0, 1) => Ok(size_of::<ArchivedTestV0_1>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 1);
    }

    unsafe impl ProbeOf<Test> for TestProbeMajor0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V::Archived {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }
    }

    impl TestProbeMajor0 {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u8> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.b)
        }

        pub fn c(&self) -> Option<&u32> {
            if let Some(v1) = self.probe_as::<TestV0_1>() {
                Some(&v1.c)
            } else {
                None
            }
        }
    }
}

mod v2 {
    use protoss::{VersionOf, Evolving, Version, ProbeOf, AnyProbe};
    use ptr_meta::Pointee;

    use super::shared::*;

    // #[derive(Evolving)]
    // #[evolving(current_version = 0.2)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(as = "<<Self as Evolving>::LatestVersion as Archive>::Archived")]
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
    impl From<Test> for ArchivedTestV0_2 {
        fn from(Test { a, b, c, d }: Test) -> Self {
            ArchivedTestV0_2 {
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
    pub struct TestProbeMajor0 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeMajor0;
        type LatestVersion = TestV0_2;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match (version.major, version.minor) {
                (0, 0) => Ok(size_of::<ArchivedTestV0_0>()),
                (0, 1) => Ok(size_of::<ArchivedTestV0_1>()),
                (0, 2) => Ok(size_of::<ArchivedTestV0_2>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 1);
    }
    unsafe impl VersionOf<Test> for TestV0_2 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 2);
    }

    unsafe impl ProbeOf<Test> for TestProbeMajor0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V::Archived {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }
    }

    impl TestProbeMajor0 {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u8> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.b)
        }

        pub fn c(&self) -> Option<&u32> {
            if let Some(v1) = self.probe_as::<TestV0_1>() {
                Some(&v1.c)
            } else {
                None
            }
        }

        pub fn d(&self) -> Option<&u8> {
            if let Some(v2) = self.probe_as::<TestV0_2>() {
                Some(&v2.d)
            } else {
                None
            }
        }
    }
}

mod v3 {
    use protoss::{VersionOf, Evolving, Version, ProbeOf, AnyProbe};
    use ptr_meta::Pointee;

    use super::shared::*;

    // #[derive(Evolving)]
    // #[evolving(current_version = 1.0)]
    #[derive(rkyv::Archive, rkyv::Serialize)]
    #[archive(as = "<<Self as Evolving>::LatestVersion as Archive>::Archived")]
    pub struct Test {
        //#[field(id = 0, since_minor_version = 0)]
        pub a: u32,
        //#[field(id = 1, since_minor_version = 0)]
        pub b: u32,
    }

    // #[protoss::previous_evolution_definition(
    //     of = Test,
    //     major = 0,
    // )]
    // pub struct TestMajor0 {
    //     //#[field(id = 0, since_minor_version = 0)]
    //     pub a: u32,
    //     //#[field(id = 1, since_minor_version = 0)]
    //     pub b: u8,
    //     //#[field(id = 2, since_minor_version = 1)]
    //     pub c: u32,
    //     //#[field(id = 3, since_minor_version = 2)]
    //     pub d: u8,
    // }

    // pub enum TestMajor0 {
    //     Minor0(TestV0_0),
    //     Minor1(TestV0_1),
    //     Minor2(TestV0_2),
    // }

    // pub enum TestMajor1 {
    //     Minor0(TestV1_0)
    // }

    // trait DefEvolutionOfTest {
    //     fn upgrade_major_0_to_major_1(major_0: &TestProbeMajor0) -> TestMajor1;
    // }

    // impl DefEvolutionOfTest for Test {
    //     fn upgrade_major_0_to_major_1(major_0: &TestProbeMajor0) -> TestMajor1 {
    //
    //     }
    // }

    // imagine this as Serialize
    impl From<Test> for ArchivedTestV1_0 {
        fn from(Test { a, b }: Test) -> Self {
            ArchivedTestV1_0 {
                a,
                b,
                _pad0: Default::default(),
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeMajor0 {
        data: [u8]
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeMajor1 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeMajor1;
        type LatestVersion = TestV1_0;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match (version.major, version.minor) {
                (0, 0) => Ok(size_of::<ArchivedTestV0_0>()),
                (0, 1) => Ok(size_of::<ArchivedTestV0_1>()),
                (0, 2) => Ok(size_of::<ArchivedTestV0_2>()),
                (1, 0) => Ok(size_of::<ArchivedTestV1_0>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 1);
    }
    unsafe impl VersionOf<Test> for TestV0_2 {
        type ProbedBy = TestProbeMajor0;
        const VERSION: Version = Version::new(0, 2);
    }
    unsafe impl VersionOf<Test> for TestV1_0 {
        type ProbedBy = TestProbeMajor1;
        const VERSION: Version = Version::new(1, 0);
    }

    unsafe impl ProbeOf<Test> for TestProbeMajor0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V::Archived {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }
    }

    impl TestProbeMajor0 {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u8> {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            Some(&v0.b)
        }

        pub fn c(&self) -> Option<&u32> {
            if let Some(v1) = self.probe_as::<TestV0_1>() {
                Some(&v1.c)
            } else {
                None
            }
        }

        pub fn d(&self) -> Option<&u8> {
            if let Some(v2) = self.probe_as::<TestV0_2>() {
                Some(&v2.d)
            } else {
                None
            }
        }
    }

    unsafe impl ProbeOf<Test> for TestProbeMajor1 {
        const PROBES_MAJOR_VERSION: u16 = 1;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V::Archived {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V::Archived>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V::Archived> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V::Archived>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked::<V>() })
            } else {
                None
            }
        }
    }

    impl TestProbeMajor1 {
        pub fn a(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV1_0>() };
            Some(&v0.a)
        }

        pub fn b(&self) -> Option<&u32> {
            let v0 = unsafe { self.as_version_unchecked::<TestV1_0>() };
            Some(&v0.b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::shared::*;

    use protoss::ProbeOf;
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
        let v1_pylon: Pylon<v1::Test> = Pylon::new::<TestV0_1>(ArchivedTestV0_1::from(v1)).unwrap();

        let probe_v1 = v1_pylon.into_boxed_probe();

        assert_eq!(probe_v1.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe_v1.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
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
        let v1_pylon: Pylon<v1::Test> = Pylon::new::<TestV0_1>(ArchivedTestV0_1::from(v1)).unwrap();

        let v1_probe = v1_pylon.into_boxed_probe();

        let v2 = v2::Test {
            a: 5,
            b: 6, 
            c: 7,
            d: 8,
        };
        let v2_pylon: Pylon<v2::Test> = Pylon::new::<TestV0_2>(ArchivedTestV0_2::from(v2)).unwrap();

        let v2_probe = v2_pylon.into_boxed_probe();

        let v1_from_v2 = unsafe { core::mem::transmute::<&v2::TestProbeMajor0, &v1::TestProbeMajor0>(&v2_probe) };

        assert_eq!(v1_from_v2.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 5, b: 6, _pad0: pad() }));
        assert_eq!(v1_from_v2.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));

        let v2_from_v1 = unsafe { core::mem::transmute::<&v1::TestProbeMajor0, &v2::TestProbeMajor0>(&v1_probe) };

        assert_eq!(v2_from_v1.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(v2_from_v1.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(v2_from_v1.probe_as::<TestV0_2>(), None);
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

        let probe = archived_test.try_as_latest_probe().unwrap();

        assert_eq!(probe.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
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
        let probe: &v2::TestProbeMajor0 = archived_test.try_as_latest_probe().unwrap();

        assert_eq!(probe.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 1, b: 2, _pad0: pad(), c: 3, _pad1: pad() }));
        assert_eq!(probe.probe_as::<TestV0_2>(), None);
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
        let probe: &v1::TestProbeMajor0 = archived_test.try_as_latest_probe().unwrap();

        assert_eq!(probe.probe_as::<TestV0_0>(), Some(&ArchivedTestV0_0 { a: 5, b: 6, _pad0: pad() }));
        assert_eq!(probe.probe_as::<TestV0_1>(), Some(&ArchivedTestV0_1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));
        // compile fails because v1 doesn't know about V0_2!
        // assert_eq!(probe.probe_as::<TestV0_2>(), Some(&ArchivedTestV0_2 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad(), d: 8, _pad2: pad() }));
        assert_eq!(probe.a(), Some(&5));
        assert_eq!(probe.b(), Some(&6));
        assert_eq!(probe.c(), Some(&7));
        // compile fails because v1 doesn't know about field d on V0_2!
        // assert_eq!(probe.d(), Some(&8));
    }
}
