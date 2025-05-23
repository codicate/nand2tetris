#[derive(Debug)]
enum TokenType {
    Keyword,
    Symbol,
    IntegerConstant,
    StringConstant,
    Identifier,
}

struct Token {
    type_: TokenType,
    content: String,
}

pub struct Tokenizer {
    idx: usize,
    content: Vec<char>,
}

impl Tokenizer {
    const KEYWORDS: [&'static str; 21] = [
        "class",
        "constructor",
        "function",
        "method",
        "field",
        "static",
        "var",
        "int",
        "char",
        "boolean",
        "void",
        "true",
        "false",
        "null",
        "this",
        "let",
        "do",
        "if",
        "else",
        "while",
        "return",
    ];

    const SYMBOLS: [char; 21] = [
        '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=',
        '~', ' ', '"',
    ];

    fn is_all_digits(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    fn is_valid_identifier(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    pub fn new(content: String) -> Self {
        Tokenizer {
            idx: 0,
            content: content.chars().collect(),
        }
    }

    pub fn hasMoreTokens(&self) -> bool {
        return self.idx < self.content.len();
    }

    pub fn advance(&mut self) -> Option<Token> {
        if !self.hasMoreTokens() {
            return None;
        }

        let cur = self.content[self.idx];
        if Self::SYMBOLS.contains(&cur) {
            // ignore whitespace
            if cur == ' ' {
                self.idx += 1;
                return None;
            }

            // string constant
            if cur == '"' {
                let rest = &self.content[self.idx + 1..];
                if let Some(pos) = rest.iter().position(|&c| c == '"') {
                    let string_constant = &rest[..pos];
                    self.idx += pos + 1;
                    return Some(Token {
                        type_: TokenType::StringConstant,
                        content: string_constant.iter().collect(),
                    });
                } else {
                    eprintln!("unclosed double quote \"");
                    std::process::exit(1);
                }
            }

            // check if we landed on a comment and skip over comment content
            if cur == '/' && self.idx + 1 < self.content.len() {
                let next = self.content[self.idx + 1];
                let rest = &self.content[self.idx + 2..];

                // single line comment //
                if next == '/' {
                    if let Some(pos) = rest.iter().position(|&c| c == '\n') {
                        self.idx += pos + 2;
                    } else {
                        self.idx = self.content.len();
                    }
                    return None;

                // multi line comment /*  */
                } else if next == '*' {
                    if let Some(pos) = rest.windows(2).position(|w| w == ['*', '/']) {
                        self.idx += pos + 2;
                    } else {
                        eprintln!("unclosed multi line comment /*");
                        std::process::exit(1);
                    }
                    return None;
                }
            }

            // symbol
            return Some(Token {
                type_: TokenType::Symbol,
                content: cur.to_string(),
            });
        }

        // we consider the current string slice before encountering another symbol
        let mut token = String::new();
        while self.hasMoreTokens() && !Self::SYMBOLS.contains(&self.content[self.idx]) {
            token.push(self.content[self.idx]);
            self.idx += 1;
        }

        // keyword
        if Self::KEYWORDS.contains(&token.as_str()) {
            return Some(Token {
                type_: TokenType::Keyword,
                content: token,
            });
        }

        // integer constant
        if Self::is_all_digits(&token) {
            return Some(Token {
                type_: TokenType::IntegerConstant,
                content: token,
            });
        }

        // identifier
        if Self::is_valid_identifier(&token) {
            return Some(Token {
                type_: TokenType::Identifier,
                content: token,
            });
        } else {
            eprintln!("illegal token {}", token);
            std::process::exit(1);
        }
    }

    pub fn output(&mut self) -> String {
        let mut output = String::new();
        output.push_str("<tokens>\n");

        while self.hasMoreTokens() {
            if let Some(token) = self.advance() {
                let type_ = format!("{:?}", token.type_).to_lowercase();
                output.push_str(&format!("<{}> {} </{}>", type_, token.content, type_));
            }
        }

        output.push_str("</tokens>\n");
        return output;
    }
}
