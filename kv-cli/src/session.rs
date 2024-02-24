use std::io::BufRead;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::config::{ConfigLoad, DEFAULT_PROMPT};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use log::info;
use rustyline::{CompletionType, Editor};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use tokio::time::Instant;
use kv::row::rows::ServerStats;
use kv::storage::engine::Engine;
use kv::storage::log_cask::LogCask;
use crate::ast::token_kind::TokenKind;
use crate::ast::tokenizer::{Token, Tokenizer};
use crate::rusty::CliHelper;
use crate::show::Show;

pub struct Session {
    is_repl: bool,

    running: Arc<AtomicBool>,
    engine: Box<dyn Engine>,

    settings: ConfigLoad,
    query: String,
    in_comment_block: bool,

    keywords: Arc<Vec<String>>,
}

impl Session {
    pub async fn try_new(settings: ConfigLoad, is_repl: bool, running: Arc<AtomicBool>) -> Result<Self> {
        if is_repl {
            println!("Welcome to {}.", DEFAULT_PROMPT);
            println!("Connecting to Client.");
            println!();
        }

        let engine = LogCask::new(settings.get_storage_path().clone())?;

        let mut keywords = Vec::with_capacity(1024);

        Ok(Self {
            is_repl,
            running,
            engine: Box::new(engine),
            settings,
            query: String::new(),
            in_comment_block: false,
            keywords: Arc::new(keywords),
        })
    }

    async fn prompt(&self) -> String {
        if !self.query.trim().is_empty() {
            format!("{} > ", DEFAULT_PROMPT).to_owned()
        } else {
            if self.settings.prompt.is_some() {
                let mut prompt = self.settings.prompt.as_ref().unwrap().clone();
                // prompt = prompt.replace("{user}", &user);
                format!("{} > ", prompt.trim_end())
            } else {
                format!("{} > ", DEFAULT_PROMPT)
            }
        }
    }

    pub async fn handle_repl(&mut self) {
        let config = Builder::new()
            .completion_prompt_limit(5)
            .completion_type(CompletionType::Circular)
            .build();
        let mut rl = Editor::<CliHelper, DefaultHistory>::with_config(config).unwrap();

        rl.set_helper(Some(CliHelper::with_keywords(self.keywords.clone())));
        rl.load_history(&get_history_path()).ok();

        'F: loop {
            if !self.running.load(Ordering::SeqCst) {
                break 'F;
            }

            match rl.readline(&self.prompt().await) {
                Ok(line) => {
                    let queries = self.append_query(&line);
                    for query in queries {
                        let _ = rl.add_history_entry(&query);
                        match self.handle_query(true, &query).await {
                            Ok(None) => {
                                break 'F;
                            }
                            Ok(Some(_)) => {}
                            Err(e) => {
                                eprintln!("error: {}", e);
                                self.query.clear();
                                break;
                            }
                        }
                    }
                },
                Err(e) => match e {
                    ReadlineError::Io(err) => {
                        eprintln!("io err: {err}");
                    }
                    ReadlineError::Interrupted => {
                        println!("^C");

                        self.query.clear();
                        self.running.store(false, Ordering::SeqCst);
                    }
                    ReadlineError::Eof => {
                        break;
                    }
                    _ => {}
                },
            }
        }

