#![feature(const_trait_impl)]

pub mod progressbar;
pub mod emoji;
pub mod command;
pub mod trace;
pub mod rusty;
pub mod new;
pub mod ast;
pub mod show;
pub mod server;

use crate::progressbar::ProgressOutput;

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();
