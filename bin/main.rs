use apicius::{checks, grammar, types};

const HEADER: &str = "
<!DOCTYPE html>
<html>
  <body>
    <style type=\"text/css\">
      body { font-family: \"Fira Sans\", arial; }
      td {
        padding: 1em;
      }
      table, td, tr {
        border: 2px solid;
        border-spacing: 0px;
      }
      .ingredient {
        background-color: #ddd;
      }
      .done {
        background-color: #555;
      }
    </style>
";

const FOOTER: &str = "
  </body>
</html>
";

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
    s.debug_recipe(&mut std::io::stdout(), &recipe).unwrap();
    let analysis = checks::Analysis::from_recipe(&s, &recipe);
    let tree = analysis.into_tree().unwrap();
    tree.debug_raw(&mut std::io::stdout(), &s).unwrap();
    let table = apicius::render::table::TableGenerator::new(&s, &tree).compute();
    {
        use std::io::Write;
        let mut f = std::fs::File::create("samp.html").unwrap();
        write!(&mut f, "{}", HEADER).unwrap();
        write!(&mut f, "{}", table).unwrap();
        write!(&mut f, "{}", FOOTER).unwrap();
    }
}
