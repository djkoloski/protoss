use std::collections::HashMap;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Error, Fields, Ident, ItemStruct, Lit, Meta};

fn parse_version(attr: &Attribute) -> Result<usize, Error> {
    let meta = attr.parse_meta()?;
    match meta {
        Meta::NameValue(name_value) => match name_value.lit {
            Lit::Int(int) => Ok(int.base10_parse()?),
            _ => Err(Error::new_spanned(name_value, "version attribute must be an integer")),
        }
        _ => Err(Error::new_spanned(attr, "version attribute must be of the form `#[version = n]`")),
    }
}

fn version_struct_name(name: &Ident, version: usize) -> Ident {
    Ident::new(&format!("{}Version{}", name, version), name.span())
}

fn version_field_name(version: usize) -> Ident {
    Ident::new(&format!("version_{}", version), Span::call_site())
}

fn parts_struct_name(name: &Ident) -> Ident {
    Ident::new(&format!("{}Parts", name), name.span())
}

fn version_accessor(version: usize) -> Ident {
    Ident::new(&format!("__version_{}", version), Span::call_site())
}

fn version_accessor_mut(version: usize) -> Ident {
    Ident::new(&format!("__version_{}_mut", version), Span::call_site())
}

pub fn generate(mut input: ItemStruct) -> Result<TokenStream, Error> {
    input.generics.make_where_clause();

    let name = &input.ident;
    let vis = &input.vis;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clause = where_clause.unwrap();

    let mut version_to_fields = HashMap::new();
    match input.fields {
        Fields::Named(ref fields) => {
            let mut last_version = None;
            for field in fields.named.iter() {
                let version_attrs = field.attrs.iter()
                    .filter(|a| a.path.is_ident("version"))
                    .map(parse_version)
                    .collect::<Result<Vec<_>, _>>()?;
                let version = match version_attrs.len() {
                    0 => last_version.ok_or_else(|| Error::new_spanned(field, "field is not associated with a version"))?,
                    1 => version_attrs[0],
                    _ => return Err(Error::new_spanned(field, "field is associated with multiple versions")),
                };
                last_version = Some(version);
                let fields = version_to_fields.entry(version).or_insert(Vec::new());
                fields.push(field);
            }
        },
        _ => return Err(Error::new_spanned(input, "protoss")),
    };

    let mut versions = version_to_fields.drain().collect::<Vec<_>>();
    versions.sort_by_key(|(v, _)| *v);

    let version_structs = versions.iter().map(|(version, fields)| {
        let struct_name = version_struct_name(name, *version);
        let field_names = fields.iter().map(|f| &f.ident).collect::<Vec<_>>();
        let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

        quote! {
            // TODO: #[repr(C)] if `strict`
            // TODO: #[derive(Archive, Deserialize, Serialize)]
            struct #struct_name #generics {
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
        let version_accessor = version_accessor(*version);
        let version_accessor_mut = version_accessor_mut(*version);
        let version_struct = version_struct_name(name, *version);
        let version_field = version_field_name(*version);

        quote! {
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

    Ok(quote! {
        #(#version_structs)*

        #[repr(C)]
        // TODO: #[derive(Archive, Deserialize, Serialize)]
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
    })
}
