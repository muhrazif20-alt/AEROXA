use crate::diagnostics::{Diagnostic, SourceLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Else,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    At,

    Let,
    Mut,
    Fn,

    And,
    Or,

    Condition,
    Alternative,
    OtherWise,
    LoopCheck,

    Identifier(String),
    Number(String),
    StringLiteral(String),
    Boolean(bool),

    Colon,
    DoubleColon,
    Semicolon,

    Equal,
    EqualEqual,
    NotEqual,

    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    AndAnd,
    OrOr,
    Bang,

    Arrow,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    EOF,
}

#[derive(Debug, Clone)]
pub struct LocatedToken {
    pub token: Token,
    pub location: SourceLocation,
}

pub struct LexerResult {
    pub tokens: Vec<LocatedToken>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn tokenize(source: &str) -> LexerResult {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    let mut chars = source.chars().peekable();

    let mut line = 1;
    let mut column = 1;

    while let Some(ch) = chars.next() {
        let current_line = line;
        let current_column = column;

        if ch == '\n' {
            line += 1;
            column = 1;
            continue;
        }

        column += 1;

        let token = match ch {
            ' ' | '\r' | '\t' => None,

            // ANNOTATIONS
            '@' => {
                let mut annotation = String::new();

                while let Some(next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || *next == '_' {
                        annotation.push(chars.next().unwrap());
                        column += 1;
                    } else {
                        break;
                    }
                }

                match annotation.as_str() {
                    "condition" => Some(Token::Condition),
                    "alternative" => Some(Token::Alternative),
                    "otherwise" => Some(Token::OtherWise),

                    "overrideDouble" => {
                        tokens.push(LocatedToken {
                            token: Token::At,
                            location: SourceLocation {
                                line: current_line,
                                column: current_column,
                            },
                        });

                        tokens.push(LocatedToken {
                            token: Token::Identifier(annotation),
                            location: SourceLocation {
                                line: current_line,
                                column: current_column + 1,
                            },
                        });

                        None
                    }

                    _ => {
                        diagnostics.push(Diagnostic::error(
                            "AX011",
                            &format!(
                                "Unknown annotation: '@{}'.",
                                annotation
                            ),
                            current_line,
                            current_column,
                        ));

                        None
                    }
                }
            }

            // #else$
            '#' => {
                let keyword = "else$";
                let mut matched = true;

                for expected in keyword.chars() {
                    if chars.peek() == Some(&expected) {
                        chars.next();
                        column += 1;
                    } else {
                        matched = false;
                        break;
                    }
                }

                if matched {
                    Some(Token::Else)
                } else {
                    diagnostics.push(Diagnostic::error(
                        "AX018",
                        "Invalid keyword. Expected '#else$'.",
                        current_line,
                        current_column,
                    ));

                    None
                }
            }

            // PUNCTUATION
            ':' => {
                if chars.peek() == Some(&':'){
                    chars.next();
                    column += 1;
                    Some(Token::DoubleColon)
                }else{
                    Some(Token::Colon)
                }
            },
            ';' => Some(Token::Semicolon),

            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),

            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),

            // ASSIGNMENT AND EQUALITY
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    column += 1;
                    Some(Token::EqualEqual)
                } else {
                    Some(Token::Equal)
                }
            }

            // LOGICAL NOT AND NOT EQUAL
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    column += 1;
                    Some(Token::NotEqual)
                } else {
                    Some(Token::Bang)
                }
            }

            // LOGICAL AND
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    column += 1;
                    Some(Token::AndAnd)
                } else {
                    diagnostics.push(Diagnostic::error(
                        "AX016",
                        "Expected '&' after '&'.",
                        current_line,
                        current_column,
                    ));

                    None
                }
            }

            // LOGICAL OR
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    column += 1;
                    Some(Token::OrOr)
                } else {
                    diagnostics.push(Diagnostic::error(
                        "AX017",
                        "Expected '|' after '|'.",
                        current_line,
                        current_column,
                    ));

                    None
                }
            }

            // GREATER THAN
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    column += 1;
                    Some(Token::GreaterEqual)
                } else {
                    Some(Token::Greater)
                }
            }

            // LESS THAN
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    column += 1;
                    Some(Token::LessEqual)
                } else {
                    Some(Token::Less)
                }
            }

            // ARITHMETIC OPERATORS
            '+' => Some(Token::Plus),

            '-' => {
                if chars.peek() == Some(&'>') {
                    chars.next();
                    column += 1;
                    Some(Token::Arrow)
                } else {
                    Some(Token::Minus)
                }
            }

            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '%' => Some(Token::Percent),

            // STRING LITERAL
            '"' => {
                let mut value = String::new();
                let mut closed = false;

                while let Some(c) = chars.next() {
                    if c == '"' {
                        closed = true;
                        column += 1;
                        break;
                    }

                    if c == '\n' {
                        diagnostics.push(Diagnostic::error(
                            "AX002",
                            "Unterminated string literal.",
                            current_line,
                            current_column,
                        ));

                        line += 1;
                        column = 1;
                        break;
                    }

                    value.push(c);
                    column += 1;
                }

                if !closed {
                    if !diagnostics.iter().any(|d| {
                        d.code == "AX002"
                            && d.location.line == current_line
                            && d.location.column == current_column
                    }) {
                        diagnostics.push(Diagnostic::error(
                            "AX002",
                            "Unterminated string literal.",
                            current_line,
                            current_column,
                        ));
                    }

                    None
                } else {
                    Some(Token::StringLiteral(value))
                }
            }

            // NUMBER
            c if c.is_ascii_digit() => {
                let mut number = String::from(c);

                while let Some(next) = chars.peek() {
                    if next.is_ascii_digit() || *next == '.' {
                        number.push(chars.next().unwrap());
                        column += 1;
                    } else {
                        break;
                    }
                }

                Some(Token::Number(number))
            }

            // IDENTIFIERS AND KEYWORDS
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut identifier = String::from(c);

                while let Some(next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || *next == '_' {
                        identifier.push(chars.next().unwrap());
                        column += 1;
                    } else {
                        break;
                    }
                }

                Some(match identifier.as_str() {
                    "let" => Token::Let,

                    // IMPORTANT: MUTABILITY KEYWORD
                    "mut" => Token::Mut,

                    "fn" => Token::Fn,

                    // CONDITIONAL KEYWORDS
                    "check" => Token::Condition,
                    "orcheck" => Token::Alternative,

                    // LOOP KEYWORD
                    "loopcheck" => Token::LoopCheck,

                    // BOOLEAN
                    "true" => Token::Boolean(true),
                    "false" => Token::Boolean(false),

                    _ => Token::Identifier(identifier),
                })
            }

            // INVALID CHARACTER
            _ => {
                diagnostics.push(Diagnostic::error(
                    "AX003",
                    &format!("Invalid character '{}'.", ch),
                    current_line,
                    current_column,
                ));

                None
            }
        };

        if let Some(token) = token {
            tokens.push(LocatedToken {
                token,
                location: SourceLocation {
                    line: current_line,
                    column: current_column,
                },
            });
        }
    }

    // END OF FILE
    tokens.push(LocatedToken {
        token: Token::EOF,
        location: SourceLocation {
            line,
            column,
        },
    });

    LexerResult {
        tokens,
        diagnostics,
    }
}