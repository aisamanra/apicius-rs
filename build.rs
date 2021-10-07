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
            let mut expected = exp.clone();
            expected.pop();
            expected.push(format!("{}.exp", prefix));

            writeln!(test_file, "// writing test for {}", fname).unwrap();
            writeln!(test_file, "#[test]").unwrap();
            writeln!(test_file, "fn test_{}() {{", prefix).unwrap();
            writeln!(test_file, "  let source = include_str!({:?});", exp.as_path()).unwrap();
            writeln!(test_file, "  let mut s = State::new();").unwrap();
            writeln!(test_file, "  let recipe = grammar::RecipeParser::new().parse(&mut s, source);").unwrap();
            writeln!(test_file, "  assert!(recipe.is_ok());").unwrap();
            if expected.exists() {
                writeln!(test_file, "  let exp = std::fs::read_to_string({:?}).unwrap();", expected.as_path()).unwrap();
                writeln!(test_file, "  let mut buf = Vec::new();").unwrap();
                writeln!(test_file, "  s.debug_recipe(&mut buf, recipe.unwrap()).unwrap();").unwrap();
                writeln!(test_file, "  assert_eq!(std::str::from_utf8(&buf).unwrap().trim(), exp.trim());").unwrap();
            }
            writeln!(test_file, "}}").unwrap();
        }
    }
}
