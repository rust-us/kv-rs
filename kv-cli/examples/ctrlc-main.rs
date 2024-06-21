#![allow(unused)]

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, StderrLock, StdoutLock, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use anyhow::{Context, Result};

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

/// ctrlc 箱只会处理 Ctrl+C，或者在 Unix 系统中，称为 SIGINT（中断信号）
///
/// cargo run -- --port 3006
///
fn main() -> Result<()> {
    let args = Cli::from_args();

    println!("Hello, world! {:?}", args);

    let pb = indicatif::ProgressBar::new(100);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    println!("Waiting for Ctrl-C...");
    let mut idx = 0;
    while running.load(Ordering::SeqCst) {
        if idx >= 50 {
            break;
        }

        // Following code does the actual work, and can be interrupted by pressing
        // Ctrl-C. As an example: Let's wait a few seconds.
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

