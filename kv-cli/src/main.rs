#![allow(unused)]

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, StderrLock, stdin, StdoutLock, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, panic, thread};
use std::time::Duration;
use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use signal_hook::{consts::SIGINT, iterator::Signals};
use human_panic::setup_panic;
use log::info;
use kvcli::{Args, PBAR, session, trace};
use kvcli::command::{Command, run_pack};
use kvcli::config::ConfigLoad;


/// CMD like:
///     kv-cli         ==>  Cli { quiet: false }
///     kv-cli --quiet ==>  Cli { quiet: true }
///
#[tokio::main]
pub async fn main() -> Result<()> {
    setup_panic_hooks();

    eprintln!();
    eprintln!("██  ██  █        █");
    eprintln!("██ ██   ██      ██");
    eprintln!("███      ██    ██");
    eprintln!("██ ██    ██  ██");
    eprintln!("██  ██     ████");
    eprintln!();

    let mut cfg: ConfigLoad = confy::load_path("config")?;
    println!("Config {:?}", &cfg);

    let args = Args::parse();
    println!("Args{:?}", args);
    eprintln!();

    let mut cmd = Args::command();
    if args.help {
        cmd.print_help()?;
        return Ok(());
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    if args.quiet.is_some() {
        PBAR.set_quiet(true);
    }
    let is_terminal = stdin().is_terminal();
    let is_repl = is_terminal && !args.non_interactive && args.query.is_none();
    if is_repl {
        cfg.terminal_update();
    }

    let mut session = session::Session::try_new(cfg, true, running.clone()).await?;

    let log_dir = format!(
        "{}/.kvcli",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    let _guards = trace::init_logging(&log_dir, &args.log_level).await?;

    if is_repl {
        session.handle_repl().await;
        return Ok(());
    }

    match args.query {
        None => {
            session.handle_reader(stdin().lock()).await?;
        },
        Some(query) => {
            session.handle_reader(std::io::Cursor::new(query)).await?;
        }
    }

    info!("Prepare Running run_pack");
    run_pack(args.cmd.unwrap())?;

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
