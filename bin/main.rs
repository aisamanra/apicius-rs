use apicius::{checks, grammar, types};

const SAMPLE: &str = "
nicer scrambled eggs {
  [1/2] onion + [1 clove] garlic
    -> chop coarsely -> sautee & butter -> $mix;
  [2] eggs -> whisk -> $mix;
  $mix -> stir & salt -> <>;
}
";

fn main() {
    let mut s = types::State::new();
    let recipe = grammar::RecipeParser::new().parse(&mut s, SAMPLE);
    println!("{:?}", recipe);
    assert!(recipe.is_ok());
    let recipe = recipe.unwrap();
    assert!(checks::to_tree(&s, &recipe).is_ok());
    s.debug_recipe(&mut std::io::stdout(), &recipe).unwrap();
}
