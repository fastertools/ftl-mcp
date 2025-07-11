use schemars::{schema_for, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct EchoInput {
    message: String,
    delay: Option<u64>,
}

fn main() {
    let schema = schema_for\!(EchoInput);
    println\!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
