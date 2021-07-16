//! Procedural macros for `protoss`.

#![deny(broken_intra_doc_links)]
#![deny(missing_docs)]
#![deny(missing_crate_level_docs)]

mod composite;
mod util;

extern crate proc_macro;

use syn::{ItemStruct, parse_macro_input};

/// Generates a composite struct and parts based on the annotated struct.
#[proc_macro]
pub fn protoss(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    input.generics.make_where_clause();

    match composite::generate(&input) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
