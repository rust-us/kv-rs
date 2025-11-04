use std::convert::Infallible;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::server::config::{ConfigLoad, DEFAULT_PROMPT};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use log::info;
use rustyline::{CompletionType, Editor};
use rustyline::config::Builder;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use tokio::time::Instant;
use kv_rs::error::{CResult, Error};
use kv_rs::info::get_info;
use kv_rs::row::rows::ServerStats;
use kv_rs::storage::engine::Engine;
use kv_rs::storage::log_cask::LogCask;
use kv_rs::encoding::{EncodingEngine, EncodingFormat, Base64Codec, HexCodec, JsonCodec};
use crate::ast::token_kind::TokenKind;
use crate::ast::tokenizer::{Token, Tokenizer};
use crate::rusty::CliHelper;
use crate::show::Show;

pub const SET_RESP_STR: &str = "OK";
pub const GET_RESP_NOT_FOUND_STR: &str = "N/A";
pub const SET_RESP_BYE_STR: &str = "Bye~";

/// Session and kv storage cmd and running
pub struct Session {
    is_repl: bool,

    running: Arc<AtomicBool>,
    engine: LogCask,
    encoding_engine: EncodingEngine,

    settings: ConfigLoad,
    query: String,
    in_comment_block: bool,

    keywords: Arc<Vec<String>>,
}

impl Session {
    /// Initialize the encoding engine with configuration and register codecs
    fn initialize_encoding_engine(settings: &ConfigLoad) -> Result<EncodingEngine> {
        // Validate encoding configuration first
        settings.validate_encoding_config()
            .map_err(|e| anyhow!("Invalid encoding configuration: {}", e))?;
        
        // Get the default format from configuration
        let default_format = settings.get_default_encoding_format()
            .map_err(|e| anyhow!("Failed to get default encoding format: {}", e))?;
        
        // Create encoding engine with the configured default format
        let mut encoding_engine = EncodingEngine::new(default_format);
        
        // Register all available codecs
        encoding_engine.register_codec(EncodingFormat::Base64, Box::new(Base64Codec::new()));
        encoding_engine.register_codec(EncodingFormat::Hex, Box::new(HexCodec::new()));
        encoding_engine.register_codec(EncodingFormat::Json, Box::new(JsonCodec::new()));
        
        info!("Encoding engine initialized with default format: {}", default_format);
        info!("Auto-detection enabled: {}", settings.is_auto_detect_enabled());
        info!("Batch size: {}", settings.get_batch_size());
        
        Ok(encoding_engine)
    }

