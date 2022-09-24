use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, Error, Ident, Token, Type};

/// Series of type definitions
pub struct InterfaceTypeDefinitions {
    pub definitions: Vec<InterfaceTypeDefinition>,
}

/// Definition for a single interface type which will have corresponding types generated in Rust
/// and TypeScript, and an extern type created in Rust which translates to the proper type in
/// wasm bindgen
pub struct InterfaceTypeDefinition {
    pub name_rust: Ident,
    pub name_js: Option<Ident>,
    pub fields: Vec<InterfaceTypeField>,
}

/// Single field of an interface type definition
pub struct InterfaceTypeField {
    pub name: Ident,
    pub type_rust: Type,
    pub type_js: proc_macro2::TokenStream,
    pub optional: bool,
}

/// Error message for issues parsing the struct name(s)
const STRUCT_NAME_ERROR_MSG: &'static str =
    "Expected either the format `RustName` or `RustName, JavaScriptName`";

impl Parse for InterfaceTypeDefinitions {
    fn parse(input: ParseStream) -> Result<Self> {
        let definitions = std::iter::from_fn(|| {
            (!input.is_empty()).then(|| input.parse::<InterfaceTypeDefinition>())
        })
        .collect::<Result<Vec<_>>>()?;

        Ok(InterfaceTypeDefinitions { definitions })
    }
}

impl Parse for InterfaceTypeDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let name_idents = Punctuated::<Ident, Token![,]>::parse_separated_nonempty(&input)?;
        if name_idents.len() > 2 {
            return Err(Error::new_spanned(name_idents, STRUCT_NAME_ERROR_MSG));
        }
        // Must be non empty with length <= 2, so 1 or 2
        let mut name_idents = name_idents.into_iter();
        let name_rust = name_idents.next().unwrap();
        let name_js = name_idents.next();

        let fields = extract_interface_type_fields(input);

        Ok(InterfaceTypeDefinition {
            name_rust: name_rust,
            name_js: name_js,
            fields: fields?,
        })
    }
}

impl Parse for InterfaceTypeField {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;

        let optional = input.parse::<Token![?]>().is_ok();
        input.parse::<Token![:]>()?;

        let types;
        parenthesized!(types in input);
        let type_rust = types.parse::<Type>()?;
        types.parse::<Token![,]>()?;
        let type_js = types.parse()?;

        Ok(InterfaceTypeField {
            name,
            type_rust,
            type_js,
            optional,
        })
    }
}

/// Parse series of interface type fields inside braces
fn extract_interface_type_fields(input: ParseStream) -> Result<Vec<InterfaceTypeField>> {
    let inner;
    braced!(inner in input);
    Ok(
        Punctuated::<InterfaceTypeField, Token![,]>::parse_terminated(&inner)?
            .into_iter()
            .collect(),
    )
}
