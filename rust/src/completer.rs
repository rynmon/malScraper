use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{self, Validator, ValidationContext};
use rustyline::{Context, Helper};
use rustyline::Result as RustylineResult;

pub struct CommandCompleter {
    commands: Vec<String>,
}

impl CommandCompleter {
    pub fn new() -> Self {
        // All available commands and their aliases (case-insensitive)
        let commands = vec![
            // Main scan commands
            "full".to_string(),
            "full-scan".to_string(),
            "fscan".to_string(),
            "quick".to_string(),
            "quick-scan".to_string(),
            "qscan".to_string(),
            // Navigation commands
            "help".to_string(),
            "get-help".to_string(),
            "?".to_string(),
            "-?".to_string(),
            "/?".to_string(),
            "menu".to_string(),
            "tutorial".to_string(),
            "home".to_string(),
            "back".to_string(),
            "cd ..".to_string(),
            "clear".to_string(),
            "clear-host".to_string(),
            "cls".to_string(),
            // File operations
            "open".to_string(),
            "reopen".to_string(),
            // System commands
            "quit".to_string(),
            "exit".to_string(),
            "install".to_string(),
            "update".to_string(),
        ];

        Self { commands }
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let line_lower = line.to_lowercase().trim().to_string();
        
        if line_lower.is_empty() {
            // Return all commands if input is empty
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .map(|cmd| Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                })
                .collect();
            return Ok((0, matches));
        }

        // Find commands that start with the input (case-insensitive)
        let matches: Vec<Pair> = self
            .commands
            .iter()
            .filter(|cmd| cmd.to_lowercase().starts_with(&line_lower))
            .map(|cmd| Pair {
                display: cmd.clone(),
                replacement: cmd.clone(),
            })
            .collect();

        Ok((0, matches))
    }
}

impl Default for CommandCompleter {
    fn default() -> Self {
        Self::new()
    }
}

// Make CommandCompleter work as a Helper for rustyline
impl Helper for CommandCompleter {}
impl Validator for CommandCompleter {
    fn validate(
        &self,
        _ctx: &mut ValidationContext<'_>,
    ) -> RustylineResult<validate::ValidationResult> {
        Ok(validate::ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
impl Hinter for CommandCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}
impl Highlighter for CommandCompleter {}

