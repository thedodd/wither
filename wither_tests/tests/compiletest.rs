extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();
    config.mode = mode.parse().expect("Argument `mode` must be a valid FS path.");
    config.src_base = PathBuf::from(format!("tests/{}", mode));
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path.
    config.clean_rmeta(); // If your tests import the parent crate, this helps with E0464.

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode("compile-fail");
    // NOTE: any other top-level files in this directory will essentailly act as `run-pass` compiletests.
}
