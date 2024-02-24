#![feature(const_trait_impl)]

pub mod config;
pub mod progressbar;
pub mod emoji;
pub mod command;
pub mod session;
pub mod trace;
pub mod rusty;
pub mod new;
pub mod ast;
mod show;

use crate::progressbar::ProgressOutput;

/// The global progress bar and user-facing message output.
pub static PBAR: ProgressOutput = ProgressOutput::new();
