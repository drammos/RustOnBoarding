use std::fs;

use graph_ql_server::openapi;

const OPENAPI_FILENAME: &str = "api-docs/openapi.json";

fn main() {
    fs::create_dir_all("api-docs").expect("Failed to create dir `api-docs`");

    let api_docs = openapi::gen_openapi();

    let json = api_docs
        .to_pretty_json()
        .expect("Failed to generate json from OpenApi");
    fs::write(OPENAPI_FILENAME, &json).expect("Failed to write OpenApi docs to file");

    println!("Saved OpenApi docs to `{}`", OPENAPI_FILENAME);
}
