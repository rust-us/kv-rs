pub mod config;
pub mod progressbar;
pub mod emoji;
pub mod command;
pub mod session;
pub mod trace;
pub mod rusty;
pub mod new;

use clap::Parser;
use crate::progressbar::ProgressOutput;


/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();

#[derive(Debug, Parser, PartialEq)]
#[command(version)]
// disable default help flag since it would conflict with --host
#[command(author, about, disable_help_flag = true)]
pub struct Args {
    #[clap(long, help = "Print help information")]
    pub help: bool,

    /// The subcommand to run.
    #[clap(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: Option<command::Command>,

    #[clap(long = "quiet", short = 'q')]
    /// No output printed to stdout
    pub quiet: Option<bool>,

    #[clap(short = 'l', default_value = "info", long)]
    pub log_level: String,

    #[clap(short = 'n', long, help = "Force non-interactive mode")]
    pub non_interactive: bool,

    #[clap(long, require_equals = true, help = "Query to execute")]
    pub query: Option<String>,
}