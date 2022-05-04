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
      .amt { color: #555; }
      .seasonings { color: #333; }
    </style>
";

const FOOTER: &str = "
  </body>
</html>
";

const SAMPLE: &str = "
soondubu jigae {
  [1/2] yellow onion
     -> dice
     -> cook 5m
     -> $chili
     -> cook 1m
     -> $zucchini
     -> stir &salt
     -> $kimchi
     -> simmer 2m
     -> $broth
     -> boil &salt
     -> $tofu
     -> cover with broth -> simmer
     -> $eggs
     -> cook 2m
     -> <>;
  [2 tbsp] chili paste -> $chili;
  [1] zucchini -> dice -> $zucchini;
  [1 cup] kimchi -> chop coarsely -> $kimchi;
  [2 cups] beef or chicken broth + [1 tsp] soy sauce
    -> $broth;
  [16oz] silken tofu -> $tofu;
  [3] eggs -> $eggs;
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

    println!("{:#?}", types::Printable {
        value: &tree,
        state: &s,
    });
    let table = apicius::render::table::TableGenerator::new(&s, &tree).compute();
    {
        use std::io::Write;
        let mut f = std::fs::File::create("samp.html").unwrap();
        write!(&mut f, "{}", HEADER).unwrap();
        write!(&mut f, "{}", table).unwrap();
        write!(&mut f, "{}", FOOTER).unwrap();
    }
}
