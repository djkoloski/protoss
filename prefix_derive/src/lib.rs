// extern crate proc_macro;

// use proc_macro2::{Span, TokenStream};
// use quote::{quote, quote_spanned};
// use syn::{parse_macro_input, DeriveInput};

// #[proc_macro_derive(Proto, attributes(id))]
// pub fn proto_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);

//     let proto_impl = derive_proto_impl(&input);

//     proc_macro::TokenStream::from(proto_impl)
// }

// fn derive_proto_impl(input: &DeriveInput) -> TokenStream { 
//     let name = &input.ident;
//     let vis = &input.vis;

//     let generic_params = input
//         .generics
//         .params
//         .iter()
//         .map(|p| quote_spanned! { p.span() => #p });
//     let generic_params = quote! { #(#generic_params,)* };

//     let generic_args = input.generics.type_params().map(|p| {
//         let name = &p.ident;
//         quote_spanned! { name.span() => #name }
//     });
//     let generic_args = quote! { #(#generic_args,)* };

//     let generic_predicates = match input.generics.where_clause {
//         Some(ref clause) => {
//             let predicates = clause.predicates.iter().map(|p| quote! { #p });
//             quote! { #(#predicates,)* }
//         }
//         None => quote! {},
//     };

//     let serialize_impl_generics = 

//     quote! {
//         #[repr(transparent)]
//         #vis struct #archived<#generic_params>
//         where
//             #generic_predicates
//         {
//             _phantom: PhantomData<(#generic_args)>,
//             bytes: [u8],
//         }

//         const _: () = {
//             use ptr_meta::Pointee;
//             use rkyv::{ArchivedMetadata, ArchivedUsize, Serialize, Serializer, SerializeUnsized};

//             impl<#generic_params> #archived<#generic_args> {
//                 #(#field_accessors)*
//                 #(#pin_field_accessors)*
//             }

//             #[repr(C)]
//             struct ArchivedData<#generic_params>
//             where
//                 #generic_predicates
//                 #archive_predicates
//             {
//                 #(#archived_fields,)*
//             }

//             impl<#generic_params> ArchivePointee for #archived<#generic_args>
//             where
//                 #generic_predicates
//             {
//                 type ArchivedMetadata = ArchivedUSize;

//                 fn pointer_metadata(archived: &Self::ArchivedMetadata) -> <Self as Pointee>::Metadata {
//                     archived as usize
//                 }
//             }

//             impl<#generic_params> ArchiveUnsized for #name<#generic_args>
//             where
//                 #generic_predicates
//                 #archive_predicates
//             {
//                 type Archived = #archived<#generic_args>;
//                 type MetadataResolver = ();

//                 fn resolve_metadata(&self, _: usize, _: Self::MetadataResolver) -> ArchivedMetadata<Self> {
//                     core::mem::size_of::<ArchivedData<#generic_args>>() as ArchiveUSize
//                 }
//             }

//             impl<__S: Serializer + ?Sized, #generic_params> SerializeUnsized<__S> for #name<#generic_args>
//             where
//                 #generic_predicates
//                 #serialize_predicates
//             {
//                 fn serialize_unsized(&self, serializer: &mut __S) -> Result<usize, __S::Error> {
//                     #(#serialize_fields)*
//                     let pos = serializer.align_for::<ArchivedData<#generic_args>>();
//                 }
//             }
//         };
//     }
// }
