use crate::tokenizer::{Token, TokenType, Tokenizer};

pub struct Parser {
    path: String,
    tokenizer: Tokenizer,
    output: String,
}

impl Parser {
    pub fn new(path: String, content: String) -> Self {
        let tokenizer = Tokenizer::new(path.clone(), content);
        Parser { path, tokenizer, output:String::new()}
    }

    pub fn parse(&mut self) -> String {
        self.compile_class();
        self.output.clone()
    }

    fn expect(&mut self,type_: TokenType, content: Option<&str>) -> () {
        let token = self.tokenizer.expect(type_,content);
        self.output.push_str(&token.output());
    }

    fn compile_class(&mut self) {
        self.output.push_str("<class>\n");
        self.expect(TokenType::Keyword, Some("class"));
        self.expect(TokenType::Identifier, None);
        self.expect(TokenType::Symbol, Some("{"));
        self.compile_classVarDec();
        self.compile_subroutineDec();
        self.expect(TokenType::Symbol, Some("}"));
        self.output.push_str("</class>\n");
    }

    fn compile_classVarDec(&mut self) {}

    fn compile_subroutineDec(&mut self) {}
}
