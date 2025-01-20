use inflections::Inflect;
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

use crate::parser::{JsonMacroInput, JsonStruct, JsonValue};

/// Generates Rust structs from a JSON-like structure with flexible configuration.
///
/// # Parameters
/// - `json_struct`: The input JSON macro structure
/// - `base_name`: The base name for the primary struct
///
/// # Returns
/// A tuple containing:
/// 1. The main generated struct as a token stream
/// 2. A vector of additional nested structs
pub fn generate_structs(
    json_struct: &JsonMacroInput,
    base_name: &Ident,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    // Collect all generated structs
    let mut all_structs = Vec::new();
    let mut fields = Vec::new();

    // Determine base derives
    //
    // usually clone is needed for json, so by default, it get's derived
    let mut derives = vec![quote!(::std::clone::Clone)];

    // Conditionally add derives based on flags
    //
    // not really need to be a seprate flag, but it's nice to have a quick way to do so
    if json_struct.flags.debug {
        derives.push(quote!(::std::fmt::Debug));
    }

    // Collected from the `@derive(...)`
    derives.extend(json_struct.flags.custom_derives.iter().map(|d| quote!(#d)));

    // Process each entry in the JSON-like structure
    for (key, value) in &json_struct.content.entries {
        // Just in case the identifier is not a valid struct name
        let field_name = format_ident!("{}", sanitize_identifier(key));

        // Infer field type and handle nested structures
        let (field_type, _) = match value {
            JsonValue::Str(_) => (quote!(String), Vec::<proc_macro2::TokenStream>::new()),
            JsonValue::Number(_) => (quote!(f64), Vec::new()),
            JsonValue::Boolean(_) => (quote!(bool), Vec::new()),

            JsonValue::Array(arr) => {
                let (elem_type, _) = infer_array_type(arr);
                (quote!(Vec<#elem_type>), Vec::new())
            }

            JsonValue::Object(obj) => {
                // Generate nested struct for object and concat the key with the struct name
                //
                // `Example`
                //
                //```rust
                //
                // struct User {
                //  age: UserAge
                // }
                //
                // struct UserAge;
                //
                //````
                let nested_name = format_ident!("{}{}", base_name, key.to_pascal_case());

                let json_content = JsonStruct {
                    entries: obj.clone(),
                };

                let nested_macro_input = JsonMacroInput {
                    struct_name: json_struct.struct_name.clone(),
                    flags: json_struct.flags.clone(),
                    content: json_content,
                };

                // Recursively generate nested structs
                let (nested_struct, nested_structs) =
                    generate_structs(&nested_macro_input, &nested_name);

                all_structs.extend(nested_structs);
                all_structs.push(nested_struct.clone());

                (
                    format_ident!("{}", nested_name).into_token_stream(),
                    Vec::new(),
                )
            }
            JsonValue::Null => (quote!(Option<::serde_json::Value>), Vec::new()),
        };

        // Handle Serde alias configuration
        //
        // this is usefull when serializing, and when also specifing the @camel|pascal|snake flags
        //
        // if you have a json that's formatted like so
        //
        // ```json
        // {
        //   "name": "Abdullah",
        //   "jobs_list": ["Cybersecurity"]
        // }
        // ```
        //
        // the keys are written in snake_case,
        // which means if you have a sruct that you want to deserialize to which has an attribte that looks like this
        //
        // ```rust
        // #[derive(Deserialize, Serialize)]
        // #[serde(rename_all = "camelCase")]
        // struct User {
        //   name: String,
        //   jobs_list: Vec<String>
        // }
        // ```
        //
        // this will only deserialize if you give it a camelCase keys, not snake_case
        //
        // this is where the `#[serde(alias = "jobs_list")]` comes in, it allows you to have both,
        // so you can deserialize with camelCase and snake_case
        let field = if json_struct.flags.use_serde_alias {
            quote! {
                #[serde(alias = #key)]
                #field_name: #field_type
            }
        } else {
            quote! {
                #field_name: #field_type
            }
        };

        fields.push(field);
    }

    // Prepare struct name and rename strategy
    let struct_name = base_name;
    let style = json_struct
        .clone()
        .flags
        .rename_all
        .map(|style| Some(style.to_string()));

    // Generate the main struct with optional rename strategy
    let main_struct = if let Some(rename_all_style) = style {
        quote! {
            #[derive(#(#derives),*, ::serde::Deserialize, ::serde::Serialize)]
            #[serde(rename_all = #rename_all_style)]
            struct #struct_name {
                #(#fields),*
            }
        }
    } else {
        quote! {
            #[derive(#(#derives),*, ::serde::Deserialize, ::serde::Serialize)]
            struct #struct_name {
                #(#fields),*
            }
        }
    };

    (main_struct, all_structs)
}

/// Infers the element type for an array of JSON values.
///
/// # Parameters
/// - `arr`: A slice of JSON values
///
/// # Returns
/// A tuple containing:
/// 1. The inferred element type as a token stream
/// 2. Any additional generated structs (currently unused)
fn infer_array_type(
    arr: &[JsonValue],
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    // Handle empty array
    if arr.is_empty() {
        return (quote!(::serde_json::Value), Vec::new());
    }

    // Infer type based on first element
    match &arr[0] {
        JsonValue::Str(_) => (quote!(String), Vec::new()),
        JsonValue::Number(_) => (quote!(f64), Vec::new()),
        JsonValue::Boolean(_) => (quote!(bool), Vec::new()),
        _ => (quote!(::serde_json::Value), Vec::new()),
    }
}

/// Sanitizes a string to create a valid Rust identifier.
///
/// # Parameters
/// - `name`: The input string to sanitize
///
/// # Returns
/// A sanitized, lowercase string suitable for use as a Rust identifier
fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
}
