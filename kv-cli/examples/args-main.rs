#![allow(unused)]

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{self, BufWriter, StderrLock, StdoutLock, Write};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt, Debug)]
struct Cli {
    /// The pattern to look for
    #[structopt(short = "p", long = "pattern")]
    pattern: String,

    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

impl Display for Cli {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {:?})", self.pattern, self.path)
    }
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    println!("Hello, world! {:?}", args);

    find_files(&args.path, &args.pattern)?;

    Ok(())
}

pub fn find_files(path: &PathBuf, pattern: &str) -> Result<()> {
    let f  = File::open(&path)
        .with_context(|| format!("could not read file `{:?}`", &path))?;
    let mut reader = BufReader::new(f);

    find_matches(&mut reader, pattern);

    Ok(())
}

fn find_matches(reader: &mut BufReader<File>, pattern: &str) {
    // 进度条
    let pb = indicatif::ProgressBar::new(100);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        // .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    // 关于打印性能： 通过  BufWriter 包装 stdout 的句柄（BufWriter的默认缓存为 8 kB）， 提升打印性能。
    // 当你想立即打印时，在 BufWriter 上调用 .flush() 即可
    // handle.flush();
    // 为 stdout（或 stderr）申请一把锁并使用 writeln! 来直接打印, 它会阻止系统不停地锁死和解锁 stdout。
    let stdout = io::stdout().lock(); // get the global stdout entity
    let stderr = io::stderr().lock();
    let mut stdout_handle = io::BufWriter::new(stdout); // optional: wrap that handle in a buffer
    let mut stderr_handle = io::BufWriter::new(stderr);
    let mut idx = 0;
    loop {
        // 文件分批读取到内存中
        let mut line = String::new();
        let rs = reader.read_line(&mut line);
        if rs.is_err() {
            break;
        }

        let len = rs.unwrap();
        assert_eq!(line.len(), len);
        if len == 0 {
            break;
        }

        find_matche_line(&line, &pattern, &pb, &mut stdout_handle, &mut stderr_handle);
        // if line.contains(&pattern) {
        //     // stdout
        //     writeln!(stdout_handle, "line is {} bytes long, {}", len, line);
        //     // } else {
        //     //     // stderr
        //     //     writeln!(stderr_handle, "line is {} bytes long", len);
        // }

        idx += 1;
        // 打印进度条的输出信息
        // pb.println(format!("[+] finished #{}", idx));
        // 更新进度条进度
        pb.inc(1);
    }
    pb.finish_with_message("done");
}

fn find_matche_line(line: &str, pattern: &str, pb: &ProgressBar,
                    out: &mut impl std::io::Write, err: &mut impl std::io::Write) -> bool{
    if line.contains(&pattern) {
        // stdout
        writeln!(out, "line is {} bytes long, {}", line.len(), line);
        // } else {
        //     // stderr
        //     writeln!(stderr_handle, "line is {} bytes long", line.len());

        return true;
    }

    return false;
}

#[cfg(test)]
mod test {
    use std::io;
    use std::process::Command;
    use crate::find_matche_line;

    #[test]
    fn test_find_matche_line() {
        let pb = indicatif::ProgressBar::new(100);

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let line_str = "lorem ipsum\ndolor sit amet";
        let rs_1 = find_matche_line(&line_str, "lorem", &pb, &mut stdout, &mut stderr);
        assert!(rs_1);
        assert_eq!(&line_str.len(), &26usize);
        // b 使得它被当作 byte 字节，即其类型为 &[u8] 而非 &str
        assert_eq!(stdout, b"line is 26 bytes long, lorem ipsum\ndolor sit amet\n");
        assert!(stderr.is_empty());

        stdout.clear();
        stderr.clear();
        let rs_2 = find_matche_line("abcd", "lorem", &pb, &mut stdout, &mut stderr);
        assert!(!rs_2);
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}
