#[cfg(feature = "rkyv")]
mod rkyv;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV0 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV1 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
    c: u32,
    _pad1: [u8; 0],
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct TestV2 {
    a: u32,
    b: u8,
    _pad0: [u8; 3],
    c: u32,
    _pad1: [u8; 0],
    d: u8,
    _pad2: [u8; 3],
}

mod v1 {
    use protoss::{VersionOf, Evolving};

    use super::{TestV0, TestV1};

    pub struct Test {
        pub a: u32,
        pub b: u8,
        pub c: u32,
    }

    // imagine this as Serialize
    impl From<Test> for TestV1 {
        fn from(Test { a, b, c}: Test) -> Self {
            TestV1 {
                a,
                b,
                c,
                _pad0: [0; 3],
                _pad1: [0; 0],
            }
        }
    }

    #[derive(ptr_meta::Pointee)]
    #[repr(transparent)]
    pub struct TestProbe {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type Probe = TestProbe;
        type Latest = TestV1;
        fn probe_metadata(version: u16) -> <Self::Probe as ptr_meta::Pointee>::Metadata {
            match version {
                 0 => core::mem::size_of::<TestV0>(),
                 1 => core::mem::size_of::<TestV1>(),
                _ => panic!("tried to get probe metadata for a version that doesn't exist")
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0 {
        const VERSION: u16 = 0;
    }
    unsafe impl VersionOf<Test> for TestV1 {
        const VERSION: u16 = 1;
    }

    impl TestProbe {
        pub fn probe_as<V: VersionOf<Test>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_unchecked() })
            } else {
                None
            }
        }

        #[inline(always)]
        pub unsafe fn as_unchecked<V: VersionOf<Test>>(&self) -> &V {
            &*self.data.as_ptr().cast::<V>()
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_unchecked::<TestV0>() };
            &v0.a
        }

        pub fn b(&self) -> &u8 {
            let v0 = unsafe { self.as_unchecked::<TestV0>() };
            &v0.b
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
    use protoss::{VersionOf, Evolving};

    use super::{TestV0, TestV1, TestV2};

    pub struct Test {
        pub a: u32,
        pub b: u8,
        pub c: u32,
        pub d: u8,
    }

    // imagine this as Serialize
    impl From<Test> for TestV2 {
        fn from(Test { a, b, c, d }: Test) -> Self {
            TestV2 {
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

    #[derive(ptr_meta::Pointee)]
    #[repr(transparent)]
    pub struct TestProbe {
        data: [u8]
    }

    unsafe impl Evolving for Test {
        type Probe = TestProbe;
        type Latest = TestV2;
        fn probe_metadata(version: u16) -> <Self::Probe as ptr_meta::Pointee>::Metadata {
            match version {
                 0 => core::mem::size_of::<TestV0>(),
                 1 => core::mem::size_of::<TestV1>(),
                 2 => core::mem::size_of::<TestV2>(),
                _ => panic!("tried to get probe metadata for a version that doesn't exist")
            }
        }
    }

    unsafe impl VersionOf<Test> for TestV0 {
        const VERSION: u16 = 0;
    }
    unsafe impl VersionOf<Test> for TestV1 {
        const VERSION: u16 = 1;
    }
    unsafe impl VersionOf<Test> for TestV2 {
        const VERSION: u16 = 2;
    }

    impl TestProbe {
        pub fn probe_as<V: VersionOf<Test>>(&self) -> Option<&V> {
            let data_size = core::mem::size_of_val(&self.data);
            let version_size = core::mem::size_of::<V>();
            if version_size <= data_size {
                Some(unsafe { self.as_unchecked() })
            } else {
                None
            }
        }

        #[inline(always)]
        pub unsafe fn as_unchecked<V: VersionOf<Test>>(&self) -> &V {
            &*self.data.as_ptr().cast::<V>()
        }

        pub fn a(&self) -> &u32 {
            let v0 = unsafe { self.as_unchecked::<TestV0>() };
            &v0.a
        }

        pub fn b(&self) -> &u8 {
            let v0 = unsafe { self.as_unchecked::<TestV0>() };
            &v0.b
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
        let v1_pylon: Pylon<v1::Test> = Pylon::new(TestV1::from(v1));

        let probe_v1 = v1_pylon.into_boxed_probe();

        assert_eq!(probe_v1.probe_as::<TestV0>(), Some(&TestV0 { a: 1, b: 2, _pad0: pad() }));
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
        let v1_pylon: Pylon<v1::Test> = Pylon::new(TestV1::from(v1));

        let v1_probe = v1_pylon.into_boxed_probe();

        let v2 = v2::Test {
            a: 5,
            b: 6, 
            c: 7,
            d: 8,
        };
        let v2_pylon: Pylon<v2::Test> = Pylon::new(TestV2::from(v2));

        let v2_probe = v2_pylon.into_boxed_probe();

        let v1_from_v2 = unsafe { core::mem::transmute::<&v2::TestProbe, &v1::TestProbe>(&v2_probe) };

        assert_eq!(v1_from_v2.probe_as::<TestV1>(), Some(&TestV1 { a: 5, b: 6, _pad0: pad(), c: 7, _pad1: pad() }));

        let v2_from_v1 = unsafe { core::mem::transmute::<&v1::TestProbe, &v2::TestProbe>(&v1_probe) };

        assert_eq!(v2_from_v1.probe_as::<TestV2>(), None);
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
