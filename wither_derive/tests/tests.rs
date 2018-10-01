// extern crate compiletest_rs as compiletest;

// use std::path::PathBuf;

// fn run_mode(mode: &'static str) {
//     let mut config = compiletest::Config::default();
//     config.mode = mode.parse().expect("Argument `mode` must be a valid FS path.");
//     config.src_base = PathBuf::from(format!("tests/{}", mode));
//     config.link_deps(); // Populate config.target_rustcflags with dependencies on the path.
//     config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464.

//     compiletest::run_tests(&config);
// }

// #[test]
// fn compile_test() {
//     run_mode("compile-fail");
//     run_mode("run-pass");
// }


extern crate bson;
extern crate compiletest_rs as compiletest;
extern crate mongodb;
extern crate serde;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate serde_json;
extern crate wither;
#[macro_use(Model)]
extern crate wither_derive;

use wither::Model;
use mongodb::coll::options::IndexModel;

#[derive(Serialize, Deserialize, Model)]
#[model(collection_name="valid_data_models_0", skip_serde_checks="false")]
struct ValidDataModel0 {
    /// The ID of the model.
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    id: Option<bson::oid::ObjectId>,

    /// A field to test base line index options & bool fields with `true`.
    #[model(index(
        direction="asc",
        background="true", sparse="true", unique="true",
        expire_after_seconds="15", name="field0", storage_engine="wt", version="1", default_language="en_us",
        language_override="en_us", text_version="1", sphere_version="1", bits="1", max="10.0", min="1.0", bucket_size="1",
    ))]
    field0: String,

    /// A field to test bool fields with `false`.
    #[model(index(
        direction="dsc",
        background="false", sparse="false", unique="false",
    ))]
    field1: String,

    /// A field to test `weights` option.
    #[model(index(direction="dsc", /*weights=""*/))] // TODO: ensure weights are compiling correctly.
    field2: String,
}

fn main() {}
