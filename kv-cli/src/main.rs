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
use human_panic::setup_panic;
use log::info;
use kvcli::{command, PBAR, trace};
use kvcli::command::{Command, run_pack};
use kvcli::server::config::{ConfigLoad};
use kvcli::server::session;

#[derive(Debug, Parser, PartialEq)]
#[command(version)]
// disable default help flag since it would conflict with --host
#[command(author, about, disable_help_flag = true)]
pub struct Args {
    #[clap(short, long, help = "debug model")]
    debug: bool,

    #[clap(long, help = "Print help information")]
    help: bool,

    #[clap(long = "storage_path", default_value = "config")]
    /// load config path, default '${pwd}/config'
    config_path: Option<PathBuf>,

    /// The subcommand to run.
    #[clap(subcommand)] // Note that we mark a field as a subcommand
    cmd: Option<command::Command>,

    #[clap(long = "quiet", short = 'q')]
    /// No output printed to stdout
    quiet: Option<bool>,

    #[clap(short = 'l', default_value = "info", long)]
    log_level: String,

    #[clap(short = 'n', long, help = "Force non-interactive mode")]
    non_interactive: bool,

    #[clap(long, require_equals = true, help = "Query to execute")]
    query: Option<String>,
}

impl Args {
    pub fn fix_settings(&mut self) {
        if self.config_path.is_none() {
            self.config_path = Some("config".parse().unwrap());
        }
    }
}

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
    eprintln!("██ ██     ██  ██");
    eprintln!("██  ██     ████  KV Storage CLI");
    eprintln!();

    let mut args = Args::parse();
    if args.debug {
        println!("{:?}", args);
    }
    args.fix_settings();
    info!("kvcli start args: {:?}", &args);

    let log_dir = format!(
        "{}/.kvcli",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    );
    let _guards = trace::init_logging(&log_dir, &args.log_level).await?;

    let mut cfg: ConfigLoad = confy::load_path(args.config_path.as_ref().unwrap())?;
    cfg.fix_settings();
    if args.debug {
        println!("{:?}", &cfg);
        eprintln!();
    }
    info!("kvcli start config: {:?}", &cfg);

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

    info!("kvcli starting, Prepare Running packet with is_repl[{}].", is_repl);

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
