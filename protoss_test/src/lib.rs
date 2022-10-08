#[cfg(feature = "rkyv")]
mod rkyv;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV0_0 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV0_1 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
    c: u32,
    _pad1: [u8; 0],
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV0_2 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
    c: u32,
    _pad1: [u8; 0],
    d: u8,
    _pad2: [u8; 3],
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV1_0 {
    a: u32,
    b: u32,
}

mod v1 {
    use protoss::{VersionOf, Evolving, AnyProbe, ProbeOf, Version};
    use ptr_meta::Pointee;

    use super::{TestV0_0, TestV0_1};

    pub struct Test {
        pub a: u32,
        pub b: u8,
        pub c: u32,
    }

    // imagine this as Serialize
    impl From<Test> for TestV0_1 {
        fn from(Test { a, b, c}: Test) -> Self {
            TestV0_1 {
                a,
                b,
                c,
                _pad0: [0; 3],
                _pad1: [0; 0],
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeV0 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeV0;
        type LatestVersion = TestV0_1;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match version.major_minor() {
                (0, 0) => Ok(size_of::<TestV0_0>()),
                (0, 1) => Ok(size_of::<TestV0_1>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 1);
    }

    unsafe impl ProbeOf<Test> for TestProbeV0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked() })
            } else {
                None
            }
        }
    }

    impl TestProbeV0 {
        pub fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            <Self as ProbeOf<Test>>::probe_as(self)
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.a
        }

        pub fn b(&self) -> &u8 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.b
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

    use super::{TestV0_0, TestV0_1, TestV0_2};

    pub struct Test {
        pub a: u32,
        pub b: u8,
        pub c: u32,
        pub d: u8,
    }

    // imagine this as Serialize
    impl From<Test> for TestV0_2 {
        fn from(Test { a, b, c, d }: Test) -> Self {
            TestV0_2 {
                a,
                b,
                c,
                d,
                _pad0: [0; 3],
                _pad1: [0; 0],
                _pad2: [0; 3],
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeV0 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeV0;
        type LatestVersion = TestV0_2;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match (version.major, version.minor) {
                (0, 0) => Ok(size_of::<TestV0_0>()),
                (0, 1) => Ok(size_of::<TestV0_1>()),
                (0, 2) => Ok(size_of::<TestV0_2>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 1);
    }
    unsafe impl VersionOf<Test> for TestV0_2 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 2);
    }

    unsafe impl ProbeOf<Test> for TestProbeV0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked() })
            } else {
                None
            }
        }
    }

    impl TestProbeV0 {
        pub fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            <Self as ProbeOf<Test>>::probe_as(self)
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.a
        }

        pub fn b(&self) -> &u8 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.b
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

    use super::{TestV0_0, TestV0_1, TestV0_2, TestV1_0};

    pub struct Test {
        pub a: u32,
        pub b: u32,
    }

    // imagine this as Serialize
    impl From<Test> for TestV1_0 {
        fn from(Test { a, b }: Test) -> Self {
            TestV1_0 {
                a,
                b,
            }
        }
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeV0 {
        data: [u8]
    }

    #[derive(Pointee)]
    #[repr(transparent)]
    pub struct TestProbeV1 {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type LatestProbe = TestProbeV1;
        type LatestVersion = TestV1_0;
        fn probe_metadata(version: Version) -> Result<<AnyProbe<Test> as Pointee>::Metadata, protoss::Error> {
            use core::mem::size_of;
            match (version.major, version.minor) {
                (0, 0) => Ok(size_of::<TestV0_0>()),
                (0, 1) => Ok(size_of::<TestV0_1>()),
                (0, 2) => Ok(size_of::<TestV0_2>()),
                (1, 0) => Ok(size_of::<TestV1_0>()),
                _ => Err(protoss::Error::TriedToGetProbeMetadataForNonExistentVersion)
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0_0 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 0);
    }
    unsafe impl VersionOf<Test> for TestV0_1 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 1);
    }
    unsafe impl VersionOf<Test> for TestV0_2 {
        type ProbedBy = TestProbeV0;
        const VERSION: Version = Version::new(0, 2);
    }
    unsafe impl VersionOf<Test> for TestV1_0 {
        type ProbedBy = TestProbeV1;
        const VERSION: Version = Version::new(1, 0);
    }

    unsafe impl ProbeOf<Test> for TestProbeV0 {
        const PROBES_MAJOR_VERSION: u16 = 0;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked() })
            } else {
                None
            }
        }
    }

    impl TestProbeV0 {
        pub fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            <Self as ProbeOf<Test>>::probe_as(self)
        }

        #[inline(always)]
        pub unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            <Self as ProbeOf<Test>>::as_version_unchecked(self)
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.a
        }

        pub fn b(&self) -> &u8 {
            let v0 = unsafe { self.as_version_unchecked::<TestV0_0>() };
            &v0.b
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

    unsafe impl ProbeOf<Test> for TestProbeV1 {
        const PROBES_MAJOR_VERSION: u16 = 1;

        #[inline(always)]
        unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            debug_assert!(V::VERSION.major == Self::PROBES_MAJOR_VERSION);
            &*self.data.as_ptr().cast::<V>()
        }

        fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_version_unchecked() })
            } else {
                None
            }
        }
    }

    impl TestProbeV1 {
        pub fn probe_as<V: VersionOf<Test, ProbedBy = Self>>(&self) -> Option<&V> {
            <Self as ProbeOf<Test>>::probe_as(self)
        }

        #[inline(always)]
        pub unsafe fn as_version_unchecked<V: VersionOf<Test, ProbedBy = Self>>(&self) -> &V {
            <Self as ProbeOf<Test>>::as_version_unchecked(self)
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_version_unchecked::<TestV1_0>() };
            &v0.a
        }

        pub fn b(&self) -> &u32 {
            let v0 = unsafe { self.as_version_unchecked::<TestV1_0>() };
            &v0.b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use protoss::Pylon;

    const fn pad<const N: usize>() -> [u8; N] {
        [0u8; N]
    }

    #[test]
    fn into_boxed_probe() {
        let v1 = v1::Test {
            a: 1,
            b: 2,
            c: 3,
        };
        let v1_pylon: Pylon<v1::Test> = Pylon::new(TestV0_1::from(v1));

        let probe_v1 = v1_pylon.into_boxed_probe();

        assert_eq!(probe_v1.probe_as::<TestV0_0>(), Some(&TestV0_0 { a: 1, b: 2, _pad0: pad() }));
        assert_eq!(probe_v1.a(), &1);
        assert_eq!(probe_v1.b(), &2);
        assert_eq!(probe_v1.c(), Some(&3));
    }

    #[test]
    fn basic_evolution() {
        let v1 = v1::Test {
            a: 1,
            b: 2,
            c: 3,
        };
        let v1_pylon: Pylon<v1::Test> = Pylon::new(TestV0_1::from(v1));

        let v1_probe = v1_pylon.into_boxed_probe();

        let v2 = v2::Test {
            a: 5,
            b: 6, 
            c: 7,
            d: 8,
        };
        let v2_pylon: Pylon<v2::Test> = Pylon::new(TestV0_2::from(v2));

        let v2_probe = v2_pylon.into_boxed_probe();

        let v1_from_v2 = unsafe { core::mem::transmute::<&v2::TestProbe, &v1::TestProbe>(&v2_probe) };

        assert_eq!(v1_from_v2.probe_as::<TestV0_1>(), Some(&TestV0_1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));

        let v2_from_v1 = unsafe { core::mem::transmute::<&v1::TestProbe, &v2::TestProbe>(&v1_probe) };

        assert_eq!(v2_from_v1.probe_as::<TestV0_2>(), None);
        assert_eq!(v2_from_v1.a(), &1);
        assert_eq!(v2_from_v1.c(), Some(&3));
    }


    // #[test]
    // fn check_drop() {
    //     use std::rc::Rc;

    //     impl_composite! {
    //         struct ExampleDropV0 as ExampleDropPartsV0 {
    //             a (a_mut): Rc<i32>,
    //         }
    //     }

    //     impl_composite! {
    //         struct ExampleDropV1 as ExampleDropPartsV1 {
    //             a (a_mut): Rc<i32>,
    //             b (b_mut): Rc<i32>,
    //         }
    //     }

    //     let a = Rc::new(0);
    //     let b = Rc::new(1);

    //     assert_eq!(Rc::strong_count(&a), 1);
    //     assert_eq!(Rc::strong_count(&b), 1);

    //     let partial_v0 = Partial::new(ExampleDropV0 {
    //         a: a.clone(),
    //     });

    //     assert_eq!(Rc::strong_count(&a), 2);
    //     assert_eq!(Rc::strong_count(&b), 1);

    //     let partial_v1 = Partial::new(ExampleDropV1 {
    //         a: a.clone(),
    //         b: b.clone(),
    //     });

    //     assert_eq!(Rc::strong_count(&a), 3);
    //     assert_eq!(Rc::strong_count(&b), 2);

    //     let parts_v0 = partial_v0.into_boxed_parts();

    //     assert_eq!(Rc::strong_count(&a), 3);
    //     assert_eq!(Rc::strong_count(&b), 2);

    //     let parts_v1 = partial_v1.into_boxed_parts();

    //     assert_eq!(Rc::strong_count(&a), 3);
    //     assert_eq!(Rc::strong_count(&b), 2);

    //     core::mem::drop(parts_v0);

    //     assert_eq!(Rc::strong_count(&a), 2);
    //     assert_eq!(Rc::strong_count(&b), 2);

    //     core::mem::drop(parts_v1);

    //     assert_eq!(Rc::strong_count(&a), 1);
    //     assert_eq!(Rc::strong_count(&b), 1);
    // }

    // #[test]
    // fn check_boxed_drop() {
    //     use std::rc::Rc;

    //     impl_composite! {
    //         struct ExampleDrop as ExampleDropParts {
    //             a (a_mut): Rc<i32>,
    //         }
    //     }

    //     let a = Rc::new(0);

    //     assert_eq!(Rc::strong_count(&a), 1);

    //     {
    //         let partial = Partial::new(ExampleDrop {
    //             a: a.clone(),
    //         });

    //         assert_eq!(Rc::strong_count(&a), 2);

    //         let boxed_parts = partial.into_boxed_parts();

    //         assert_eq!(Rc::strong_count(&a), 2);

    //         // Explicitly drop boxed parts to avoid unused variable warnings
    //         core::mem::drop(boxed_parts);
    //     }

    //     assert_eq!(Rc::strong_count(&a), 1);
    // }

    // #[test]
    // fn check_derive() {
    //     use protoss::protoss;

    //     #[protoss]
    //     pub struct Test {
    //         #[version = 0]
    //         pub a: i32,
    //         pub b: i32,
    //         #[version = 1]
    //         pub c: u32,
    //         pub d: u8,
    //     }

    //     let test_v0 = Test::partial_v0(1, 2).into_boxed_parts();
    //     let test_v1 = Test::partial_v1(1, 2, 3, 4).into_boxed_parts();

    //     assert_eq!(test_v0.a(), test_v1.a());
    //     assert_eq!(test_v0.b(), test_v1.b());
    //     assert_eq!(test_v0.c(), None);
    //     assert_eq!(test_v0.d(), None);
    //     assert_eq!(test_v1.c(), Some(&3));
    //     assert_eq!(test_v1.d(), Some(&4));
    // }
}
