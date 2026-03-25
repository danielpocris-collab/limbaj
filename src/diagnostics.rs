use std::fmt;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Error => write!(f, "error"),
            Level::Warning => write!(f, "warning"),
            Level::Note => write!(f, "note"),
        }
    }
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, line: usize, col: usize) -> Self {
        Diagnostic {
            level: Level::Error,
            message: message.into(),
            line,
            col,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn format_message(&self, source_line: Option<&str>) -> String {
        let mut output = format!(
            "{}:{}:{}: {}: {}\n",
            self.line, self.col, self.level, self.level, self.message
        );

        if let Some(line) = source_line {
            output.push_str(line);
            output.push('\n');
            output.push_str(&" ".repeat(self.col.saturating_sub(1)));
            output.push_str("^\n");
        }

        if let Some(suggestion) = &self.suggestion {
            output.push_str(&format!("  hint: {}\n", suggestion));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("Type mismatch", 1, 5);
        assert_eq!(diag.message, "Type mismatch");
        assert_eq!(diag.line, 1);
    }

    #[test]
    fn test_diagnostic_with_suggestion() {
        let diag = Diagnostic::error("Type mismatch", 1, 5)
            .with_suggestion("Did you mean i32?");
        assert!(diag.suggestion.is_some());
    }
}
