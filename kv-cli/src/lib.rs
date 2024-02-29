#![feature(const_trait_impl)]

//! `kv-rs` CLI Tools. [Author fengyang]
//!
//! ## Getting started
//!
//! ```doc
//! ❯ ./kvcli
//!
//! ██  ██  █        █
//! ██ ██   ██      ██
//! ███      ██    ██
//! ██ ██     ██  ██
//! ██  ██     ████  KV Storage CLI
//!
//! Welcome to kvcli.
//! Connecting to Client.
//!
//!
//! kvcli > SET order_key xxx
//! OK ~
//!
//! kvcli > keys
//! order_key
//!
//! kvcli > ksize
//! 1
//!
//! kvcli > GET order_key
//! xxx
//!
//! kvcli > DEL order_key
//! OK ~
//!
//! kvcli > GET order_key
//! N/A ~
//! ```

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