        println!("Bye~");
        let _ = rl.save_history(&get_history_path());
    }

    pub async fn handle_reader<R: BufRead>(&mut self, r: R) -> Result<()> {
        let start = Instant::now();
        let mut lines = r.lines();
        let mut stats: Option<ServerStats> = None;

        loop {
            match lines.next() {
                Some(Ok(line)) => {
                    let queries = self.append_query(&line);
                    for query in queries {
                        stats = self.handle_query(false, &query).await?;
                    }
                }
                Some(Err(e)) => {
                    return Err(anyhow!("read lines err: {}", e.to_string()));
                }
                None => break,
            }
        }

        // if the last query is not finished with `;`, we need to execute it.
        let query = self.query.trim().to_owned();
        if !query.is_empty() {
            self.query.clear();
            stats = self.handle_query(false, &query).await?;
        }

        // local time
        println!("{:.3}", start.elapsed().as_secs_f64());

        Ok(())
    }

    /// 用于输入不完整的命令的追加和补充。
    fn append_query(&mut self, line: &str) -> Vec<String> {
        let line = line.trim();
        if line.is_empty() {
            return vec![];
        }

        if !self.settings.get_auto_append_part_cmd() {
            return vec![line.to_owned()];
        }

        if self.query.is_empty()
            &&
            (
                line.starts_with('.')
                || line == "exit"
                || line == "quit"
                || line.to_uppercase().starts_with("SET")
            )
        {
            return vec![line.to_owned()];
        }

        if self.settings.multi_line.is_some() && !self.settings.multi_line.as_ref().unwrap() {
            if line.starts_with("--") {
                return vec![];
            } else {
                return vec![line.to_owned()];
            }
        }

        self.query.push(' ');

        let mut queries = Vec::new();
        let mut tokenizer = Tokenizer::new(line);
        let mut in_comment = false;
        let mut start = 0;
        let mut comment_block_start = 0;

        let append_part_cmd_symbol = self.settings.get_auto_append_part_cmd_symbol();
        while let Some(Ok(token)) = tokenizer.next() {
            match token.kind {
                TokenKind::SemiColon => {
                    if in_comment || self.in_comment_block {
                        continue;
                    } else {
                        let mut sql = self.query.trim().to_owned();
                        if sql.is_empty() {
                            continue;
                        }

                        sql.push(';');

                        queries.push(sql);
                        self.query.clear();
                    }
                }
                TokenKind::Comment => {
                    in_comment = true;
                }
                TokenKind::EOI => {
                    in_comment = false;
                }
                TokenKind::Newline => {
                    in_comment = false;
                    self.query.push('\n');
                }
                TokenKind::CommentBlockStart => {
                    if !self.in_comment_block {
                        comment_block_start = token.span.start;
                    }
                    self.in_comment_block = true;
                }
                TokenKind::CommentBlockEnd => {
                    self.in_comment_block = false;
                    self.query
                        .push_str(&line[comment_block_start..token.span.end]);
                }
                _ => {
                    if !in_comment && !self.in_comment_block {
                        self.query.push_str(&line[start..token.span.end]);
                    }
                }
            }
            start = token.span.end;
        }

        if self.in_comment_block {
            self.query.push_str(&line[comment_block_start..]);
        }

        queries
    }

    /// executor cmd
    async fn handle_query(
        &mut self,
        is_repl: bool,
        query: &str,
    ) -> Result<Option<ServerStats>> {
        let query = query.trim_end_matches(';').trim();
        if is_repl && (query == "exit" || query == "quit") {
            return Ok(None); // exit
        }

        if is_repl && query.starts_with('.') {
            let query = query
                .trim_start_matches('.')
                .split_whitespace()
                .collect::<Vec<_>>();
            if query.len() != 2 {
                return Err(anyhow!(
                    "Control command error, must be syntax of `.cmd_name cmd_value`."
                ));
            }

            self.settings.inject_cmd(query[0], query[1])?;
            info!("refresh config: {:?}", &self.settings);
            eprintln!("Refresh Config OK ~");

            return Ok(Some(ServerStats::default()));
        }

        let mut tokenizer = Tokenizer::new(query);
        let mut token_list = Vec::<Token>::new();
        while let Some(Ok(token)) = tokenizer.next() {
            token_list.push(token);
        }

        self.dispatcher(is_repl, query, token_list).await
    }

    /// executor cmd
    async fn dispatcher(
        &mut self,
        is_repl: bool,
        query: &str,
        token_list: Vec<Token<'_>>,
    ) -> Result<Option<ServerStats>> {

        let start = Instant::now();
        let kind = QueryKind::try_from(token_list[0].kind.clone()).unwrap();

        match (kind, is_repl) {
            (QueryKind::Time, _) => {
                let affected = 1;
                if is_repl {
                    // data
                    let now: DateTime<Local> = Local::now();
                    let now_format = now.format("%Y-%m-%d %H:%M:%S%.3f");
                    eprintln!("{}", now_format);

                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                    show.output(affected);
                }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Set, _) => {
                let args: Vec<String> = get_put_get_args(query);
                if args.len() != 3 {
                    eprintln!("set args are invalid, must be 2 argruments");
                    return Ok(Some(ServerStats::default()));
                }

                let affected = 1;

                let key = &args[1];
                let value = &args[2];

                let rs = self.engine.set(key.as_bytes(), value.as_bytes().to_vec());
                match rs {
                    Ok(_) => {
                        eprintln!("OK ~");
                    }
                    Err(err) => {
                        eprintln!("{}", err.to_string());
                    }
                }

                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                show.output(affected);
                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Get, _) => {
                let args: Vec<String> = get_put_get_args(query);
                if args.len() != 2 {
                    eprintln!("put args are invalid, must be 1 argruments");
                    return Ok(Some(ServerStats::default()));
                }

                let key = &args[1];
                let rs = self.engine.get(key.as_bytes());
                match rs {
                    Ok(v) => {
                        if v.is_none() {
                            eprintln!("N/A ~");
                        } else {
                            let val = v.unwrap();
                            eprintln!("{}", String::from_utf8(val).expect("Get engine#get error"));
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err.to_string());
                    }
                };

                Ok(Some(ServerStats::default()))
            }
            (_, _) => {
                println!("__ {}", &query);

                let stats = ServerStats::default();
                Ok(Some(stats))
            }
        }
    }
}

fn get_history_path() -> String {
    format!(
        "{}/.kvcli_history",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    )
}

#[derive(PartialEq, Eq, Debug)]
pub enum QueryKind {
    Info,
    Time,
    KSize,
    Exit,
    Select,
    Set,
    Get,
    Del,
    GetSet,
    MGet,
    SetEx,
}

impl From<TokenKind> for QueryKind {
    fn from(kind: TokenKind) -> Self {
        match kind {
            TokenKind::TIME => QueryKind::Time,
            TokenKind::SET => QueryKind::Set,
            TokenKind::GET => QueryKind::Get,
            TokenKind::SELECT => QueryKind::Select,
            _ => QueryKind::Select,
        }
    }
}

fn get_put_get_args(query: &str) -> Vec<String> {
    query
        .split_ascii_whitespace()
        .map(|x| x.to_owned())
        .collect()
}
