#[cfg(feature = "rkyv")]
mod rkyv;

#[cfg(test)]
mod tests {
    use protoss::{Composite, Partial};

    macro_rules! impl_composite {
        (
            struct $composite:ident as $parts:ident {
                $($field:ident ($field_mut:ident): $ty:ty,)*
            }
        ) => {
            #[repr(C)]
            struct $composite {
                $($field: $ty,)*
            }

            unsafe impl Composite for $composite {
                type Parts = $parts;
            }

            #[repr(transparent)]
            #[derive(ptr_meta::Pointee)]
            struct $parts {
                bytes: [u8],
            }

            impl Drop for $parts {
                fn drop(&mut self) {
                    unsafe {
                        $(
                            if let Some(field) = self.$field_mut() {
                                core::ptr::drop_in_place(field as *mut $ty);
                            }
                        )*
                    }
                }
            }

            impl $parts {
                $(
                    #[allow(dead_code)]
                    fn $field(&self) -> Option<&$ty> {
                        unsafe {
                            let struct_ptr = (self as *const Self).cast::<$composite>();
                            let field_ptr = core::ptr::addr_of!((*struct_ptr).$field);
                            let offset = field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>()) as usize;
                            let size = core::mem::size_of::<$ty>();
                            if offset + size > self.bytes.len() {
                                None
                            } else {
                                Some(&*field_ptr)
                            }
                        }
                    }

                    #[allow(dead_code)]
                    fn $field_mut(&mut self) -> Option<&mut $ty> {
                        unsafe {
                            let struct_ptr = (self as *mut Self).cast::<$composite>();
                            let field_ptr = core::ptr::addr_of_mut!((*struct_ptr).$field);
                            let offset = field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>()) as usize;
                            let size = core::mem::size_of::<$ty>();
                            if offset + size > self.bytes.len() {
                                None
                            } else {
                                Some(&mut *field_ptr)
                            }
                        }
                    }
                )*
            }
        }
    }

    impl_composite! {
        struct ExampleV0 as ExampleV0Parts {
            a (a_mut): i32,
        }
    }

    impl_composite! {
        struct ExampleV1 as ExampleV1Parts {
            a (a_mut): i32,
            b (b_mut): String,
        }
    }

    impl_composite! {
        struct ExampleV2 as ExampleV2Parts {
            a (a_mut): i32,
            b (b_mut): String,
            c (c_mut): Option<usize>,
        }
    }

    #[test]
    fn basic_evolution() {
        let partial_v0 = Partial::new(ExampleV0 {
            a: 1,
        });

        let partial_v1 = Partial::new(ExampleV1 {
            a: 2,
            b: String::from("foo"),
        });

        let partial_v2 = Partial::new(ExampleV2 {
            a: 3,
            b: String::from("bar"),
            c: Some(100),
        });

        use core::mem::transmute;

        let v1_v0 = unsafe { transmute::<&ExampleV0Parts, &ExampleV1Parts>(partial_v0.parts()) };
        let v1_v1 = partial_v1.parts();
        let v1_v2 = unsafe { transmute::<&ExampleV2Parts, &ExampleV1Parts>(partial_v2.parts()) };

        assert_eq!(v1_v0.a(), Some(&1));
        assert_eq!(v1_v0.b(), None);

        assert_eq!(v1_v1.a(), Some(&2));
        assert_eq!(v1_v1.b(), Some(&String::from("foo")));

        assert_eq!(v1_v2.a(), Some(&3));
        assert_eq!(v1_v2.b(), Some(&String::from("bar")));
    }

    #[test]
    fn into_boxed_parts() {
        let partial_v1 = Partial::new(ExampleV1 {
            a: 2,
            b: String::from("foo"),
        });

        let parts_v1 = partial_v1.into_boxed_parts();

        assert_eq!(parts_v1.a(), Some(&2));
        assert_eq!(parts_v1.b(), Some(&String::from("foo")));
    }

    #[test]
    fn check_drop() {
        use std::rc::Rc;

        impl_composite! {
            struct ExampleDropV0 as ExampleDropPartsV0 {
                a (a_mut): Rc<i32>,
            }
        }

        impl_composite! {
            struct ExampleDropV1 as ExampleDropPartsV1 {
                a (a_mut): Rc<i32>,
                b (b_mut): Rc<i32>,
            }
        }

        let a = Rc::new(0);
        let b = Rc::new(1);

        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(Rc::strong_count(&b), 1);

        let partial_v0 = Partial::new(ExampleDropV0 {
            a: a.clone(),
        });

        assert_eq!(Rc::strong_count(&a), 2);
        assert_eq!(Rc::strong_count(&b), 1);

        let partial_v1 = Partial::new(ExampleDropV1 {
            a: a.clone(),
            b: b.clone(),
        });

        assert_eq!(Rc::strong_count(&a), 3);
        assert_eq!(Rc::strong_count(&b), 2);

        let parts_v0 = partial_v0.into_boxed_parts();

        assert_eq!(Rc::strong_count(&a), 3);
        assert_eq!(Rc::strong_count(&b), 2);

        let parts_v1 = partial_v1.into_boxed_parts();

        assert_eq!(Rc::strong_count(&a), 3);
        assert_eq!(Rc::strong_count(&b), 2);

        core::mem::drop(parts_v0);

        assert_eq!(Rc::strong_count(&a), 2);
        assert_eq!(Rc::strong_count(&b), 2);

        core::mem::drop(parts_v1);

        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(Rc::strong_count(&b), 1);
    }

    #[test]
    fn check_boxed_drop() {
        use std::rc::Rc;

        impl_composite! {
            struct ExampleDrop as ExampleDropParts {
                a (a_mut): Rc<i32>,
            }
        }

        let a = Rc::new(0);

        assert_eq!(Rc::strong_count(&a), 1);

        {
            let partial = Partial::new(ExampleDrop {
                a: a.clone(),
            });

            assert_eq!(Rc::strong_count(&a), 2);

            let boxed_parts = partial.into_boxed_parts();

            assert_eq!(Rc::strong_count(&a), 2);

            // Explicitly drop boxed parts to avoid unused variable warnings
            core::mem::drop(boxed_parts);
        }

        assert_eq!(Rc::strong_count(&a), 1);
    }

    #[test]
    fn check_derive() {
        use protoss::protoss;

        #[protoss]
        pub struct Test {
            #[version = 0]
            pub a: i32,
            pub b: i32,
            #[version = 1]
            pub c: u32,
            pub d: u8,
        }

        let test_v0 = Test::partial_v0(1, 2).into_boxed_parts();
        let test_v1 = Test::partial_v1(1, 2, 3, 4).into_boxed_parts();

        assert_eq!(test_v0.a(), test_v1.a());
        assert_eq!(test_v0.b(), test_v1.b());
        assert_eq!(test_v0.c(), None);
        assert_eq!(test_v0.d(), None);
        assert_eq!(test_v1.c(), Some(&3));
        assert_eq!(test_v1.d(), Some(&4));
    }
}
