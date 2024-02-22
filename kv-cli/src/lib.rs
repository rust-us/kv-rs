pub mod config;
pub mod progressbar;
pub mod emoji;
pub mod command;
pub mod npm;
pub mod child;

use clap::Parser;
use crate::progressbar::ProgressOutput;


/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();

#[derive(Debug, Parser)]
#[command(version)]
pub struct Cli {
    /// The subcommand to run.
    #[clap(subcommand)] // Note that we mark a field as a subcommand
    pub cmd: command::Command,

    #[clap(long = "quiet", short = 'q')]
    /// No output printed to stdout
    pub quiet: bool,
}