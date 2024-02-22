#![allow(unused)]

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, StderrLock, StdoutLock, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, panic, thread};
use std::time::Duration;
use anyhow::{Context, Result};
use clap::Parser;
use signal_hook::{consts::SIGINT, iterator::Signals};
use human_panic::setup_panic;
use kvcli::{Cli, PBAR};
use kvcli::command::run_pack;
use kvcli::config::ConfigLoad;


/// CMD like:
///     kv-cli --quiet ==>  Cli { quiet: true }
///
fn main() {
    setup_panic_hooks();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);

        for cause in e.chain() {
            eprintln!("Caused by: {}", cause);
        }
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cfg: ConfigLoad = confy::load_path("config")?;
    println!("cfg {:?}", cfg);

    let args = Cli::parse();
    println!("Hello, world! {:?}", args);

    if args.quiet {
        PBAR.set_quiet(true);
    }

    run_pack(args.cmd)?;

    Ok(())
}

fn setup_panic_hooks() {
    let meta = human_panic::Metadata {
        version: env!("CARGO_PKG_VERSION").into(),
        name: env!("CARGO_PKG_NAME").into(),
        authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
        homepage: env!("CARGO_PKG_HOMEPAGE").into(),
    };

    let default_hook = panic::take_hook();

    if let Err(_) = env::var("RUST_BACKTRACE") {
        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            // First call the default hook that prints to standard error.
            default_hook(info);

            // Then call human_panic.
            let file_path = human_panic::handle_dump(&meta, info);
            human_panic::print_msg(file_path, &meta)
                .expect("human-panic: printing error message to console failed");
        }));
    }
}
