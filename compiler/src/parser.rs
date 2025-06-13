use crate::tokenizer::{TokenType, Tokenizer};

pub struct Parser {
    path: String,
    tokenizer: Tokenizer,
}

impl Parser {
    pub fn new(path: String, content: String) -> Self {
        let tokenizer = Tokenizer::new(path.clone(), content);
        Parser { path, tokenizer }
    }

    fn compile_class(&mut self) {
        let t1 = self.tokenizer.expect(TokenType::Keyword, Some("class"));
    }
}
