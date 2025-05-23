#[derive(Debug)]
enum TokenType {
    Keyword,
    Symbol,
    IntegerConstant,
    StringConstant,
    Identifier,
}

pub struct Token {
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

    const SYMBOLS: [char; 24] = [
        '{', '}', '(', ')', '[', ']', '.', ',', ';', '+', '-', '*', '/', '&', '|', '<', '>', '=',
        '~', ' ', '"', '\n', '\r', '\t',
    ];

    fn is_all_digits(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_digit())
    }

    fn is_valid_identifier(s: &str) -> bool {
        s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    fn escape_xml_char(c: char) -> String {
        match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '&' => "&amp;".to_string(),
            _ => c.to_string(),
        }
    }

    fn lowercase_first_letter(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    pub fn new(content: String) -> Self {
        Tokenizer {
            idx: 0,
            content: content.chars().collect(),
        }
    }

    pub fn has_more_tokens(&self) -> bool {
        return self.idx < self.content.len();
    }

    pub fn advance(&mut self) -> Option<Token> {
        if !self.has_more_tokens() {
            return None;
        }

        let cur = self.content[self.idx];
        if Self::SYMBOLS.contains(&cur) {
            // ignore whitespace and new lines
            if [' ', '\n', '\r', '\t'].contains(&cur) {
                self.idx += 1;
                return None;
            }

            // string constant
            if cur == '"' {
                let rest = &self.content[self.idx + 1..];
                if let Some(pos) = rest.iter().position(|&c| c == '"') {
                    let string_constant = &rest[..pos];
                    self.idx += pos + 2; // skip over ""
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
                        self.idx += pos + 2; // skip over //
                    } else {
                        self.idx = self.content.len();
                    }
                    return None;

                // multi line comment /**/
                } else if next == '*' {
                    if let Some(pos) = rest.windows(2).position(|w| w == ['*', '/']) {
                        self.idx += pos + 4; // skip over /**/
                    } else {
                        eprintln!("unclosed multi line comment /*");
                        std::process::exit(1);
                    }
                    return None;
                }
            }

            // symbol
            self.idx += 1;
            return Some(Token {
                type_: TokenType::Symbol,
                content: Self::escape_xml_char(cur),
            });
        }

        // we consider the current string slice before encountering another symbol
        let mut token = String::new();
        while self.has_more_tokens() && !Self::SYMBOLS.contains(&self.content[self.idx]) {
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
            eprintln!("illegal token {:?}", token);
            std::process::exit(1);
        }
    }

    pub fn output(&mut self) -> String {
        let mut output = String::new();
        output.push_str("<tokens>\n");

        while self.has_more_tokens() {
            if let Some(token) = self.advance() {
                let type_ = Self::lowercase_first_letter(&format!("{:?}", token.type_));
                output.push_str(&format!("<{}> {} </{}>\n", type_, token.content, type_));
            }
        }

        output.push_str("</tokens>\n");
        return output;
    }
}
