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
use structopt::StructOpt;
use anyhow::{Context, Result};
use signal_hook::{consts::SIGINT, iterator::Signals};
use human_panic::setup_panic;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(short = "p", long = "port")]
    port: Option<i32>,
}

impl Display for Cli {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.port.as_ref().unwrap())
    }
}

/// signal-hook 可以去处理更多的 Unix 信号。 在 https://vorner.github.io/2018/06/28/signal-hook.html 中描述了它的设计原理， 且它是目前社区里支持最为广泛的库。
///
/// cargo run -- --port 3006
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
    let args = Cli::from_args();

    println!("Hello, world! {:?}", args);

    let pb = indicatif::ProgressBar::new(100);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&running))?;
    let mut signals = Signals::new(&[SIGINT])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    println!("Waiting for Ctrl-C...");
    let mut idx = 0;
    while running.load(Ordering::SeqCst) {
        if idx >= 50 {
            break;
        }

        thread::sleep(Duration::from_secs(1));

        pb.inc(2);
        idx += 1;
    }

    pb.finish_with_message("done");
    if running.load(Ordering::SeqCst) {
        println!("idx is 100, Exiting...");
    } else {
        println!("Got it! Exiting...");
    }

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
