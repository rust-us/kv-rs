use std::io::BufRead;
use std::sync::Arc;
use crate::config::ConfigLoad;
use anyhow::{anyhow, Result};
use rustyline::{CompletionType, Editor};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use tokio::time::Instant;
use kv::row::rows::ServerStats;
use crate::rusty::CliHelper;

const DEFAULT_PROMPT: &str = "kvcli";

pub struct Session {
    is_repl: bool,
    settings: ConfigLoad,
    query: String,
    in_comment_block: bool,

    keywords: Arc<Vec<String>>,
}

impl Session {
    pub async fn try_new(settings: ConfigLoad, is_repl: bool) -> Result<Self> {
        if is_repl {
            println!("Welcome to kvcli.");
            println!();
        }

        let mut keywords = Vec::with_capacity(1024);

        Ok(Self {
            is_repl,
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
                format!("{} ", prompt.trim_end())
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
            match rl.readline(&self.prompt().await) {
                Ok(line) => {
                    let queries = self.append_query(&line);
                    for query in queries {
                        let _ = rl.add_history_entry(&query);

                        let rs = self.handle_query(true, &query).await;
                        match rs {
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
                        self.query.clear();
                        println!("^C");
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

    fn append_query(&mut self, line: &str) -> Vec<String> {
        let line = line.trim();
        if line.is_empty() {
            return vec![];
        }

        if self.query.is_empty()
            &&
            (
                line.starts_with('.')
                || line == "exit"
                || line == "quit"
                || line.to_uppercase().starts_with("PUT")
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

        let mut queries = Vec::new();
        queries.push(line.trim().to_owned());
        self.query.clear();

        queries
    }

    async fn handle_query(
        &mut self,
        is_repl: bool,
        query: &str,
    ) -> Result<Option<ServerStats>> {
        let query = query.trim_end_matches(';').trim();
        if is_repl && (query == "exit" || query == "quit") {
            return Ok(None);
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

            // self.settings.inject_ctrl_cmd(query[0], query[1])?;
            return Ok(Some(ServerStats::default()));
        }

        println!("cmd: {}", query);

        let stats = ServerStats::default();
        Ok(Some(stats))
    }
}

fn get_history_path() -> String {
    format!(
        "{}/.kvcli_history",
        std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
    )
}