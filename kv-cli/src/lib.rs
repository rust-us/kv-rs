pub mod config;
pub mod progressbar;
pub mod emoji;
pub mod command;
pub mod session;
pub mod trace;
pub mod rusty;
pub mod new;

use crate::progressbar::ProgressOutput;

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();
