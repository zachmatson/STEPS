use quote::quote;
use syn::parse_macro_input;

mod codegen;
mod parsing;

/// Create types which serve as an interface between Rust and JavaScript code by generating Rust
/// structs and TypeScript interfaces, along with extern types in Rust which will translate to
/// the proper TypeScript types in wasm bidngen
#[proc_macro]
pub fn create_typescript_interface(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let definitions = parse_macro_input!(tokens as parsing::InterfaceTypeDefinitions).definitions;

    let typescript_section = codegen::create_typescript_section(&definitions);
    let extern_section = codegen::create_extern_section(&definitions);
    let rust_structs = codegen::create_rust_structs(&definitions);

    (quote! {
        #typescript_section
        #extern_section
        #rust_structs
    })
    .into()
}
