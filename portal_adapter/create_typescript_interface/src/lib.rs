use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, LitStr, Type};

mod parse;

struct InterfaceTypeDefinitions {
    definitions: Vec<InterfaceTypeDefinition>,
}

struct InterfaceTypeDefinition {
    pub name_rust: Ident,
    pub name_js: Option<Ident>,
    pub fields: Vec<InterfaceTypeField>,
}

struct InterfaceTypeField {
    pub name: Ident,
    pub type_rust: Type,
    pub type_js: TokenStream,
    pub optional: bool,
}

#[proc_macro]
pub fn create_typescript_interface(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let definitions = parse_macro_input!(tokens as InterfaceTypeDefinitions).definitions;

    let typescript_section = create_typescript_section(&definitions);
    let extern_section = create_extern_section(&definitions);
    let rust_structs = create_rust_structs(&definitions);

    (quote! {
        #typescript_section
        #extern_section
        #rust_structs
    })
    .into()
}

fn create_typescript_section(definitions: &[InterfaceTypeDefinition]) -> proc_macro2::TokenStream {
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

fn create_typescript_interface_string(definition: &InterfaceTypeDefinition) -> String {
    format!(
        "export interface {} {{\n{}}};",
        get_or_infer_name_js(definition),
        definition
            .fields
            .iter()
            .map(create_typescript_field_string)
            .collect::<String>()
    )
}

fn create_typescript_field_string(field: &InterfaceTypeField) -> String {
    format!(
        "  {}{}: {};\n",
        field.name.to_string(),
        if field.optional { "?" } else { "" },
        field.type_js.to_string()
    )
}

fn create_extern_section(definitions: &[InterfaceTypeDefinition]) -> proc_macro2::TokenStream {
    let declarations = definitions.iter().map(create_extern_type_declaration);

    quote! {
        #[automatically_derived]
        #[wasm_bindgen]
        extern "C" {
            #(#declarations)*
        }
    }
}

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

fn create_rust_structs(definitions: &[InterfaceTypeDefinition]) -> proc_macro2::TokenStream {
    definitions.iter().map(create_rust_struct).collect()
}

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

fn get_or_infer_name_js(definition: &InterfaceTypeDefinition) -> Ident {
    definition.name_js.to_owned().unwrap_or_else(|| {
        Ident::new(
            &format!("Js{}", definition.name_rust.to_string()),
            definition.name_rust.span(),
        )
    })
}
