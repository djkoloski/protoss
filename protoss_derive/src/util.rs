use std::collections::HashMap;
use proc_macro2::Span;
use syn::{Attribute, Error, Field, Fields, Ident, Lit, Meta};

pub fn parse_version(attr: &Attribute) -> Result<usize, Error> {
    let meta = attr.parse_meta()?;
    match meta {
        Meta::NameValue(name_value) => match name_value.lit {
            Lit::Int(int) => Ok(int.base10_parse()?),
            _ => Err(Error::new_spanned(name_value, "version attribute must be an integer")),
        }
        _ => Err(Error::new_spanned(attr, "version attribute must be of the form `#[version = n]`")),
    }
}

pub fn collect_versions(fields: &Fields) -> Result<Vec<(usize, Vec<&Field>)>, Error> {
    let mut version_to_fields = HashMap::new();
    match fields {
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
        _ => return Err(Error::new_spanned(fields, "protoss may only be used on structs with named fields")),
    };

    let mut versions = version_to_fields.drain().collect::<Vec<_>>();
    versions.sort_by_key(|(v, _)| *v);
    Ok(versions)
}

pub fn version_struct_name(name: &Ident, version: usize) -> Ident {
    Ident::new(&format!("{}Version{}", name, version), name.span())
}

pub fn version_field_name(version: usize) -> Ident {
    Ident::new(&format!("version_{}", version), Span::call_site())
}

pub fn parts_struct_name(name: &Ident) -> Ident {
    Ident::new(&format!("{}Parts", name), name.span())
}

pub fn version_accessor(version: usize) -> Ident {
    Ident::new(&format!("__version_{}", version), Span::call_site())
}

pub fn version_accessor_mut(version: usize) -> Ident {
    Ident::new(&format!("__version_{}_mut", version), Span::call_site())
}
