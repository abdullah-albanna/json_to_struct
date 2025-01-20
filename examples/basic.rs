use json2struct::json2struct;

json2struct!(User @camel @store_json @debug {{
    "ainfo": 0,
    "arr": ["sd"],
    "asas": {
        "s": {
            "d": "23"
        }
    },
    "extra": "r"
}});

fn main() {
    let json = (*USER_JSON_VALUE).clone();

    println!("{}", serde_json::to_string_pretty(&json).unwrap());

    let constructured_struct: User = serde_json::from_value(json).unwrap();

    println!("{:#?}", constructured_struct);
}
