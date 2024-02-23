use std::borrow::Cow;
use std::sync::Arc;

use rustyline::completion::Completer;
use rustyline::completion::FilenameCompleter;
use rustyline::completion::Pair;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::ValidationContext;
use rustyline::validate::ValidationResult;
use rustyline::validate::Validator;
use rustyline::Context;
use rustyline::Helper;
use rustyline::Result;

pub struct CliHelper {
    completer: FilenameCompleter,
    keywords: Arc<Vec<String>>,
}

impl CliHelper {
    pub fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
            keywords: Arc::new(Vec::new()),
        }
    }

    pub fn with_keywords(keywords: Arc<Vec<String>>) -> Self {
        Self {
            completer: FilenameCompleter::new(),
            keywords,
        }
    }
}

impl Highlighter for CliHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // let tokens = tokenize_sql(line);
        let mut line = line.to_owned();

        Cow::Owned(line)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> std::borrow::Cow<'b, str> {
        let _ = default;
        std::borrow::Cow::Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        std::borrow::Cow::Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: rustyline::CompletionType,
    ) -> std::borrow::Cow<'c, str> {
        let _ = completion;
        std::borrow::Cow::Borrowed(candidate)
    }

    fn highlight_char(&self, line: &str, _pos: usize) -> bool {
        !line.is_empty()
    }
}

impl Hinter for CliHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        let last_word = line
            .split(|p: char| p.is_whitespace() || p == '.')
            .last()
            .unwrap_or(line);

        if last_word.is_empty() {
            return None;
        }

        let (_, res) = KeyWordCompleter::complete(line, pos, &self.keywords);
        if !res.is_empty() {
            Some(res[0].replacement[last_word.len()..].to_owned())
        } else {
            None
        }
    }
}

impl Completer for CliHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> std::result::Result<(usize, Vec<Pair>), ReadlineError> {
        let keyword_candidates = KeyWordCompleter::complete(line, pos, self.keywords.as_ref());
        if !keyword_candidates.1.is_empty() {
            return Ok(keyword_candidates);
        }
        self.completer.complete(line, pos, ctx)
    }
}

impl Validator for CliHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult> {
        let input = ctx.input().trim_end();
        if input.strip_suffix('\\').is_some() {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl Helper for CliHelper {}

struct KeyWordCompleter {}

impl KeyWordCompleter {
    fn complete(s: &str, pos: usize, keywords: &[String]) -> (usize, Vec<Pair>) {
        let hint = s
            .split(|p: char| p.is_whitespace() || p == '.')
            .last()
            .unwrap_or(s);
        // let all_keywords = all_reserved_keywords();
        let all_keywords = Vec::<String>::new();

        let mut results: Vec<Pair> = all_keywords
            .iter()
            .filter(|keyword| keyword.starts_with(&hint.to_ascii_lowercase()))
            .map(|keyword| Pair {
                display: keyword.to_string(),
                replacement: keyword.to_string(),
            })
            .collect();

        results.extend(
            keywords
                .iter()
                .filter(|keyword| {
                    keyword
                        .to_lowercase()
                        .starts_with(&hint.to_ascii_lowercase())
                })
                .map(|keyword| Pair {
                    display: keyword.to_string(),
                    replacement: keyword.to_string(),
                }),
        );

        if pos >= hint.len() {
            (pos - hint.len(), results)
        } else {
            (0, results)
        }
    }
}
