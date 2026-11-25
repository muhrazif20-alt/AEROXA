#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub location: SourceLocation,
}

impl Diagnostic {
    pub fn error(
        code: &str,
        message: &str,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            code: code.to_string(),
            severity: Severity::Error,
            message: message.to_string(),
            location: SourceLocation {
                line,
                column,
            },
        }
    }

    pub fn display(&self) {
        println!(
            "[{:?}] {} at {}:{}",
            self.severity,
            self.code,
            self.location.line,
            self.location.column
        );

        println!("  {}", self.message);
    }
}

pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| {
        matches!(d.severity, Severity::Error)
    })
}