#[derive(Debug)]
enum TokenType {
    Keyword,
    Symbol,
    IntegerConstant,
    StringConstant,
    Identifier,
}

#[derive(Debug)]
pub struct Token {
    type_: TokenType,
    content: String,
    line_number: usize,
    column_number: usize,
}

pub struct Tokenizer {
    idx: usize,
    path: String,
    content: Vec<char>,
    newline_indices: Vec<usize>,
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

    fn newline_indices(s: &str) -> Vec<usize> {
        s.char_indices()
            .filter_map(|(i, c)| if c == '\n' { Some(i) } else { None })
            .collect()
    }

    pub fn new(path: String, content: String) -> Self {
        Tokenizer {
            idx: 0,
            path,
            content: content.chars().collect(),
            newline_indices: Self::newline_indices(&content),
        }
    }

    fn line_number(&self) -> usize {
        match self.newline_indices.binary_search(&self.idx) {
            Ok(pos) => pos + 1 + 1, // exact match means index is right at a newline → next line
            Err(pos) => pos + 1,    // pos = how many newlines are before index
        }
    }

    fn column_number(&self) -> usize {
        let line_number = self.line_number();
        if line_number == 1 {
            self.idx
        } else {
            self.idx - self.newline_indices[self.line_number() - 2]
        }
    }

    fn new_token(&self, type_: TokenType, content: String) -> Token {
        Token {
            line_number: self.line_number(),
            column_number: self.column_number() - content.len(),
            type_,
            content,
        }
    }

    fn error(&self, content: &str) -> ! {
        eprintln!(
            "{} {}:{}:{}",
            content,
            self.path,
            self.line_number(),
            self.column_number()
        );
        std::process::exit(1);
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
                    return Some(
                        self.new_token(TokenType::StringConstant, string_constant.iter().collect()),
                    );
                } else {
                    self.error("unclosed double quote \"");
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
                        self.error("unclosed multi line comment /*");
                    }
                    return None;
                }
            }

            // symbol
            self.idx += 1;
            return Some(self.new_token(TokenType::Symbol, Self::escape_xml_char(cur)));
        }

        // we consider the current string slice before encountering another symbol
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
