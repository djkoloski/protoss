use crate::util::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Generics, Ident, ItemStruct, Meta, punctuated::Punctuated, parse_quote};

#[derive(Default)]
pub struct Settings {
    impl_rkyv: bool,
}

impl Settings {
    pub fn from_attr(attr: &Option<Meta>) -> Result<Self, Error> {
        let mut result = Self::default();

        if let Some(meta) = attr {
            match meta {
                Meta::Path(path) => {
                    if path.is_ident("rkyv") {
                        result.impl_rkyv = true;
                    } else {
                        return Err(Error::new_spanned(path, "unrecognized protoss argument"));
                    }
                }
                _ => return Err(Error::new_spanned(meta, "protoss arguments must be of the form `protoss(...)`")),
            }
        }

        Ok(result)
    }
}

pub fn generate(attr: &Option<Meta>, input: &ItemStruct) -> Result<TokenStream, Error> {
    let settings = Settings::from_attr(attr)?;

    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clause = where_clause.unwrap();

    let attrs = &input.attrs;

    let rkyv_args = settings.impl_rkyv.then(|| quote! { #[archive_attr(repr(C))] });

    let versions = collect_versions(&input.fields)?;

    let version_structs = versions.iter().map(|(version, fields)| {
        let struct_name = version_struct_name(name, *version);
        let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
        let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

        quote! {
            #[repr(C)]
            #(#attrs)*
            #rkyv_args
            #vis struct #struct_name #generics {
                #(#field_names: #field_types,)*
                _phantom: ::core::marker::PhantomData<#name #ty_generics>,
            }

            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn new(#(#field_names: #field_types,)*) -> Self {
                    Self {
                        #(#field_names,)*
                        _phantom: ::core::marker::PhantomData,
                    }
                }
            }
        }
    });

    let composite_fields = versions.iter().map(|(version, _)| {
        let struct_name = version_struct_name(name, *version);
        let field_name = version_field_name(*version);

        quote! {
            #field_name: #struct_name #ty_generics
        }
    });

    let partial_constructors = versions.iter().map(|(version, _)| {
        Ident::new(&format!("partial_v{}", version), Span::call_site())
    });

    let partial_args = (1..=versions.len()).map(|n| {
        let args = versions.iter().take(n).map(|(_, fields)| {
            let struct_args = fields.iter().map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                quote! { #name: #ty }
            });
            quote! {
                #(#struct_args,)*
            }
        });
        quote! {
            #(#args)*
        }
    });

    let write_versions = (1..=versions.len()).map(|n| {
        let initializers = versions.iter().take(n).map(|(version, fields)| {
            let version_struct = version_struct_name(name, *version);
            let version_args = fields.iter().map(|f| {
                let name = &f.ident;
                quote! { #name }
            });
            let version_field = version_field_name(*version);
            quote! {
                let version_ptr = ::core::ptr::addr_of_mut!((*result_ptr).#version_field);
                version_ptr.write(#version_struct::new(#(#version_args,)*));
            }
        });
        quote! {
            #(#initializers)*
        }
    });

    let version_struct = versions.iter().map(|(version, _)| version_struct_name(name, *version));

    let parts = parts_struct_name(name);

    let drop_versions = versions.iter().map(|(version, _)| {
        let version_accessor = version_accessor_mut(*version);
        let version_struct = version_struct_name(name, *version);

        quote! {
            if let Some(version) = self.#version_accessor() {
                ::core::ptr::drop_in_place(version as *mut #version_struct #ty_generics);
            } else {
                return;
            }
        }
    });

    let version_accessors = versions.iter().map(|(version, _)| {
        let version_accessor_unchecked = version_accessor_unchecked(*version);
        let version_accessor = version_accessor(*version);
        let version_accessor_mut_unchecked = version_accessor_mut_unchecked(*version);
        let version_accessor_mut = version_accessor_mut(*version);
        let version_struct = version_struct_name(name, *version);
        let version_field = version_field_name(*version);

        quote! {
            unsafe fn #version_accessor_unchecked(&self) -> &#version_struct #ty_generics {
                let struct_ptr = (self as *const Self).cast::<#name #ty_generics>();
                let field_ptr = ::core::ptr::addr_of!((*struct_ptr).#version_field);
                &*field_ptr
            }

            fn #version_accessor(&self) -> Option<&#version_struct #ty_generics> {
                unsafe {
                    let struct_ptr = (self as *const Self).cast::<#name #ty_generics>();
                    let field_ptr = ::core::ptr::addr_of!((*struct_ptr).#version_field);
                    let offset = field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>()) as usize;
                    let size = ::core::mem::size_of::<#version_struct #ty_generics>();
                    if offset + size > self.bytes.len() {
                        None
                    } else {
                        Some(&*field_ptr)
                    }
                }
            }

            unsafe fn #version_accessor_mut_unchecked(&mut self) -> &mut #version_struct #ty_generics {
                let struct_ptr = (self as *mut Self).cast::<#name #ty_generics>();
                let field_ptr = ::core::ptr::addr_of_mut!((*struct_ptr).#version_field);
                &mut *field_ptr
            }

            fn #version_accessor_mut(&mut self) -> Option<&mut #version_struct #ty_generics> {
                unsafe {
                    let struct_ptr = (self as *mut Self).cast::<#name #ty_generics>();
                    let field_ptr = ::core::ptr::addr_of_mut!((*struct_ptr).#version_field);
                    let offset = field_ptr.cast::<u8>().offset_from(struct_ptr.cast::<u8>()) as usize;
                    let size = ::core::mem::size_of::<#version_struct #ty_generics>();
                    if offset + size > self.bytes.len() {
                        None
                    } else {
                        Some(&mut *field_ptr)
                    }
                }
            }
        }
    });

    let field_accessors = versions.iter().map(|(version, fields)| {
        let version_accessor = version_accessor(*version);
        let version_accessor_mut = version_accessor_mut(*version);

        let result = fields.iter().map(|f| {
            let vis = &f.vis;
            let name = &f.ident.as_ref().unwrap();
            let name_mut = Ident::new(&format!("{}_mut", name), name.span());
            let ty = &f.ty;

            quote! {
                #vis fn #name(&self) -> Option<&#ty> {
                    self.#version_accessor().map(|version| &version.#name)
                }

                #vis fn #name_mut(&mut self) -> Option<&mut #ty> {
                    self.#version_accessor_mut().map(|version| &mut version.#name)
                }
            }
        });
        quote! { #(#result)* }
    });

    let rkyv_impl = settings.impl_rkyv.then(|| {
        let version_size_const = versions.iter()
            .map(|(version, _)| version_size_const(*version))
            .collect::<Vec<_>>();

        let version_size = versions.iter().map(|(version, _)| {
            let struct_name = version_struct_name(name, *version);
            quote! { ::core::mem::size_of::<#struct_name #ty_generics>() }
        }).collect::<Vec<_>>();

        let archived_version_size = versions.iter().map(|(version, _)| {
            let struct_name = version_struct_name(name, *version);
            quote! { ::core::mem::size_of::<::rkyv::Archived<#struct_name #ty_generics>>() }
        });

        let serialize_version = versions.iter().map(|(version, _)| {
            let version_accessor_unchecked = version_accessor_unchecked(*version);
            quote! {
                ::rkyv::SerializeUnsized::serialize_unsized(
                    unsafe { self.#version_accessor_unchecked() },
                    serializer,
                )
            }
        });

        let archived_parts = archived_parts_struct_name(name);

        let serialize_generics = {
            let mut serialize_where_clause = where_clause.clone();
            for (version, _) in versions.iter() {
                let struct_name = version_struct_name(name, *version);
                serialize_where_clause.predicates.push(parse_quote! { #struct_name #ty_generics: ::rkyv::Serialize<__S> })
            }

            let mut serialize_params = Punctuated::default();
            serialize_params.push(parse_quote! { __S: ::rkyv::ser::Serializer + ?Sized });
            for param in input.generics.params.iter() {
                serialize_params.push(param.clone());
            }

            Generics {
                lt_token: Some(Default::default()),
                params: serialize_params,
                gt_token: Some(Default::default()),
                where_clause: Some(serialize_where_clause),
            }
        };
        let (serialize_impl_generics, _, serialize_where_clause) = serialize_generics.split_for_impl();

        quote! {
            #[repr(transparent)]
            #[derive(::ptr_meta::Pointee)]
            #vis struct #archived_parts #generics {
                _phantom: ::core::marker::PhantomData<::rkyv::Archived<#name #ty_generics>>,
                bytes: [u8],
            }

            impl #impl_generics ::rkyv::ArchivePointee for #archived_parts #ty_generics {
                type ArchivedMetadata = ::rkyv::Archived<usize>;

                fn pointer_metadata(archived: &Self::ArchivedMetadata) -> usize {
                    ::rkyv::from_archived!(*archived) as usize
                }
            }

            impl #impl_generics ::rkyv::ArchiveUnsized for #parts #ty_generics {
                type Archived = #archived_parts #ty_generics;
                type MetadataResolver = ();

                unsafe fn resolve_metadata(
                    &self,
                    pos: usize,
                    resolver: Self::MetadataResolver,
                    out: *mut ::rkyv::Archived<usize>,
                ) {
                    #(const #version_size_const: usize = #version_size;)*
                    let len = match self.bytes.len() {
                        #(#version_size_const => #archived_version_size,)*
                        _ => unsafe { ::core::hint::unreachable_unchecked() },
                    };
                    out.write(::rkyv::to_archived!(len as ::rkyv::FixedUsize));
                }
            }

            impl #serialize_impl_generics ::rkyv::SerializeUnsized<__S> for #parts #ty_generics #serialize_where_clause {
                fn serialize_unsized(&self, serializer: &mut __S) -> Result<usize, __S::Error> {
                    #(const #version_size_const: usize = #version_size;)*
                    match self.bytes.len() {
                        #(#version_size_const => #serialize_version,)*
                        _ => unsafe { ::core::hint::unreachable_unchecked() },
                    }
                }

                fn serialize_metadata(&self, serializer: &mut __S) -> Result<(), __S::Error> {
                    Ok(())
                }
            }
        }
    });

    Ok(quote! {
        #(#version_structs)*

        #[repr(C)]
        #(#attrs)*
        #rkyv_args
        #vis struct #name #generics {
            #(#composite_fields,)*
        }

        impl #impl_generics #name #ty_generics {
            #(
                #[inline]
                pub fn #partial_constructors(#partial_args) -> ::protoss::Partial<Self> {
                    unsafe {
                        let mut result = ::core::mem::MaybeUninit::<Self>::uninit();
                        let result_ptr = result.as_mut_ptr();

                        #write_versions

                        let size = version_ptr.cast::<u8>().offset_from(result_ptr.cast::<u8>()) as usize
                            + ::core::mem::size_of::<#version_struct>();
                        ::protoss::Partial::new_unchecked(result, size)
                    }
                }
            )*
        }

        unsafe impl #impl_generics ::protoss::Composite for #name #ty_generics {
            type Parts = #parts #ty_generics;
        }

        #[repr(transparent)]
        #[derive(::ptr_meta::Pointee)]
        #vis struct #parts #generics {
            _phantom: ::core::marker::PhantomData<#name #ty_generics>,
            bytes: [u8],
        }

        impl #impl_generics Drop for #parts #ty_generics {
            fn drop(&mut self) {
                unsafe {
                    #(#drop_versions)*
                }
            }
        }

        impl #impl_generics #parts #ty_generics {
            #(#version_accessors)*

            #(#field_accessors)*
        }

        #rkyv_impl
    })
}
