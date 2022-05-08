use apicius::render::table::HTMLTableOptions;

use clap::{arg, command, ArgMatches, Command};

use std::io;
use std::io::Read;

#[derive(Debug)]
pub struct Opts {
    pub command: ApiciusCommand,
    pub input: Option<String>,
    pub output: Option<String>,
}

impl Opts {
    fn subcommand(name: &str) -> Command {
        Command::new(name).arg(arg!([INPUT])).arg(arg!([OUTPUT]))
    }

    fn handle_subcommand(
        cmd: ApiciusCommand,
        opts: &ArgMatches,
    ) -> (ApiciusCommand, Option<String>, Option<String>) {
        (
            cmd,
            opts.value_of("INPUT").map(|s| s.to_string()),
            opts.value_of("OUTPUT").map(|s| s.to_string()),
        )
    }

    pub fn parse() -> Opts {
        let matches = command!()
            .propagate_version(true)
            .subcommand_required(true)
            .subcommand(Opts::subcommand("debug-parse-tree").about("Print the raw parse tree"))
            .subcommand(Opts::subcommand("debug-analysis").about("Print the analysis output"))
            .subcommand(
                Opts::subcommand("debug-backward-tree").about("Print the generated backward tree"),
            )
            .subcommand(Opts::subcommand("debug-table").about("Print the raw table layout info"))
            .subcommand(
                Opts::subcommand("html-table")
                    .about("Convert the recipe to an HTML table")
                    .arg(arg!(--standalone).required(false))
                    .arg(arg!(--html_header <HTML_HEADER>).required(false))
                    .arg(arg!(--html_footer <HTML_FOOTER>).required(false))
                    .arg(arg!(--amount_class <AMOUNT_CLASS>).required(false))
                    .arg(arg!(--seasonings_class <SEASONINGS_CLASS>).required(false))
                    .arg(arg!(--ingredient_class <INGREDIENT_CLASS>).required(false))
                    .arg(arg!(--action_class <ACTION_CLASS>).required(false))
                    .arg(arg!(--done_class <DONE_CLASS>).required(false)),
            )
            .get_matches();
        let (command, input, output) = match matches.subcommand() {
            // the basic debug ones
            Some(("debug-parse-tree", opts)) => {
                Opts::handle_subcommand(ApiciusCommand::DebugParseTree, opts)
            }
            Some(("debug-analysis", opts)) => {
                Opts::handle_subcommand(ApiciusCommand::DebugAnalysis, opts)
            }
            Some(("debug-backward-tree", opts)) => {
                Opts::handle_subcommand(ApiciusCommand::DebugBackwardTree, opts)
            }
            Some(("debug-table", opts)) => {
                Opts::handle_subcommand(ApiciusCommand::DebugTable, opts)
            }
            // table plus table options
            Some(("html-table", opts)) => {
                let mut html_options = HTMLTableOptions::default();
                html_options.standalone = opts.is_present("standalone");

                if let Some(s) = opts.value_of("html_header") {
                    html_options.html_header = s.to_string();
                }
                if let Some(s) = opts.value_of("html_footer") {
                    html_options.html_footer = s.to_string();
                }

                if let Some(s) = opts.value_of("amount_class") {
                    html_options.amount_class = s.to_string();
                }
                if let Some(s) = opts.value_of("seasonings_class") {
                    html_options.seasonings_class = s.to_string();
                }
                if let Some(s) = opts.value_of("ingredient_class") {
                    html_options.ingredient_class = s.to_string();
                }
                if let Some(s) = opts.value_of("action_class") {
                    html_options.action_class = s.to_string();
                }
                if let Some(s) = opts.value_of("done_class") {
                    html_options.done_class = s.to_string();
                }
                Opts::handle_subcommand(ApiciusCommand::HTMLTable(html_options), opts)
            }
            _ => unreachable!("Unhandled subcommand"),
        };

        Opts {
            command,
            input,
            output,
        }
    }

    pub fn get_input(&self) -> io::Result<String> {
        let path = if let Some(path) = &self.input {
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

    pub fn get_output(&self) -> io::Result<Box<dyn io::Write>> {
        let path = if let Some(path) = &self.output {
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
}

#[derive(Debug)]
pub enum ApiciusCommand {
    HTMLTable(HTMLTableOptions),
    DebugParseTree,
    DebugAnalysis,
    DebugBackwardTree,
    DebugTable,
}

impl ApiciusCommand {
    pub fn is_table_command(&self) -> bool {
        matches!(
            self,
            ApiciusCommand::HTMLTable { .. } | ApiciusCommand::DebugTable
        )
    }
}
