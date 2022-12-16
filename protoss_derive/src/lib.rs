//! Procedural macros for `protoss`.
//! 
//! \*\* ***NOTE: THIS INFORMATION IS CURRENTLY OUT OF DATE AND ALSO UNIMPLEMENTED.***
//! 
//! Two main macros drive this system: `#[derive(protoss::Evolving)]` and `#[protoss::previous_evolution_of]`.
//! 
//! When an `Evolving` type has only one major version (0), we only need the `#[derive(Evolving)]` macro:
//! 
//! ```rust,ignore
//! #[derive(Evolving)]
//! #[evolving(major_version = 0)]
//! #[evolving(minor_version = 1)]
//! pub struct Test {
//!     #[field(id = 0, since_minor_version = 0)]
//!     pub a: u32,
//!     #[field(id = 1, since_minor_version = 0)]
//!     pub b: u8,
//!     #[field(id = 2, since_minor_version = 1)]
//!     pub c: u32,
//!     #[field(id = 3, since_minor_version = 2)]
//!     pub d: u8,
//! }
//! ```
//! 
//! This would generate:
//! - Concrete, padded version structs for each minor version `ArchivedTestV0_0`, `ArchivedTestV0_1` and `ArchivedTestV0_2`
//! - A concrete probe type, `TestProbeMajor0`
//! - implementations for relevant traits: `impl Evolving for Test`, `impl VersionOf<Test> for ArchivedTestV0_X`, and `impl ProbeOf<Test> for TestProbeMajor0`
//! - and then also `impl rkyv::{Archived, Serialize, Deserialize} for Test` in terms of these types, using `protoss::ArchivedEvolution`
//! 
//! When an `Evolving` type has more than one major version, we also need to define each major version
//! separately by linking back to the original using the `#[protoss::previous_evolution_of]` macro:
//! 
//! ```rust,ignore
//! #[derive(Evolving)]
//! #[evolving(current_major_version = 1)]
//! #[evolving(current_minor_version = 0)]
//! pub struct Test {
//!     #[field(id = 0, since_minor_version = 0)]
//!     pub a: u32,
//!     #[field(id = 1, since_minor_version = 0)]
//!     pub b: u32,
//! }
//!
//! #[protoss::previous_evolution_of(evolving_ty = Test, major_version = 0)]
//! pub struct TestMajor0 {
//!     #[field(id = 0, since_minor_version = 0)]
//!     pub a: u32,
//!     #[field(id = 1, since_minor_version = 0)]
//!     pub b: u8,
//!     #[field(id = 2, since_minor_version = 1)]
//!     pub c: u32,
//!     #[field(id = 3, since_minor_version = 2)]
//!     pub d: u8,
//! }
//! ```
//! 
//! In this case, the `#[derive(Evolving)]` macro would also generate the following trait which the
//! user will need to implement for `Test`:
//! 
//! ```rust,ignore
//! trait DefEvolutionOfTest {
//!     fn upgrade_major_0_to_major_1(major_0: &TestProbeMajor0) -> ArchivedTestV1_0;
//! }
//! ```
//! 
//! The `#[derive(Evolving)]` macro would now generat the same things as previous but only for the major version 1,
//! while the `#[protoss::previous_evolution_of]` macro will generate all the code that the `#[derive(Evolving)]` macro
//! previously did for major version 0, but in terms of still using `Test` as the base `Evolving` type (and therefore not
//! directly `impl Evolving for Test`, rather just creating all the necessary types and impls for the original macro to be able to use).
//! 

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(missing_docs)]

mod composite;
mod util;

extern crate proc_macro;

use syn::{ItemStruct, Meta, parse_macro_input};

/// legacy, ignore for now
#[proc_macro_attribute]
pub fn protoss(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attr = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as Meta))
    };

    let mut input = parse_macro_input!(item as ItemStruct);
    input.generics.make_where_clause();

    match composite::generate(&attr, &input) {
        Ok(result) => result.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
