use apicius::{grammar,types::State};

fn main() {
    for exp in std::fs::read_dir("tests").unwrap() {
        let exp = exp.unwrap().path().canonicalize().unwrap();
        let fname = exp.file_name().unwrap().to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".apicius") {
            let mut expected = exp.clone();
            expected.pop();
            expected.push(format!("{}.exp", prefix));

            let src = std::fs::read_to_string(exp).unwrap();
            let mut state = State::new();
            if let Ok(recipe) = grammar::RecipeParser::new().parse(&mut state, &src) {
                let mut f = std::fs::File::create(expected).unwrap();
                state.debug_recipe(&mut f, recipe).unwrap();
            }
        }
    }
}