    pub async fn try_new(settings: ConfigLoad, is_repl: bool, running: Arc<AtomicBool>) -> Result<Self> {
        if is_repl {
            println!("Welcome to {}.", DEFAULT_PROMPT);
            println!("Connecting to Client.");
            println!();
        }

        let engine = LogCask::new_compact(settings.get_data_dir().clone(), settings.get_compact_threshold())?;
        
        // Initialize encoding engine with configuration
        let encoding_engine = Self::initialize_encoding_engine(&settings)?;

        let mut keywords = Vec::with_capacity(1024);

        Ok(Self {
            is_repl,
            running,
            engine,
            encoding_engine,
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

        println!("{}", SET_RESP_BYE_STR);
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
            eprintln!("Refresh Config OK");

            return Ok(Some(ServerStats::default()));
        }

        let mut tokenizer = Tokenizer::new(query);
        let mut token_list = Vec::<Token>::new();
        while let Some(Ok(token)) = tokenizer.next() {
            if token.kind != TokenKind::EOI {
                token_list.push(token);
            }
        }

        self.dispatcher(is_repl, query, token_list).await
    }

    /// executor cmd
    async fn dispatcher (
        &mut self,
        is_repl: bool,
        query: &str,
        token_list: Vec<Token<'_>>,
    ) -> Result<Option<ServerStats>> {

        // Handle special case for SHOW ENCODINGS
        if token_list.len() >= 2 
            && token_list[0].kind == TokenKind::SHOW 
            && token_list[1].kind == TokenKind::ENCODINGS {
            return self.dispatcher_executor(QueryKind::ShowEncodings, is_repl, query, token_list).await;
        }

        let kind_may = QueryKind::try_from(token_list[0].kind.clone());
        match kind_may {
            Ok(kind) => {
                self.dispatcher_executor(kind, is_repl, query, token_list).await
            }
            Err(inf) => {
                return Err(anyhow!(inf.to_string()));
            }
        }
    }

    /// executor cmd
    async fn dispatcher_executor (
        &mut self,
        kind: QueryKind,
        is_repl: bool,
        query: &str,
        token_list: Vec<Token<'_>>,
    ) -> Result<Option<ServerStats>> {

        let start = Instant::now();

        match (kind, is_repl) {
            (QueryKind::Info, _) => {
                if is_repl {
                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                    for info in get_info(&mut self.engine) {
                        eprintln!("{}", info);
                    }
                    show.output(1);
                }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Time, _) => {
                if is_repl {
                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                    // data
                    let now: DateTime<Local> = Local::now();
                    let now_format = now.format("%Y-%m-%d %H:%M:%S%.3f");
                    eprintln!("{}", now_format);

                    show.output(1);
                }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::KSize, _) => unsafe {
                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                // // 或者前缀搜索，或者检索元数据/索引, 或者直接元数据取size
                // let mut scan_all = self.engine.scan(..).collect::<CResult<Vec<_>>>()?;
                // let size = scan_all.len();
                let status = self.engine.status();
                let size = if status.is_ok() {
                    status.unwrap().keys as i64
                } else {
                    0
                };

                if is_repl {
                    eprintln!("{}", size);

                    show.output(size);
                }

                // let c_rs = self.engine.compact();
                // match c_rs {
                //     Ok(_) => {}
                //     Err(err) => {
                //         eprintln!("{:?}", err.to_string());
                //     }
                // }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Show, _) => unsafe {
                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                let option = &token_list[1].get_slice();

                let path = self.engine.get_path();
                if is_repl {
                    eprintln!("{}", path.unwrap());

                    show.output(1);
                }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Keys, _) => unsafe {
                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                // 或者前缀搜索，或者检索元数据/索引, 或者直接元数据取size
                let mut scan_all = self.engine.scan_prefix(b"");

                if is_repl {
                    let mut size = 0;
                    while let Some((key, value)) = scan_all.next().transpose()? {
                        eprintln!("{}", String::from_utf8_unchecked(key).as_str());
                        size += 1;
                    }

                    show.output(size);
                }

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Set, _) => {
                if token_list.len() != 3 {
                    eprintln!("set args are invalid, must be 2 argruments");
                    return Ok(Some(ServerStats::default()));
                }

                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                let key = &token_list[1].get_slice();
                let value = &token_list[2].get_slice();

                let rs = self.engine.set(key.as_bytes(), value.as_bytes().to_vec());
                match rs {
                    Ok(_) => {
                        eprintln!("{}", SET_RESP_STR);
                    }
                    Err(err) => {
                        eprintln!("{}", err.to_string());
                    }
                }
                show.output(1);

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Get, _) => {
                if token_list.len() != 2 {
                    eprintln!("get args are invalid, must be 1 argruments");
                    return Ok(Some(ServerStats::default()));
                }
                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                let key = &token_list[1].get_slice();
                let rs = self.engine.get(key.as_bytes());
                match rs {
                    Ok(v) => {
                        if v.is_none() {
                            eprintln!("{}", GET_RESP_NOT_FOUND_STR);
                        } else {
                            let val = v.unwrap();
                            eprintln!("{}", String::from_utf8(val).expect("Get engine#get error"));
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err.to_string());
                    }
                };

                show.output(1);

                Ok(Some(ServerStats::default()))
            },
            (QueryKind::Del, _) => {
                if token_list.len() != 2 {
                    eprintln!("del args are invalid, must be 1 argruments");
                    return Ok(Some(ServerStats::default()));
                }

                let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);

                let key = &token_list[1].get_slice();
                let rs = self.engine.delete(key.as_bytes());
                let mut effect_size = 0;
                match rs {
                    Ok(effect) => {
                        effect_size = effect;
                        eprintln!("effect {}", effect);
                    }
                    Err(err) => {
                        eprintln!("{}", err.to_string());
                    }
                };
                show.output(effect_size);

                Ok(Some(ServerStats::default()))
            }
            (QueryKind::Encode, _) => {
                if token_list.len() < 3 {
                    return Err(anyhow!("Usage: ENCODE <key> <format>\nSupported formats: base64, hex, json"));
                }
                
                let key = token_list[1].get_slice();
                let format_str = token_list[2].get_slice();
                
                // Parse format
                let format = match format_str.to_lowercase().as_str() {
                    "base64" => EncodingFormat::Base64,
                    "hex" => EncodingFormat::Hex,
                    "json" => EncodingFormat::Json,
                    _ => return Err(anyhow!("Unsupported format: {}. Supported formats: base64, hex, json", format_str)),
                };
                
                // Get the value from storage
                let value = match self.engine.get(key.as_bytes())? {
                    Some(data) => data,
                    None => return Err(anyhow!("Key not found: {}", key)),
                };
                
                // Encode the value
                match self.encoding_engine.encode(&value, format) {
                    Ok(encoded) => {
                        if is_repl {
                            let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                            eprintln!("Encoded ({}): {}", format_str, encoded);
                            show.output(1);
                        }
                        Ok(Some(ServerStats::default()))
                    }
                    Err(e) => Err(anyhow!("Encoding failed: {}", e)),
                }
            }
            (QueryKind::Decode, _) => {
                if token_list.len() < 2 {
                    return Err(anyhow!("Usage: DECODE <key> [format]\nSupported formats: base64, hex, json"));
                }
                
                let key = token_list[1].get_slice();
                let format_str = if token_list.len() >= 3 {
                    Some(token_list[2].get_slice())
                } else {
                    None
                };
                
                // Get the encoded value from storage
                let encoded_value = match self.engine.get(key.as_bytes())? {
                    Some(data) => String::from_utf8(data)
                        .map_err(|_| anyhow!("Stored value is not valid UTF-8 text"))?,
                    None => return Err(anyhow!("Key not found: {}", key)),
                };
                
                // Determine format
                let format = if let Some(fmt_str) = format_str {
                    match fmt_str.to_lowercase().as_str() {
                        "base64" => EncodingFormat::Base64,
                        "hex" => EncodingFormat::Hex,
                        "json" => EncodingFormat::Json,
                        _ => return Err(anyhow!("Unsupported format: {}. Supported formats: base64, hex, json", fmt_str)),
                    }
                } else {
                    // Auto-detect format
                    match self.encoding_engine.detect(&encoded_value) {
                        Ok(detected_formats) => {
                            if detected_formats.is_empty() {
                                return Err(anyhow!("Could not detect encoding format. Please specify format explicitly."));
                            }
                            detected_formats[0].format
                        }
                        Err(e) => return Err(anyhow!("Format detection failed: {}", e)),
                    }
                };
                
                // Decode the value
                match self.encoding_engine.decode(&encoded_value, format) {
                    Ok(decoded) => {
                        if is_repl {
                            let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                            let decoded_str = String::from_utf8_lossy(&decoded);
                            eprintln!("Decoded ({}): {}", format, decoded_str);
                            show.output(1);
                        }
                        Ok(Some(ServerStats::default()))
                    }
                    Err(e) => Err(anyhow!("Decoding failed: {}", e)),
                }
            }
            (QueryKind::MEncode, _) => {
                if token_list.len() < 3 {
                    return Err(anyhow!("Usage: MENCCODE <key1> [key2] ... <format>\nSupported formats: base64, hex, json"));
                }
                
                // Last token is the format, all others are keys
                let format_str = token_list.last().unwrap().get_slice();
                let keys: Vec<&str> = token_list[1..token_list.len()-1].iter()
                    .map(|token| token.get_slice())
                    .collect();
                
                if keys.is_empty() {
                    return Err(anyhow!("At least one key must be specified"));
                }
                
                // Parse format
                let format = match format_str.to_lowercase().as_str() {
                    "base64" => EncodingFormat::Base64,
                    "hex" => EncodingFormat::Hex,
                    "json" => EncodingFormat::Json,
                    _ => return Err(anyhow!("Unsupported format: {}. Supported formats: base64, hex, json", format_str)),
                };
                
                if is_repl {
                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                    
                    let mut success_count = 0;
                    let mut error_count = 0;
                    
                    eprintln!("Batch encoding {} keys with format {}:", keys.len(), format_str);
                    
                    for key in keys {
                        match self.engine.get(key.as_bytes()) {
                            Ok(Some(value)) => {
                                match self.encoding_engine.encode(&value, format) {
                                    Ok(encoded) => {
                                        eprintln!("  {} -> {}", key, encoded);
                                        success_count += 1;
                                    }
                                    Err(e) => {
                                        eprintln!("  {} -> ERROR: {}", key, e);
                                        error_count += 1;
                                    }
                                }
                            }
                            Ok(None) => {
                                eprintln!("  {} -> ERROR: Key not found", key);
                                error_count += 1;
                            }
                            Err(e) => {
                                eprintln!("  {} -> ERROR: {}", key, e);
                                error_count += 1;
                            }
                        }
                    }
                    
                    eprintln!();
                    eprintln!("Batch encoding completed: {} successful, {} errors", success_count, error_count);
                    show.output(success_count + error_count);
                }
                
                Ok(Some(ServerStats::default()))
            }
            (QueryKind::MDecode, _) => {
                if token_list.len() < 2 {
                    return Err(anyhow!("Usage: MDECODE <key1> [key2] ...\nAuto-detects format or uses configured default"));
                }
                
                // All tokens except the first are keys
                let keys: Vec<&str> = token_list[1..].iter()
                    .map(|token| token.get_slice())
                    .collect();
                
                if is_repl {
                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                    
                    let mut success_count = 0;
                    let mut error_count = 0;
                    
                    eprintln!("Batch decoding {} keys (auto-detecting format):", keys.len());
                    
                    for key in keys {
                        match self.engine.get(key.as_bytes()) {
                            Ok(Some(data)) => {
                                match String::from_utf8(data) {
                                    Ok(encoded_value) => {
                                        // Auto-detect format
                                        match self.encoding_engine.detect(&encoded_value) {
                                            Ok(detected_formats) => {
                                                if detected_formats.is_empty() {
                                                    eprintln!("  {} -> ERROR: Could not detect encoding format", key);
                                                    error_count += 1;
                                                } else {
                                                    let format = detected_formats[0].format;
                                                    let confidence = detected_formats[0].confidence;
                                                    
                                                    match self.encoding_engine.decode(&encoded_value, format) {
                                                        Ok(decoded) => {
                                                            let decoded_str = String::from_utf8_lossy(&decoded);
                                                            eprintln!("  {} ({}, {:.1}%) -> {}", key, format, confidence * 100.0, decoded_str);
                                                            success_count += 1;
                                                        }
                                                        Err(e) => {
                                                            eprintln!("  {} -> ERROR: Decoding failed: {}", key, e);
                                                            error_count += 1;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("  {} -> ERROR: Format detection failed: {}", key, e);
                                                error_count += 1;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("  {} -> ERROR: Stored value is not valid UTF-8 text", key);
                                        error_count += 1;
                                    }
                                }
                            }
                            Ok(None) => {
                                eprintln!("  {} -> ERROR: Key not found", key);
                                error_count += 1;
                            }
                            Err(e) => {
                                eprintln!("  {} -> ERROR: {}", key, e);
                                error_count += 1;
                            }
                        }
                    }
                    
                    eprintln!();
                    eprintln!("Batch decoding completed: {} successful, {} errors", success_count, error_count);
                    show.output(success_count + error_count);
                }
                
                Ok(Some(ServerStats::default()))
            }
            (QueryKind::Detect, _) => {
                if token_list.len() < 2 {
                    return Err(anyhow!("Usage: DETECT <key>\nDetects the encoding format of the value stored at key"));
                }
                
                let key = token_list[1].get_slice();
                
                // Get the value from storage
                let data = match self.engine.get(key.as_bytes())? {
                    Some(data) => data,
                    None => return Err(anyhow!("Key not found: {}", key)),
                };
                
                // Convert to string for detection
                let value_str = String::from_utf8(data)
                    .map_err(|_| anyhow!("Stored value is not valid UTF-8 text"))?;
                
                // Detect format
                match self.encoding_engine.detect(&value_str) {
                    Ok(detected_formats) => {
                        if is_repl {
                            let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                            
                            if detected_formats.is_empty() {
                                eprintln!("No encoding format detected for key: {}", key);
                                eprintln!("The value may be plain text or an unsupported format.");
                            } else {
                                eprintln!("Detected encoding formats for key '{}':", key);
                                for (i, result) in detected_formats.iter().enumerate() {
                                    let confidence_percent = result.confidence * 100.0;
                                    eprintln!("  {}. {} ({:.1}% confidence)", i + 1, result.format, confidence_percent);
                                }
                                
                                if detected_formats.len() > 1 {
                                    eprintln!();
                                    eprintln!("Recommendation: Use format '{}' (highest confidence)", detected_formats[0].format);
                                }
                            }
                            
                            show.output(detected_formats.len().max(1) as i64);
                        }
                        Ok(Some(ServerStats::default()))
                    }
                    Err(e) => Err(anyhow!("Format detection failed: {}", e)),
                }
            }
            (QueryKind::ShowEncodings, _) => {
                if is_repl {
                    let show = Show::new_with_start(self.settings.is_show_affected(), is_repl, start);
                    
                    eprintln!("Supported encoding formats:");
                    eprintln!("  base64  - Base64 encoding");
                    eprintln!("  hex     - Hexadecimal encoding");
                    eprintln!("  json    - JSON string encoding");
                    eprintln!();
                    eprintln!("Usage examples:");
                    eprintln!("  ENCODE mykey base64");
                    eprintln!("  DECODE mykey hex");
                    eprintln!("  MENCCODE key1 key2 base64");
                    eprintln!("  MDECODE key1 key2");
                    eprintln!("  DETECT mykey");
                    
                    show.output(3);
                }
                Ok(Some(ServerStats::default()))
            }
            (_, _) => {
                println!("__ {}", &query);

                Err(anyhow!("UnImplement command: [{}]", &query))
            }
        }
    }

    /// Update encoding configuration at runtime
    pub fn update_encoding_config(&mut self, new_config: crate::server::config::EncodingConfig) -> Result<()> {
        // Validate the new configuration
        new_config.validate()
            .map_err(|e| anyhow!("Invalid encoding configuration: {}", e))?;
        
        // Update the settings
        self.settings.set_encoding_config(new_config.clone());
        
        // Update the encoding engine's default format
        let new_default_format = new_config.get_default_format()
            .map_err(|e| anyhow!("Failed to parse default format: {}", e))?;
        self.encoding_engine.set_default_format(new_default_format);
        
        info!("Encoding configuration updated - Default format: {}, Auto-detect: {}, Batch size: {}", 
              new_config.default_format, new_config.auto_detect, new_config.batch_size);
        
        Ok(())
    }

    /// Get current encoding configuration
    pub fn get_encoding_config(&self) -> crate::server::config::EncodingConfig {
        self.settings.get_encoding_config()
    }

    /// Update default encoding format
    pub fn set_default_encoding_format(&mut self, format: EncodingFormat) -> Result<()> {
        self.settings.set_default_encoding_format(format);
        self.encoding_engine.set_default_format(format);
        info!("Default encoding format updated to: {}", format);
        Ok(())
    }

    /// Get current default encoding format
    pub fn get_default_encoding_format(&self) -> Result<EncodingFormat> {
        self.settings.get_default_encoding_format()
    }

    /// Check if auto-detection is enabled
    pub fn is_auto_detect_enabled(&self) -> bool {
        self.settings.is_auto_detect_enabled()
    }

    /// Set auto-detection enabled/disabled
    pub fn set_auto_detect(&mut self, enabled: bool) {
        self.settings.set_auto_detect(enabled);
        info!("Auto-detection {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Get batch size for bulk operations
    pub fn get_batch_size(&self) -> usize {
        self.settings.get_batch_size()
    }

    /// Set batch size for bulk operations
    pub fn set_batch_size(&mut self, size: usize) -> Result<()> {
        self.settings.set_batch_size(size)
            .map_err(|e| anyhow!("Failed to set batch size: {}", e))?;
        info!("Batch size updated to: {}", size);
        Ok(())
    }

    /// Get a reference to the encoding engine
    pub fn encoding_engine(&self) -> &EncodingEngine {
        &self.encoding_engine
    }

    /// Get a mutable reference to the encoding engine
    pub fn encoding_engine_mut(&mut self) -> &mut EncodingEngine {
        &mut self.encoding_engine
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
    Keys,
    Show,
    Set,
    Get,
    Del,
    GetSet,
    MGet,
    SetEx,
    Encode,
    Decode,
    MEncode,
    MDecode,
    Detect,
    ShowEncodings,
}

impl TryFrom<TokenKind> for QueryKind {
    type Error = String;

    #[inline(always)]
    fn try_from(kind: TokenKind) -> std::result::Result<Self, Self::Error> {
        match kind {
            TokenKind::TIME => Ok(QueryKind::Time),
            TokenKind::GET => Ok(QueryKind::Get),
            TokenKind::SET => Ok(QueryKind::Set),
            TokenKind::DEL |
            TokenKind::DELETE => Ok(QueryKind::Del),
            TokenKind::INFO => Ok(QueryKind::Info),
            TokenKind::KSize => Ok(QueryKind::KSize),
            TokenKind::SELECT => Ok(QueryKind::Select),
            TokenKind::KEYS => Ok(QueryKind::Keys),
            TokenKind::SHOW => Ok(QueryKind::Show),
            TokenKind::GETSET => Ok(QueryKind::GetSet),
            TokenKind::MGET => Ok(QueryKind::MGet),
            TokenKind::SETEX => Ok(QueryKind::SetEx),
            TokenKind::ENCODE => Ok(QueryKind::Encode),
            TokenKind::DECODE => Ok(QueryKind::Decode),
            TokenKind::MENCCODE => Ok(QueryKind::MEncode),
            TokenKind::MDECODE => Ok(QueryKind::MDecode),
            TokenKind::DETECT => Ok(QueryKind::Detect),
            _ => {
                Err("UnSupport cmd".to_owned())
            }
        }
    }
}