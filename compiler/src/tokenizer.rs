pub struct Tokenizer {
    content: String,
}

impl Tokenizer {
    pub fn new(content: String) -> Self {
        Tokenizer { content }
    }

    pub fn hasMoreTokens() -> bool {}

    pub fn advance() -> Token {}
}
