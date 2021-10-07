use std::env;
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
    lalrpop::process_root().unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("exp_tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    writeln!(test_file, "use crate::types::*;").unwrap();
    writeln!(test_file, "use crate::grammar;").unwrap();
    for exp in std::fs::read_dir("tests").unwrap() {
        let exp = exp.unwrap().path().canonicalize().unwrap();
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            writeln!(test_file, "// writing test for {}", fname).unwrap();
            writeln!(test_file, "#[test]").unwrap();
            writeln!(test_file, "fn test_{}() {{", prefix).unwrap();
            writeln!(test_file, "  let source = include_str!({:?});", exp.as_path()).unwrap();
            writeln!(test_file, "  let mut s = State::new();").unwrap();
            writeln!(test_file, "  let recipe = grammar::RecipeParser::new().parse(&mut s, source);").unwrap();
            writeln!(test_file, "  assert!(recipe.is_ok());").unwrap();
            writeln!(test_file, "}}").unwrap();
        }
    }
}
