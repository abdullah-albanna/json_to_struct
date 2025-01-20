use syn::{
    braced,
    parse::{Parse, ParseStream},
    Ident, Lit, Result, Token,
};

#[derive(Debug, Default, Clone)]
pub struct JsonMacroFlags {
    pub debug: bool,
    pub rename_all: Option<RenameStyle>,
    pub store_json_value: bool,
    pub use_serde_alias: bool,
    pub custom_derives: Vec<Ident>,
}

#[derive(Debug, Clone)]
pub enum RenameStyle {
    Camel,
    Snake,
    Pascal,
}

impl std::fmt::Display for RenameStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenameStyle::Camel => f.write_str("camelCase"),
            RenameStyle::Snake => f.write_str("snake_case"),
            RenameStyle::Pascal => f.write_str("PascalCase"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonMacroInput {
    pub struct_name: Ident,
    pub flags: JsonMacroFlags,
    pub content: JsonStruct,
}

impl Parse for JsonMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Require struct name
        let struct_name = input.parse::<Ident>()?;
        let mut flags = JsonMacroFlags::default();

        while input.peek(Token![@]) {
            input.parse::<Token![@]>()?;

            let flag_ident = input.parse::<Ident>()?;

            let flag_name = flag_ident.to_string();

            match flag_name.as_str() {
                "debug" => flags.debug = true,
                "store_json" => flags.store_json_value = true,
                "no_alias" => flags.use_serde_alias = false,
                "camel" => flags.rename_all = Some(RenameStyle::Camel),
                "snake" => flags.rename_all = Some(RenameStyle::Snake),
                "pascal" => flags.rename_all = Some(RenameStyle::Pascal),
                "derive" => {
                    // Parse custom derives
                    if input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in input);

                        let derives = content.parse_terminated(Ident::parse, Token![,])?;
                        flags.custom_derives.extend(derives);
                    } else {
                        return Err(syn::Error::new(flag_ident.span(), "expected @derive(...)"));
                    }
                }

                _ => {
                    let message = format!("Unknown flag: {} Supported flags: @debug @camel @snake @pascal @store_json @no_alias @derive(...)", flag_name);
                    return Err(input.error(&message));
                }
            }
        }

        // Parse the struct content

        let content;

        braced!(content in input);

        let json_struct = JsonStruct::parse(&content)?;

        Ok(JsonMacroInput {
            struct_name,
            flags,
            content: json_struct,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum JsonValue {
    Str(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[allow(dead_code)]
impl JsonValue {
    // Add methods to actually use the fields
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Vec<(String, JsonValue)>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    // Convert to serde_json::Value
    pub fn to_serde_value(&self) -> serde_json::Value {
        match self {
            JsonValue::Str(s) => serde_json::Value::String(s.clone()),
            JsonValue::Number(n) => serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap_or(serde_json::Number::from(0)),
            ),
            JsonValue::Boolean(b) => serde_json::Value::Bool(*b),
            JsonValue::Null => serde_json::Value::Null,
            JsonValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_serde_value()).collect())
            }
            JsonValue::Object(obj) => {
                let map = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_serde_value()))
                    .collect();
                serde_json::Value::Object(map)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct JsonStruct {
    pub entries: Vec<(String, JsonValue)>,
}

impl Parse for JsonStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);

        let mut entries = Vec::new();

        while !content.is_empty() {
            // Parse key (string)
            let key: Lit = content.parse()?;
            let key_str = match key {
                Lit::Str(s) => s.value(),
                _ => return Err(content.error("Key must be a string")),
            };

            // Parse arrow
            content.parse::<Token![:]>()?;

            // Parse value
            let value = parse_json_value(&content)?;

            entries.push((key_str, value));

            // Optional comma
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(JsonStruct { entries })
    }
}

pub fn parse_json_value(input: ParseStream) -> Result<JsonValue> {
    if input.peek(syn::token::Bracket) {
        // Parse array
        let content;
        syn::bracketed!(content in input);

        let mut array = Vec::new();
        while !content.is_empty() {
            let value = parse_json_value(&content)?;
            array.push(value);

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        return Ok(JsonValue::Array(array));
    }

    if input.peek(syn::token::Brace) {
        // Parse nested object
        let nested: JsonStruct = input.parse()?;
        return Ok(JsonValue::Object(nested.entries));
    }

    // Parse literal values
    let lit: Lit = input.parse()?;
    match lit {
        Lit::Str(s) => Ok(JsonValue::Str(s.value())),
        Lit::Int(i) => Ok(JsonValue::Number(i.base10_parse::<f64>()?)),
        Lit::Float(f) => Ok(JsonValue::Number(f.base10_parse::<f64>()?)),
        Lit::Bool(b) => Ok(JsonValue::Boolean(b.value)),
        _ => Err(input.error("Unsupported literal type")),
    }
}
