use tokio::time::Instant;

//! Show affected Info
pub struct Show {
    is_show_affected: bool,
    is_repl: bool,

    start: Instant,
}

impl Show {
    pub fn new(is_show_affected: bool, is_repl: bool) -> Self {
        let start = Instant::now();

        Self::new_with_start(is_show_affected, is_repl, start)
    }

    pub fn new_with_start(is_show_affected: bool, is_repl: bool, start: Instant) -> Self {
        Show {
            is_show_affected,
            is_repl,
            start,
        }
    }

    pub fn output(&self, affected: i64) {
        if self.is_show_affected && self.is_repl {
            if affected > 0 {
                eprintln!(
                    "{} rows affected in ({:.3} sec)",
                    affected,
                    self.start.elapsed().as_secs_f64()
                );
            } else {
                eprintln!("processed in ({:.3} sec)", self.start.elapsed().as_secs_f64());
            }
            eprintln!();
        }
    }
}