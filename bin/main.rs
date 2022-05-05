use apicius::types::ToPrintable;
use apicius::{checks, grammar, render, types};

use clap::{Parser, Subcommand};

use std::io;
use std::io::Read;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Opts {
    #[clap(subcommand)]
    command: Command,

    #[clap(short, long)]
    input: Option<String>,

    #[clap(short, long)]
    output: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Command {
    HTMLTable {
        #[clap(short, long)]
        standalone: bool,
    },
    DebugParseTree,
    DebugAnalysis,
    DebugBackwardTree,
    DebugTable,
}

impl Command {
    fn is_table_command(&self) -> bool {
        match self {
            Command::HTMLTable { .. } => true,
            Command::DebugTable => true,
            _ => false,
        }
    }
}

fn get_input(input: Option<String>) -> io::Result<String> {
    let path = if let Some(path) = input {
        path
    } else {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    };
    if path == "-" {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }

    std::fs::read_to_string(path)
}

fn get_output(output: Option<String>) -> io::Result<Box<dyn io::Write>> {
    let path = if let Some(path) = output {
        path
    } else {
        return Ok(Box::new(io::stdout()));
    };
    if path == "-" {
        return Ok(Box::new(io::stdout()));
    }

    let f = std::fs::File::create(path)?;
    Ok(Box::new(f))
}

fn main() {
    if let Err(err) = realmain() {
        println!("Error when running `apicius`: {}", err);
    }
}

fn realmain() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let input = get_input(opts.input)?;
    let mut output = get_output(opts.output)?;

    let mut s = types::State::new();
    // TODO: convert these errors
    let recipe = grammar::RecipeParser::new().parse(&mut s, &input).unwrap();

    if let Command::DebugParseTree = opts.command {
        s.debug_recipe(&mut output, &recipe)?;
        return Ok(());
    }

    let analysis = checks::Analysis::from_recipe(&s, &recipe);

    if let Command::DebugAnalysis = opts.command {
        writeln!(output, "{:#?}", analysis.printable(&s))?;
        return Ok(());
    }

    let tree = analysis.into_tree()?;

    if let Command::DebugBackwardTree = opts.command {
        writeln!(output, "{:#?}", tree.printable(&s))?;
        return Ok(());
    }

    if opts.command.is_table_command() {
        let table = render::table::Table::new(&s, &tree);

        if let Command::DebugTable = opts.command {
            writeln!(output, "{}", table.debug())?;
            return Ok(());
        }

        if let Command::HTMLTable { standalone } = opts.command {
            if standalone {
                writeln!(output, "{}", render::constants::STANDALONE_HTML_HEADER)?;
            }

            writeln!(output, "{}", table.html())?;

            if standalone {
                writeln!(output, "{}", render::constants::STANDALONE_HTML_FOOTER)?;
            }

            return Ok(());
        }
    }

    Ok(())
}
