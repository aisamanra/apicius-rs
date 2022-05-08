use apicius::types::ToPrintable;
use apicius::{checks, grammar, render, types};

mod opts;

fn main() {
    if let Err(err) = realmain() {
        println!("Error when running `apicius`: {}", err);
    }
}

fn realmain() -> Result<(), Box<dyn std::error::Error>> {
    let opts = opts::Opts::parse();

    let input = opts.get_input()?;
    let mut output = opts.get_output()?;

    let mut s = types::State::new();
    // TODO: convert these errors
    let recipe = grammar::RecipeParser::new().parse(&mut s, &input).unwrap();

    if let opts::ApiciusCommand::DebugParseTree = opts.command {
        s.debug_recipe(&mut output, &recipe)?;
        return Ok(());
    }

    let analysis = checks::Analysis::from_recipe(&s, &recipe);

    if let opts::ApiciusCommand::DebugAnalysis = opts.command {
        writeln!(output, "{:#?}", analysis.printable(&s))?;
        return Ok(());
    }

    let tree = analysis.into_tree()?;

    if let opts::ApiciusCommand::DebugBackwardTree = opts.command {
        writeln!(output, "{:#?}", tree.printable(&s))?;
        return Ok(());
    }

    if opts.command.is_table_command() {
        let table = render::table::Table::new(&s, &tree);

        if let opts::ApiciusCommand::DebugTable = opts.command {
            writeln!(output, "{}", table.debug())?;
            return Ok(());
        }

        if let opts::ApiciusCommand::HTMLTable(opts) = opts.command {
            if opts.standalone {
                writeln!(output, "{}", opts.html_header)?;
            }

            writeln!(output, "{}", table.html(&opts))?;

            if opts.standalone {
                writeln!(output, "{}", opts.html_footer)?;
            }

            return Ok(());
        }
    }

    Ok(())
}
