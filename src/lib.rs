//! # json2struct: Compile-Time Struct Generation
//!
//! A powerful procedural macro for generating Rust structs from JSON-like structures
//! with extensive compile-time type safety and configuration options.
//!
//! ## Features
//!
//! - Automatic struct generation from JSON-like syntax
//! - Flexible type inference
//! - Serde integration
//! - Compile-time type checking
//! - Multiple configuration flags
//!
//! ## Basic Usage
//!
//! ```rust
//! // Simple struct generation
//! json2struct!(User {
//!     "first_name" => "John",
//!     "last_name" => "Doe",
//!     "age" => 30
//! });
//! ```
//!
//! ### Output
//! ```rust
//! #[derive(Clone, Deserialize, Serialize)]
//! struct User {
//!   #[serde(alias = "first_name")]
//!   first_name: String,
//!
//!   #[serde(alias = "last_name")]
//!   last_name: String,
//!
//!   #[serde(alias = "age")]
//!   age: f64
//! }
//! ```
//!
//! ## Example with Flags
//!
//! ```rust
//! // Complex struct with multiple configurations
//! json2struct!(Company @debug @camel @derive(PartialEq) @store_json {
//!     "company_name" => "Acme Corp",
//!     "employees" => [
//!         {
//!             "id" => 1,
//!             "details" => {
//!                 "email" => "john@example.com",
//!                 "department" => "Engineering"
//!             }
//!         }
//!     ]
//! });
//! ```
//!
//! ### Output
//!
//! ```rust
//!
//!
//! static COMPANY_JSON_VALUE: LazyLock<Value> = LazyLock::new(||
//!
//! {
//!   ::serde_json::from_str(
//!           "{\"company_name\":\"Acme Corp\",\"employees\":[{\"details\":{\"department\":\"Engineering\",\"email\":\"john@example.
//! com\"},\"id\":1.0}]}",
//!        )
//!        .expect("Couldn't convert the text into valid json")
//!   });
//!
//! #[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
//! #[serde(rename_all = "camelCase")]
//! struct Company {
//!   #[serde(alias = "company_name")]
//!   company_name: String,
//!
//!   #[serde(alias = "last_name")]
//!   employees: CompanyEmplyees
//! }
//!
//! #[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
//! #[serde(rename_all = "camelCase")]
//! struct CompanyEmplyees {
//!   #[serde(alias = "id")]
//!   id: f64,
//!
//!   #[serde(alias = "details")]
//!   details:  
//! }
//!
//! #[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
//! #[serde(rename_all = "camelCase")]
//! struct CompanyEmplyeesDetails {
//!   #[serde(alias = "email")]
//!   email: String,
//!
//!   #[serde(alias = "department")]
//!   department: String
//! }
//!
//! ```
//!
//! ## Supported Flags
//!
//! | Flag            | Description                                   | Example                       |
//! |-----------------|-----------------------------------------------|-------------------------------|
//! | `@debug`        | Adds Debug derive                             | `@debug`                      |
//! | `@snake`        | Renames fields to snake_case                  | `@snake`                      |
//! | `@camel`        | Renames fields to camelCase                   | `@camel`                      |
//! | `@pascal`       | Renames fields to pascal                      | `@pascal`                     |
//! | `@derive(Type)` | Adds custom derives                           | `@derive(PartialEq, Clone)`   |
//! | `@store_json`   | Generates a static JSON Value constant        | `@store_json`                 |
//!

extern crate proc_macro;

mod generator;
mod parser;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

/// json2struct: Generates Rust structs from JSON-like structures
///
/// # Macro Syntax
///
/// ```rust
/// json2struct!(StructName [flags] {
///     "key" => value,
///     ...
/// })
/// ```
///
/// # Supported Value Types
/// - Strings: `"value"`
/// - Numbers: `42`, `3.14`
/// - Booleans: `true`, `false`
/// - Null: `null`
/// - Objects: `{ ... }`
/// - Arrays: `[ ... ]`
///
/// # Examples
///
/// Basic Struct:
/// ```rust
/// json2struct!(User {
///     "name" => "John",
///     "age" => 30
/// });
/// ```
///
/// Nested Struct:
/// ```rust
/// json2struct!(Company @debug {
///     "name" => "Acme",
///     "address" => {
///         "street" => "123 Main St",
///         "city" => "Anytown"
///     }
/// });
/// ```
///
/// # Performance
/// - Zero-cost abstraction
/// - Compile-time struct generation
/// - No runtime overhead
///
/// # Errors
/// Compilation will fail if:
/// - Invalid JSON structure
/// - Unsupported types
/// - Conflicting flags
#[proc_macro]
pub fn json2struct(input: TokenStream) -> TokenStream {
    // Parse the input into our custom macro input structure
    let json_struct = parse_macro_input!(input as parser::JsonMacroInput);

    // Initialize output token stream
    let mut output = proc_macro2::TokenStream::new();

    // Optionally generate a static JSON value constant
    if json_struct.flags.store_json_value {
        // Convert entries to serde_json::Value
        let serde_value = serde_json::Value::Object(
            json_struct
                .content
                .entries
                .clone()
                .into_iter()
                .map(|(k, v)| (k, v.to_serde_value()))
                .collect(),
        );

        // Convert to string for lazy initialization
        let serde_value_str = serde_json::to_string(&serde_value).unwrap_or_default();

        // Generate a constant name based on struct name
        let const_json_ident = format_ident!(
            "{}_{}",
            json_struct.struct_name.to_string().to_uppercase(),
            "JSON_VALUE"
        );

        // Generate lazy-loaded static JSON value
        output.extend(quote! {
            static #const_json_ident: ::std::sync::LazyLock<::serde_json::Value> =
                ::std::sync::LazyLock::new(||
                    ::serde_json::from_str(#serde_value_str)
                        .expect("Couldn't convert the text into valid json")
                );
        });
    }

    // Generate the main struct and any nested structs
    let (main_struct, all_structs) =
        generator::generate_structs(&json_struct, &json_struct.struct_name);

    // Combine all generated code
    output.extend(quote! {
        #main_struct
        #(#all_structs)*
    });

    // Convert to TokenStream for the compiler
    output.into()
}
