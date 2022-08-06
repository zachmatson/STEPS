use quote::quote;
use syn::{Ident, LitStr};

use crate::parsing::{InterfaceTypeDefinition, InterfaceTypeField};

/// Create the tokens used to make wasm bindgen export `str` consts corresponding to the
/// `InterfaceTypeDefinition`s to an exported TypeScript file
pub fn create_typescript_section(
    definitions: &[InterfaceTypeDefinition],
) -> proc_macro2::TokenStream {
    let definition_string_literals = definitions.iter().map(|def| {
        LitStr::new(
            &create_typescript_interface_string(def),
            def.name_rust.span(),
        )
    });

    quote! {
        #[automatically_derived]
        const _: () = {
            use ::wasm_bindgen::prelude::*;
            #(
                #[wasm_bindgen(typescript_custom_section)]
                const _: &'static str = #definition_string_literals;
            )*
        };
    }
}

/// Create a string for a TypeScript interface definition corresponding to an
/// `InterfaceTypeDefinition`
fn create_typescript_interface_string(definition: &InterfaceTypeDefinition) -> String {
    format!(
        "export interface {} {{\n{}}}",
        get_or_infer_name_js(definition),
        definition
            .fields
            .iter()
            .map(create_typescript_field_string)
            .collect::<String>()
    )
}

/// Create a string for a single TypeScript interface field definition corresponding to an
/// `InterfaceTypeField`
fn create_typescript_field_string(field: &InterfaceTypeField) -> String {
    format!(
        "  {}{}: {};\n",
        field.name.to_string(),
        if field.optional { "?" } else { "" },
        field.type_js.to_string()
    )
}

/// Create the full section of extern type definitions corresponding to a set of
/// `InterfaceTypeDefinition`s
pub fn create_extern_section(definitions: &[InterfaceTypeDefinition]) -> proc_macro2::TokenStream {
    let declarations = definitions.iter().map(create_extern_type_declaration);

    quote! {
        #[automatically_derived]
        #[wasm_bindgen]
        extern "C" {
            #(#declarations)*
        }
    }
}

/// Create the tokens for an extern type definition with WASM annotations that will link the
/// generated TypeScript interface to a usable JsValue type in Rust
fn create_extern_type_declaration(
    definition: &InterfaceTypeDefinition,
) -> proc_macro2::TokenStream {
    let name = get_or_infer_name_js(definition);
    let name_str = LitStr::new(&name.to_string(), name.span());

    quote! {
        #[wasm_bindgen(typescript_type = #name_str)]
        pub type #name;
    }
}

/// Create Rust structs for several `InterfaceTypeDefinition`s
pub fn create_rust_structs(definitions: &[InterfaceTypeDefinition]) -> proc_macro2::TokenStream {
    definitions.iter().map(create_rust_struct).collect()
}

/// Create the tokens for a Rust struct definition corresponding to an `InterfaceTypeDefinition`
fn create_rust_struct(definition: &InterfaceTypeDefinition) -> proc_macro2::TokenStream {
    let name = &definition.name_rust;
    let fields = definition.fields.iter().map(create_rust_field);

    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        pub struct #name {
            #(#fields),*
        }
    }
}

/// Create the tokens for a Rust struct field definition corresponding to an `InterfaceTypeField`
fn create_rust_field(field: &InterfaceTypeField) -> proc_macro2::TokenStream {
    let InterfaceTypeField {
        name,
        type_rust,
        optional,
        ..
    } = field;

    let field_type = if *optional {
        quote! { ::std::option::Option<#type_rust> }
    } else {
        quote! { #type_rust }
    };

    quote! {
        pub #name: #field_type
    }
}

/// Get the JavaScript type name from an `InterfaceTypeDefinition`, using either the name
/// provided in the macro, or by adapting the provided Rust name
fn get_or_infer_name_js(definition: &InterfaceTypeDefinition) -> Ident {
    definition.name_js.to_owned().unwrap_or_else(|| {
        Ident::new(
            &format!("Js{}", definition.name_rust.to_string()),
            definition.name_rust.span(),
        )
    })
}
