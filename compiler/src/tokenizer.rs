#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword,
    Symbol,
    IntegerConstant,
    StringConstant,
    Identifier,
}

#[derive(Debug, Clone)]
pub struct Token {
    type_: TokenType,
    content: String,
    line_number: usize,
    column_number: usize,
}

impl Token {
    fn lowercase_first_letter(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
    pub fn output(&self) -> String {
        let type_ = Self::lowercase_first_letter(&format!("{:?}", self.type_));
        format!("<{}> {} </{}>\n", type_, self.content, type_)
    }
}

pub struct Tokenizer {
    path: String,
    content: Vec<char>,
    idx: usize,
    line_number: usize,
    line_start_idx: usize, // (current) idx - line_start_idx = column_number
    peeked: Option<Token>,
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

    pub fn new(path: String, content: String) -> Self {
        Tokenizer {
            path,
            content: content.chars().collect(),
            peeked: None,
            idx: 0,
            line_number: 0,
            line_start_idx:0
        }
    }

    fn new_token(&self, type_: TokenType, content: String) -> Token {
        Token {
            line_number: self.line_number,
            column_number: self.idx - self.line_start_idx,
            type_,
            content,
        }
    }

    fn error(&self, content: &str) -> ! {
        eprintln!(
            "{} {}:{}:{}",
            content,
            self.path,
            self.line_number,
            self.idx - self.line_start_idx // column_number
        );
        std::process::exit(1);
    }

    pub fn has_more_tokens(&self) -> bool {
        return self.idx < self.content.len();
    }

    fn handle_whitespace(&mut self) -> Option<Token> {
        self.idx += 1;
        None
    }

    // ignores CR (MacOS pre-OSX) which may cause bug with line/col calculation
    fn handle_newline(&mut self) -> Option<Token> {
        self.idx += 1;
        self.line_number += 1;
        None
    }

    fn handle_string_constant(&mut self) -> Option<Token> {
        let rest = &self.content[self.idx + 1..];
        if let Some(pos) = rest.iter().position(|&c| c == '"') {
            let string_constant = &rest[..pos];
            self.idx += pos + 2; // skip over opening and closing quotes
            Some(self.new_token(TokenType::StringConstant, string_constant.iter().collect()))
        } else {
            self.error("unclosed double quote \"");
        }
    }

    fn handle_single_line_comment(&mut self) -> Option<Token> {
        let rest = &self.content[self.idx + 2..];
        if let Some(pos) = rest.iter().position(|&c| c == '\n') {
            self.idx += pos + 2;
        } else {
            self.idx = self.content.len();
        }
        None
    }

    fn handle_multi_line_comment(&mut self) -> Option<Token> {
        let rest = &self.content[self.idx + 2..];
        if let Some(pos) = rest.windows(2).position(|w| w == ['*', '/']) {
            self.idx += pos + 4;
        } else {
            self.error("unclosed multi line comment /*");
        }
        None
    }

    fn handle_slash(&mut self) -> Option<Token> {
        let next = self.content[self.idx + 1];
        match next {
            '/' => self.handle_single_line_comment(),
            '*' => self.handle_multi_line_comment(),
            _ => self.handle_general_symbol('/'),
        }
    }

    fn handle_general_symbol(&mut self, cur: char) -> Option<Token> {
        self.idx += 1;
        Some(self.new_token(TokenType::Symbol, Self::escape_xml_char(cur)))
    }

    // Consumes the next token and returns it
    // Will return None if the next token is whitespace or a comment! (or no more tokens)
    fn consume(&mut self) -> Option<Token> {
        let cur = self.content[self.idx];

        // whitespace, comments, symbols, and string constants
        if Self::SYMBOLS.contains(&cur) {
            return match cur {
                ' ' | '\t' | '\r' => self.handle_whitespace(),
                '\n' => self.handle_newline(),
                '"' => self.handle_string_constant(),
                '/' if self.idx + 1 < self.content.len() => self.handle_slash(),
                _ => self.handle_general_symbol(cur),
            };
        }

        // we consider the current string slice before encountering another symbol or whitespace
        let mut token = String::new();
        while self.has_more_tokens() && !Self::SYMBOLS.contains(&self.content[self.idx]) {
            token.push(self.content[self.idx]);
            self.idx += 1;
        }

        // keyword
        if Self::KEYWORDS.contains(&token.as_str()) {
            return Some(self.new_token(TokenType::Keyword, token));
        }

        // integer constant
        if Self::is_all_digits(&token) {
            return Some(self.new_token(TokenType::IntegerConstant, token));
        }

        // identifier
        if Self::is_valid_identifier(&token) {
            return Some(self.new_token(TokenType::Identifier, token));
        } else {
            self.error(&format!("illegal token {:?}", token));
        }
    }

    // Wraps `consume` to skip over whitespace and comments to always return a valid token
    pub fn peek(&mut self) -> Option<Token> {
        if self.peeked.is_some() {
            return self.peeked.clone();
        }
        while self.has_more_tokens() {
            if let Some(token) = self.consume() {
                self.peeked = Some(token.clone());
                return Some(token);
            }
        }
        return None;
    }

    pub fn expect(&mut self, type_: TokenType, content: Option<&str>) -> Token {
        if let Some(token) = self.peek() {
            if token.type_ == type_ && content.map_or(true, |c| c == token.content) {
                self.peeked = None; // Clear peeked token after consuming
                return token;
            } else {
                self.error(&format!(
                    "expected {:?}({}), found {:?}({})",
                    type_,
                    content.unwrap_or("ANY"),
                    token.type_,
                    token.content
                ));
            }
        } else {
            self.error(&format!("expected {:?}, but no more tokens", type_));
        }
    }

    pub fn output(&mut self) -> String {
        let mut output = String::new();
        output.push_str("<tokens>\n");

        while self.has_more_tokens() {
            if let Some(token) = self.consume() {
                output.push_str(&token.output());
            }
        }

        output.push_str("</tokens>\n");
        output
    }
}
