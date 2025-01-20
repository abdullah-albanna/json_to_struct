# json2struct: Compile-Time Struct Generation

A procedural macro for generating Rust structs from JSON-like structures with extensive compile-time type safety and configuration options.

---

## Features

- **Automatic Struct Generation**: Create Rust structs from JSON-like syntax.
- **Flexible Type Inference**: Automatically infer types for fields.
- **Serde Integration**: Easily serialize and deserialize your structs.
- **Compile-Time Type Checking**: Catch errors during compilation.
- **Configurable with Flags**: Customize struct generation with powerful flags like `@debug`, `@snake`, `@derive`.

---

## Installation

Add this crate as a dependency in your `Cargo.toml`:

```toml
[dependencies]
json2struct = "0.1"
```

## Basic Usage

### Generate a simple struct from JSON-like syntax:

```rust
use json2struct::json2struct;

json2struct!(User {
    "first_name" => "John",
    "last_name" => "Doe",
    "age" => 30
});
```

#### Output

```rust

#[derive(Clone, Deserialize, Serialize)]
struct User {
    #[serde(alias = "first_name")]
    first_name: String,

    #[serde(alias = "last_name")]
    last_name: String,

    #[serde(alias = "age")]
    age: f64,
}
```

## Advanced Usage

### Customize your structs with flags:

```rust
json2struct!(Company @debug @camel @derive(PartialEq) @store_json {
    "company_name" => "Acme Corp",
    "employees" => [
        {
            "id" => 1,
            "details" => {
                "email" => "john@example.com",
                "department" => "Engineering"
            }
        }
    ]
});
```

#### Output

*This example generates nested structs, debug derives, and a static JSON value*

```rust
static COMPANY_JSON_VALUE: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(
        "{\"company_name\":\"Acme Corp\",\"employees\":[{\"details\":{\"department\":\"Engineering\",\"email\":\"john@example.com\"},\"id\":1.0}]}"
    ).expect("Couldn't convert the text into valid json")
});

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Company {
    #[serde(alias = "company_name")]
    company_name: String,


    #[serde(alias = "employees")]
    employees: Vec<Employee>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Employee {

    #[serde(alias = "id")]
    id: f64,

    #[serde(alias = "details")]
    details: Details,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Details {

    #[serde(alias = "email")]
    email: String,

    #[serde(alias = "department")]
    department: String,
}
```

## Supported Flags

| Flag            | Description                                   | Example                       |
|-----------------|-----------------------------------------------|-------------------------------|
| `@debug`        | Adds `Debug` derive                           | `@debug`                      |
| `@snake`        | Renames fields to `snake_case`                | `@snake`                      |
| `@camel`        | Renames fields to `camelCase`                 | `@camel`                      |
| `@pascal`       | Renames fields to `PascalCase`                | `@pascal`                     |
| `@derive(Type)` | Adds custom derives                           | `@derive(PartialEq, Clone)`   |
| `@store_json`   | Generates a static JSON value constant        | `@store_json`                 |



## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License.
