#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
#[cfg(feature = "rkyv")]
mod rkyv {















































    // Explicitly drop boxed parts to avoid unused variable warnings





    use protoss::protoss;
    use rkyv::{Archive, Serialize, Deserialize};
    #[repr(C)]
    #[archive_attr(repr(C))]
    struct TestVersion0 {
        a: i32,
        b: i32,
        _phantom: ::core::marker::PhantomData<Test>,
    }
    #[automatically_derived]
    #[doc = "An archived `TestVersion0`"]
    #[repr(C,)]
    struct ArchivedTestVersion0 where i32: ::rkyv::Archive,
           i32: ::rkyv::Archive,
           ::core::marker::PhantomData<Test>: ::rkyv::Archive {
        #[doc = "The archived counterpart of `TestVersion0::a`"]
        a: ::rkyv::Archived<i32>,
        #[doc = "The archived counterpart of `TestVersion0::b`"]
        b: ::rkyv::Archived<i32>,
        #[doc = "The archived counterpart of `TestVersion0::_phantom`"]
        _phantom: ::rkyv::Archived<::core::marker::PhantomData<Test>>,
    }
    #[automatically_derived]
    #[doc = "The resolver for archived `TestVersion0`"]
    struct TestVersion0Resolver where i32: ::rkyv::Archive,
           i32: ::rkyv::Archive,
           ::core::marker::PhantomData<Test>: ::rkyv::Archive {
        a: ::rkyv::Resolver<i32>,
        b: ::rkyv::Resolver<i32>,
        _phantom: ::rkyv::Resolver<::core::marker::PhantomData<Test>>,
    }
    #[automatically_derived]
    const _: () =
        {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for TestVersion0 where i32: ::rkyv::Archive,
             i32: ::rkyv::Archive,
             ::core::marker::PhantomData<Test>: ::rkyv::Archive {
                type Archived = ArchivedTestVersion0;
                type Resolver = TestVersion0Resolver;
                #[allow(clippy :: unit_arg)]
                #[inline]
                unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver,
                                  out: *mut Self::Archived) {
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).a;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.a).resolve(pos + fp, resolver.a, fo);
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).b;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.b).resolve(pos + fp, resolver.b, fo);
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out)._phantom;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self._phantom).resolve(pos + fp, resolver._phantom, fo);
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl <__S: Fallible + ?Sized> Serialize<__S> for TestVersion0
             where i32: Serialize<__S>, i32: Serialize<__S>,
             ::core::marker::PhantomData<Test>: Serialize<__S> {
                #[inline]
                fn serialize(&self, serializer: &mut __S)
                 -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(TestVersion0Resolver{a:
                                                Serialize::<__S>::serialize(&self.a,
                                                                            serializer)?,
                                            b:
                                                Serialize::<__S>::serialize(&self.b,
                                                                            serializer)?,
                                            _phantom:
                                                Serialize::<__S>::serialize(&self._phantom,
                                                                            serializer)?,})
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl <__D: Fallible + ?Sized> Deserialize<TestVersion0, __D> for
             Archived<TestVersion0> where i32: Archive,
             Archived<i32>: Deserialize<i32, __D>, i32: Archive,
             Archived<i32>: Deserialize<i32, __D>,
             ::core::marker::PhantomData<Test>: Archive,
             Archived<::core::marker::PhantomData<Test>>: Deserialize<::core::marker::PhantomData<Test>,
                                                                      __D> {
                #[inline]
                fn deserialize(&self, deserializer: &mut __D)
                 -> ::core::result::Result<TestVersion0, __D::Error> {
                    Ok(TestVersion0{a:
                                        Deserialize::<i32,
                                                      __D>::deserialize(&self.a,
                                                                        deserializer)?,
                                    b:
                                        Deserialize::<i32,
                                                      __D>::deserialize(&self.b,
                                                                        deserializer)?,
                                    _phantom:
                                        Deserialize::<::core::marker::PhantomData<Test>,
                                                      __D>::deserialize(&self._phantom,
                                                                        deserializer)?,})
                }
            }
        };
    impl TestVersion0 {
        pub fn new(a: i32, b: i32) -> Self {
            Self{a, b, _phantom: ::core::marker::PhantomData,}
        }
    }
    #[repr(C)]
    #[archive_attr(repr(C))]
    struct TestVersion1 {
        c: u32,
        d: u8,
        _phantom: ::core::marker::PhantomData<Test>,
    }
    #[automatically_derived]
    #[doc = "An archived `TestVersion1`"]
    #[repr(C,)]
    struct ArchivedTestVersion1 where u32: ::rkyv::Archive,
           u8: ::rkyv::Archive,
           ::core::marker::PhantomData<Test>: ::rkyv::Archive {
        #[doc = "The archived counterpart of `TestVersion1::c`"]
        c: ::rkyv::Archived<u32>,
        #[doc = "The archived counterpart of `TestVersion1::d`"]
        d: ::rkyv::Archived<u8>,
        #[doc = "The archived counterpart of `TestVersion1::_phantom`"]
        _phantom: ::rkyv::Archived<::core::marker::PhantomData<Test>>,
    }
    #[automatically_derived]
    #[doc = "The resolver for archived `TestVersion1`"]
    struct TestVersion1Resolver where u32: ::rkyv::Archive,
           u8: ::rkyv::Archive,
           ::core::marker::PhantomData<Test>: ::rkyv::Archive {
        c: ::rkyv::Resolver<u32>,
        d: ::rkyv::Resolver<u8>,
        _phantom: ::rkyv::Resolver<::core::marker::PhantomData<Test>>,
    }
    #[automatically_derived]
    const _: () =
        {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for TestVersion1 where u32: ::rkyv::Archive,
             u8: ::rkyv::Archive,
             ::core::marker::PhantomData<Test>: ::rkyv::Archive {
                type Archived = ArchivedTestVersion1;
                type Resolver = TestVersion1Resolver;
                #[allow(clippy :: unit_arg)]
                #[inline]
                unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver,
                                  out: *mut Self::Archived) {
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).c;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.c).resolve(pos + fp, resolver.c, fo);
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).d;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.d).resolve(pos + fp, resolver.d, fo);
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out)._phantom;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self._phantom).resolve(pos + fp, resolver._phantom, fo);
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl <__S: Fallible + ?Sized> Serialize<__S> for TestVersion1
             where u32: Serialize<__S>, u8: Serialize<__S>,
             ::core::marker::PhantomData<Test>: Serialize<__S> {
                #[inline]
                fn serialize(&self, serializer: &mut __S)
                 -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(TestVersion1Resolver{c:
                                                Serialize::<__S>::serialize(&self.c,
                                                                            serializer)?,
                                            d:
                                                Serialize::<__S>::serialize(&self.d,
                                                                            serializer)?,
                                            _phantom:
                                                Serialize::<__S>::serialize(&self._phantom,
                                                                            serializer)?,})
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl <__D: Fallible + ?Sized> Deserialize<TestVersion1, __D> for
             Archived<TestVersion1> where u32: Archive,
             Archived<u32>: Deserialize<u32, __D>, u8: Archive,
             Archived<u8>: Deserialize<u8, __D>,
             ::core::marker::PhantomData<Test>: Archive,
             Archived<::core::marker::PhantomData<Test>>: Deserialize<::core::marker::PhantomData<Test>,
                                                                      __D> {
                #[inline]
                fn deserialize(&self, deserializer: &mut __D)
                 -> ::core::result::Result<TestVersion1, __D::Error> {
                    Ok(TestVersion1{c:
                                        Deserialize::<u32,
                                                      __D>::deserialize(&self.c,
                                                                        deserializer)?,
                                    d:
                                        Deserialize::<u8,
                                                      __D>::deserialize(&self.d,
                                                                        deserializer)?,
                                    _phantom:
                                        Deserialize::<::core::marker::PhantomData<Test>,
                                                      __D>::deserialize(&self._phantom,
                                                                        deserializer)?,})
                }
            }
        };
    impl TestVersion1 {
        pub fn new(c: u32, d: u8) -> Self {
            Self{c, d, _phantom: ::core::marker::PhantomData,}
        }
    }
    #[repr(C)]
    #[archive_attr(repr(C))]
    struct Test {
        version_0: TestVersion0,
        version_1: TestVersion1,
    }
    #[automatically_derived]
    #[doc = "An archived `Test`"]
    #[repr(C,)]
    struct ArchivedTest where TestVersion0: ::rkyv::Archive,
           TestVersion1: ::rkyv::Archive {
        #[doc = "The archived counterpart of `Test::version_0`"]
        version_0: ::rkyv::Archived<TestVersion0>,
        #[doc = "The archived counterpart of `Test::version_1`"]
        version_1: ::rkyv::Archived<TestVersion1>,
    }
    #[automatically_derived]
    #[doc = "The resolver for archived `Test`"]
    struct TestResolver where TestVersion0: ::rkyv::Archive,
           TestVersion1: ::rkyv::Archive {
        version_0: ::rkyv::Resolver<TestVersion0>,
        version_1: ::rkyv::Resolver<TestVersion1>,
    }
    #[automatically_derived]
    const _: () =
        {
            use ::core::marker::PhantomData;
            use ::rkyv::{out_field, Archive, Archived};
            impl Archive for Test where TestVersion0: ::rkyv::Archive,
             TestVersion1: ::rkyv::Archive {
                type Archived = ArchivedTest;
                type Resolver = TestResolver;
                #[allow(clippy :: unit_arg)]
                #[inline]
                unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver,
                                  out: *mut Self::Archived) {
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).version_0;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.version_0).resolve(pos + fp, resolver.version_0,
                                              fo);
                    let (fp, fo) =
                        {

                            #[allow(unused_unsafe)]
                            unsafe {
                                let fo = &raw mut (*out).version_1;
                                (fo.cast::<u8>().offset_from(out.cast::<u8>())
                                     as usize, fo)
                            }
                        };
                    (&self.version_1).resolve(pos + fp, resolver.version_1,
                                              fo);
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Fallible, Serialize};
            impl <__S: Fallible + ?Sized> Serialize<__S> for Test where
             TestVersion0: Serialize<__S>, TestVersion1: Serialize<__S> {
                #[inline]
                fn serialize(&self, serializer: &mut __S)
                 -> ::core::result::Result<Self::Resolver, __S::Error> {
                    Ok(TestResolver{version_0:
                                        Serialize::<__S>::serialize(&self.version_0,
                                                                    serializer)?,
                                    version_1:
                                        Serialize::<__S>::serialize(&self.version_1,
                                                                    serializer)?,})
                }
            }
        };
    #[automatically_derived]
    const _: () =
        {
            use ::rkyv::{Archive, Archived, Deserialize, Fallible};
            impl <__D: Fallible + ?Sized> Deserialize<Test, __D> for
             Archived<Test> where TestVersion0: Archive,
             Archived<TestVersion0>: Deserialize<TestVersion0, __D>,
             TestVersion1: Archive,
             Archived<TestVersion1>: Deserialize<TestVersion1, __D> {
                #[inline]
                fn deserialize(&self, deserializer: &mut __D)
                 -> ::core::result::Result<Test, __D::Error> {
                    Ok(Test{version_0:
                                Deserialize::<TestVersion0,
                                              __D>::deserialize(&self.version_0,
                                                                deserializer)?,
                            version_1:
                                Deserialize::<TestVersion1,
                                              __D>::deserialize(&self.version_1,
                                                                deserializer)?,})
                }
            }
        };
    impl Test {
        #[inline]
        pub fn partial_v0(a: i32, b: i32) -> ::protoss::Partial<Self> {
            unsafe {
                let mut result = ::core::mem::MaybeUninit::<Self>::uninit();
                let result_ptr = result.as_mut_ptr();
                let version_ptr = &raw mut (*result_ptr).version_0;
                version_ptr.write(TestVersion0::new(a, b));
                let size =
                    version_ptr.cast::<u8>().offset_from(result_ptr.cast::<u8>())
                        as usize + ::core::mem::size_of::<TestVersion0>();
                ::protoss::Partial::new_unchecked(result, size)
            }
        }
        #[inline]
        pub fn partial_v1(a: i32, b: i32, c: u32, d: u8)
         -> ::protoss::Partial<Self> {
            unsafe {
                let mut result = ::core::mem::MaybeUninit::<Self>::uninit();
                let result_ptr = result.as_mut_ptr();
                let version_ptr = &raw mut (*result_ptr).version_0;
                version_ptr.write(TestVersion0::new(a, b));
                let version_ptr = &raw mut (*result_ptr).version_1;
                version_ptr.write(TestVersion1::new(c, d));
                let size =
                    version_ptr.cast::<u8>().offset_from(result_ptr.cast::<u8>())
                        as usize + ::core::mem::size_of::<TestVersion1>();
                ::protoss::Partial::new_unchecked(result, size)
            }
        }
    }
    unsafe impl ::protoss::Composite for Test {
        type Parts = TestParts;
    }
    #[repr(transparent)]
    struct TestParts {
        _phantom: ::core::marker::PhantomData<Test>,
        bytes: [u8],
    }
    const _: () =
        {
            use ptr_meta::Pointee;
            impl Pointee for TestParts where [u8]: Pointee {
                type Metadata = <[u8] as Pointee>::Metadata;
            }
        };
    impl Drop for TestParts {
        fn drop(&mut self) {
            unsafe {
                if let Some(version) = self.__version_0_mut() {
                    ::core::ptr::drop_in_place(version as *mut TestVersion0);
                } else { return; }
                if let Some(version) = self.__version_1_mut() {
                    ::core::ptr::drop_in_place(version as *mut TestVersion1);
                } else { return; }
            }
        }
    }
    impl TestParts {
        unsafe fn __version_0_unchecked(&self) -> &TestVersion0 {
            let struct_ptr = (self as *const Self).cast::<Test>();
            let field_ptr = &raw const (*struct_ptr).version_0;
            &*field_ptr
        }
        fn __version_0(&self) -> Option<&TestVersion0> {
            unsafe {
                let struct_ptr = (self as *const Self).cast::<Test>();
                let field_ptr = &raw const (*struct_ptr).version_0;
                let offset =
                    field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>())
                        as usize;
                let size = ::core::mem::size_of::<TestVersion0>();
                if offset + size > self.bytes.len() {
                    None
                } else { Some(&*field_ptr) }
            }
        }
        unsafe fn __version_0_mut_unchecked(&mut self) -> &mut TestVersion0 {
            let struct_ptr = (self as *mut Self).cast::<Test>();
            let field_ptr = &raw mut (*struct_ptr).version_0;
            &mut *field_ptr
        }
        fn __version_0_mut(&mut self) -> Option<&mut TestVersion0> {
            unsafe {
                let struct_ptr = (self as *mut Self).cast::<Test>();
                let field_ptr = &raw mut (*struct_ptr).version_0;
                let offset =
                    field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>())
                        as usize;
                let size = ::core::mem::size_of::<TestVersion0>();
                if offset + size > self.bytes.len() {
                    None
                } else { Some(&mut *field_ptr) }
            }
        }
        unsafe fn __version_1_unchecked(&self) -> &TestVersion1 {
            let struct_ptr = (self as *const Self).cast::<Test>();
            let field_ptr = &raw const (*struct_ptr).version_1;
            &*field_ptr
        }
        fn __version_1(&self) -> Option<&TestVersion1> {
            unsafe {
                let struct_ptr = (self as *const Self).cast::<Test>();
                let field_ptr = &raw const (*struct_ptr).version_1;
                let offset =
                    field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>())
                        as usize;
                let size = ::core::mem::size_of::<TestVersion1>();
                if offset + size > self.bytes.len() {
                    None
                } else { Some(&*field_ptr) }
            }
        }
        unsafe fn __version_1_mut_unchecked(&mut self) -> &mut TestVersion1 {
            let struct_ptr = (self as *mut Self).cast::<Test>();
            let field_ptr = &raw mut (*struct_ptr).version_1;
            &mut *field_ptr
        }
        fn __version_1_mut(&mut self) -> Option<&mut TestVersion1> {
            unsafe {
                let struct_ptr = (self as *mut Self).cast::<Test>();
                let field_ptr = &raw mut (*struct_ptr).version_1;
                let offset =
                    field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>())
                        as usize;
                let size = ::core::mem::size_of::<TestVersion1>();
                if offset + size > self.bytes.len() {
                    None
                } else { Some(&mut *field_ptr) }
            }
        }
        pub fn a(&self) -> Option<&i32> {
            self.__version_0().map(|version| &version.a)
        }
        pub fn a_mut(&mut self) -> Option<&mut i32> {
            self.__version_0_mut().map(|version| &mut version.a)
        }
        pub fn b(&self) -> Option<&i32> {
            self.__version_0().map(|version| &version.b)
        }
        pub fn b_mut(&mut self) -> Option<&mut i32> {
            self.__version_0_mut().map(|version| &mut version.b)
        }
        pub fn c(&self) -> Option<&u32> {
            self.__version_1().map(|version| &version.c)
        }
        pub fn c_mut(&mut self) -> Option<&mut u32> {
            self.__version_1_mut().map(|version| &mut version.c)
        }
        pub fn d(&self) -> Option<&u8> {
            self.__version_1().map(|version| &version.d)
        }
        pub fn d_mut(&mut self) -> Option<&mut u8> {
            self.__version_1_mut().map(|version| &mut version.d)
        }
    }
    #[repr(transparent)]
    struct ArchivedTestParts {
        _phantom: ::core::marker::PhantomData<::rkyv::Archived<Test>>,
        bytes: [u8],
    }
    const _: () =
        {
            use ptr_meta::Pointee;
            impl Pointee for ArchivedTestParts where [u8]: Pointee {
                type Metadata = <[u8] as Pointee>::Metadata;
            }
        };
    impl ::rkyv::ArchivePointee for ArchivedTestParts {
        type ArchivedMetadata = ::rkyv::Archived<usize>;
        fn pointer_metadata(archived: &Self::ArchivedMetadata) -> usize {
            {

                #[cfg(not(any(feature = "archive_le", feature =
                              "archive_be")))]
                { *archived }
            } as usize
        }
    }
    impl ::rkyv::ArchiveUnsized for TestParts {
        type Archived = ArchivedTestParts;
        type MetadataResolver = ();
        unsafe fn resolve_metadata(&self, pos: usize,
                                   resolver: Self::MetadataResolver,
                                   out: *mut ::rkyv::Archived<usize>) {
            const VERSION_0_SIZE: usize =
                ::core::mem::size_of::<TestVersion0>();
            const VERSION_1_SIZE: usize =
                ::core::mem::size_of::<TestVersion1>();
            let len =
                match self.bytes.len() {
                    VERSION_0_SIZE =>
                    ::core::mem::size_of::<::rkyv::Archived<TestVersion0>>(),
                    VERSION_1_SIZE =>
                    ::core::mem::size_of::<::rkyv::Archived<TestVersion1>>(),
                    _ => unsafe { ::core::hint::unreachable_unchecked() },
                };
            out.write({

                          #[cfg(not(any(feature = "archive_le", feature =
                                        "archive_be")))]
                          { len as ::rkyv::FixedUsize }
                      });
        }
    }
    impl <__S: ::rkyv::ser::Serializer + ?Sized> ::rkyv::SerializeUnsized<__S>
     for TestParts where TestVersion0: ::rkyv::Serialize<__S>,
     TestVersion1: ::rkyv::Serialize<__S> {
        fn serialize_unsized(&self, serializer: &mut __S)
         -> Result<usize, __S::Error> {
            const VERSION_0_SIZE: usize =
                ::core::mem::size_of::<TestVersion0>();
            const VERSION_1_SIZE: usize =
                ::core::mem::size_of::<TestVersion1>();
            match self.bytes.len() {
                VERSION_0_SIZE =>
                ::rkyv::SerializeUnsized::serialize_unsized(unsafe {
                                                                self.__version_0_unchecked()
                                                            }, serializer),
                VERSION_1_SIZE =>
                ::rkyv::SerializeUnsized::serialize_unsized(unsafe {
                                                                self.__version_1_unchecked()
                                                            }, serializer),
                _ => unsafe { ::core::hint::unreachable_unchecked() },
            }
        }
        fn serialize_metadata(&self, serializer: &mut __S)
         -> Result<(), __S::Error> {
            Ok(())
        }
    }
}
