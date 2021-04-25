#![cfg_attr(not(feature = "std"), no_std)]

use core::{marker::PhantomData, ops::{Deref, DerefMut}, pin::Pin};
use ptr_meta::{Pointee, PtrExt};

#[cfg(feature = "rkyv")]
mod rkyv;

#[cfg(feature = "rkyv")]
pub use rkyv::*;

pub trait Fields {
    unsafe fn drop_in_place(&mut self);
}

pub unsafe trait Prefixed {
    type Fields: Fields + ?Sized;

    fn fields(bytes: &[u8]) -> &Self::Fields;
    fn fields_mut(bytes: &mut [u8]) -> &mut Self::Fields;
}

union Partial<T: Prefixed> {
    bytes: [u8; mem::size_of::<T>()],
    value: T,
}

pub struct Prefix<T: Prefixed> {
    partial: Partial<T>,
    size: usize,
}

impl<T: Prefixed> Prefix<T> {
    pub fn new(value: T) -> Self {
        Self {
            partial: Partial { value },
            size: mem::size_of::<T>(),
        }
    }
}

#[repr(transparent)]
#[derive(Pointee)]
pub struct Pre<T: Prefixed> {
    bytes: [u8],
    _phantom: PhantomData<T>,
}

impl<T: Prefixed> Pre<T> {
    #[cfg(feature = "std")]
    pub fn new(prefix: Prefix<T>) -> Box<Self> {
        unsafe {
            use std::alloc::{alloc, Layout};

            let ptr = alloc(Layout::new::<T>());
            ptr.cast::<T>().write(prefix.partial.value);
            let result = ptr_meta::from_raw_parts_mut(ptr.cast(), core::mem::size_of::<T>());
            Box::from_raw(result)
        }
    }

    pub unsafe fn cast<U: Prefixed>(&self) -> &Prefix<U> {
        let (data_address, metadata) = (self as *const Self).to_raw_parts();
        &*ptr_meta::from_raw_parts(data_address.cast(), metadata)
    }

    pub fn get(&self) -> Option<&T> {
        if self.bytes.len() >= core::mem::size_of::<T>() {
            unsafe { Some(self.get_unchecked()) }
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self) -> &T {
        &*self.bytes.as_ptr().cast::<T>()
    }

    pub fn get_pin(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        if self.bytes.len() >= core::mem::size_of::<T>() {
            unsafe { Some(self.get_pin_unchecked()) }
        } else {
            None
        }
    }

    pub unsafe fn get_pin_unchecked(self: Pin<&mut Self>) -> Pin<&mut T> {
        self.map_unchecked_mut(|s| &mut *s.bytes.as_mut_ptr().cast())
    }
}

impl<T: Prefixed> Drop for Prefix<T> {
    fn drop(&mut self) {
        unsafe {
            self.drop_in_place()
        }
    }
}

impl<T: Prefixed> Deref for Prefix<T> {
    type Target = T::Fields;

    fn deref(&self) -> &Self::Target {
        T::fields(&self.bytes)
    }
}

impl<T: Prefixed> DerefMut for Prefix<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        T::fields_mut(&mut self.bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fields, Prefix, Prefixed};

    macro_rules! impl_prefixed {
        (struct $name:ident($fields:ident) { $($field:ident: $ty:ty,)* }) => {
            #[repr(transparent)]
            pub struct $fields {
                bytes: [u8],
            }

            const _: () = {
                impl ptr_meta::Pointee for $fields {
                    type Metadata = usize;
                }

                impl $fields {
                    $(
                        pub fn $field(&self) -> Option<&$ty> {
                            let offset = memoffset::offset_of!($name, $field);
                            if offset + core::mem::size_of::<$ty>() <= self.bytes.len() {
                                Some(unsafe { &*self.bytes.as_ptr().add(offset).cast() })
                            } else {
                                None
                            }
                        }
                    )*
                }

                impl Fields for $fields {
                    unsafe fn drop_in_place(&mut self) {
                        $(
                            {
                                let offset = memoffset::offset_of!($name, $field);
                                if self.bytes.len() < offset + core::mem::size_of::<$ty>() {
                                    return;
                                }
                                self.bytes.as_mut_ptr().add(offset).cast::<$ty>().drop_in_place();
                            }
                        )*
                    }
                }

                unsafe impl Prefixed for $name {
                    type Fields = $fields;

                    fn fields(bytes: &[u8]) -> &Self::Fields {
                        unsafe { &*ptr_meta::from_raw_parts(bytes.as_ptr().cast(), bytes.len()) }
                    }

                    fn fields_mut(bytes: &mut [u8]) -> &mut Self::Fields {
                        unsafe { &mut *ptr_meta::from_raw_parts_mut(bytes.as_mut_ptr().cast(), bytes.len()) }
                    }
                }
            };
        }
    }

    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
    #[repr(C)]
    pub struct TestV0 {
        a: i32,
        b: String,
    }

    impl_prefixed! {
        struct TestV0(TestV0Accessor) {
            a: i32,
            b: String,
        }
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct TestV1 {
        a: i32,
        b: String,
        c: u32,
    }

    impl_prefixed! {
        struct TestV1(TestV1Accessor) {
            a: i32,
            b: String,
            c: u32,
        }
    }

    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub struct TestV2 {
        a: i32,
        b: String,
        c: u32,
        d: String,
    }

    impl_prefixed! {
        struct TestV2(TestV2Accessor) {
            a: i32,
            b: String,
            c: u32,
            d: String,
        }
    }

    #[test]
    fn basic_functionality() {
        let prefix = Prefix::new(TestV1 {
            a: 42,
            b: "hello world".into(),
            c: 100,
        });

        assert!(prefix.get().is_some());
        let value = prefix.get().unwrap();

        let as_v0 = unsafe { prefix.cast::<TestV0>() };
        assert_eq!(as_v0.get(), Some(&TestV0 { a: 42, b: "hello world".into() }));

        let as_v2 = unsafe { prefix.cast::<TestV2>() };
        assert_eq!(as_v2.get(), None);

        let access_v2 = &**as_v2;
        assert_eq!(access_v2.a(), Some(&value.a));
        assert_eq!(access_v2.b(), Some(&value.b));
        assert_eq!(access_v2.c(), Some(&value.c));
        assert_eq!(access_v2.d(), None);
    }
}
