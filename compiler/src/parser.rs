use crate::tokenizer::{Token, TokenType, Tokenizer};

macro_rules! expect {
    ($self_:ident, $( $pattern:pat_param )|+) => {
        let token = $self_.tokenizer.consume();
        if matches!((&token.type_, token.content.as_str()), $( $pattern )|+ ) {
            $self_.output.push_str(token.output().as_str());
        } else {
            $self_.error(token);
        }
    };
}

macro_rules! match_token {
    ($token:ident, $( $pattern:pat_param )|+) => {
        matches!(
            (&$token.type_, $token.content.as_str()),
            $( $pattern )|+
        )
    };
}

pub struct Parser {
    tokenizer: Tokenizer,
    output: String,
}

impl Parser {
    pub fn new(path: String, content: String) -> Self {
        let tokenizer = Tokenizer::new(path.clone(), content);
        Parser {
            tokenizer,
            output: String::new(),
        }
    }

    pub fn parse(&mut self) -> String {
        self.compile_class();
        self.output.clone()
    }

    fn error(&mut self, token: Token) -> ! {
        self.tokenizer.error(&format!(
            "unexpected token {:?}({})",
            token.type_, token.content
        ));
    }

    fn expect(&mut self, type_: TokenType, content: Option<&str>) -> () {
        let token = self.tokenizer.expect(type_, content);
        self.output.push_str(&token.output());
    }

    fn compile_class(&mut self) {
        self.output.push_str("<class>\n");
        self.expect(TokenType::Keyword, Some("class"));
        self.expect(TokenType::Identifier, None);
        self.expect(TokenType::Symbol, Some("{"));

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Keyword, "static" | "field") => {
                    self.compile_class_var_dec();
                }
                _ => break,
            }
        }

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Keyword, "constructor" | "function" | "method") => {
                    self.compile_subroutine_dec();
                }
                _ => break,
            }
        }

        self.expect(TokenType::Symbol, Some("}"));
        self.output.push_str("</class>\n");
    }

    fn compile_var_list(&mut self) {
        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Symbol, ",") => {
                    self.tokenizer.consume();
                    self.output.push_str(token.output().as_str());
                    self.expect(TokenType::Identifier, None);
                }
                _ => break,
            }
        }
    }

    fn compile_class_var_dec(&mut self) {
        self.output.push_str("<classVarDec>\n");
        expect!(self, (TokenType::Keyword, "static" | "field"));
        self.compile_type();
        self.expect(TokenType::Identifier, None);
        self.compile_var_list();
        self.expect(TokenType::Symbol, Some(";"));
        self.output.push_str("</classVarDec>\n");
    }

    fn compile_type(&mut self) {
        expect!(
            self,
            (TokenType::Keyword, "int" | "char" | "boolean") | (TokenType::Identifier, _)
        );
    }

    fn compile_subroutine_dec(&mut self) {
        self.output.push_str("<subroutineDec>\n");
        expect!(
            self,
            (TokenType::Keyword, "constructor" | "method" | "function")
        );

        let token = self.tokenizer.peek();
        if match_token!(token, (TokenType::Keyword, "void")) {
            self.tokenizer.consume();
            self.output.push_str(token.output().as_str());
        } else {
            self.compile_type();
        }

        self.expect(TokenType::Identifier, None);
        self.expect(TokenType::Symbol, Some("("));
        self.compile_parameter_list();
        self.expect(TokenType::Symbol, Some(")"));
        self.compile_subroutine_body();
        self.output.push_str("</subroutineDec>\n");
    }

    fn compile_parameter_list(&mut self) {
        self.output.push_str("<parameterList>\n");

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ")")) {
            self.compile_type();
            self.expect(TokenType::Identifier, None);

            while self.tokenizer.has_more_tokens() {
                let token = self.tokenizer.peek();
                match (&token.type_, token.content.as_str()) {
                    (TokenType::Symbol, ",") => {
                        self.tokenizer.consume();
                        self.output.push_str(token.output().as_str());
                        self.compile_type();
                        self.expect(TokenType::Identifier, None);
                    }
                    _ => break,
                }
            }
        }

        self.output.push_str("</parameterList>\n");
    }

    fn compile_subroutine_body(&mut self) {
        self.output.push_str("<subroutineBody>\n");
        self.expect(TokenType::Symbol, Some("{"));

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Keyword, "var") => {
                    self.compile_var_dec();
                }
                _ => break,
            }
        }

        self.compile_statements();
        self.expect(TokenType::Symbol, Some("}"));
        self.output.push_str("</subroutineBody>\n");
    }

    fn compile_var_dec(&mut self) {
        self.output.push_str("<varDec>\n");
        self.expect(TokenType::Keyword, Some("var"));
        self.compile_type();
        self.expect(TokenType::Identifier, None);
        self.compile_var_list();
        self.expect(TokenType::Symbol, Some(";"));
        self.output.push_str("</varDec>\n");
    }

    fn compile_statements(&mut self) {
        self.output.push_str("<statements>\n");
        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Keyword, "let") => {
                    self.compile_let_statement();
                }
                (TokenType::Keyword, "if") => {
                    self.compile_if_statement();
                }
                (TokenType::Keyword, "while") => {
                    self.compile_while_statement();
                }
                (TokenType::Keyword, "do") => {
                    self.compile_do_statement();
                }
                (TokenType::Keyword, "return") => {
                    self.compile_return_statement();
                }
                _ => break,
            }
        }
        self.output.push_str("</statements>\n");
    }

    fn compile_let_statement(&mut self) {
        self.output.push_str("<letStatement>\n");
        self.expect(TokenType::Keyword, Some("let"));
        self.expect(TokenType::Identifier, None);

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, "=")) {
            self.expect(TokenType::Symbol, Some("["));
            self.compile_expression();
            self.expect(TokenType::Symbol, Some("]"));
        }

        self.expect(TokenType::Symbol, Some("="));
        self.compile_expression();
        self.expect(TokenType::Symbol, Some(";"));
        self.output.push_str("</letStatement>\n");
    }

    fn compile_if_statement(&mut self) {
        self.output.push_str("<ifStatement>\n");
        self.expect(TokenType::Keyword, Some("if"));
        self.expect(TokenType::Symbol, Some("("));
        self.compile_expression();
        self.expect(TokenType::Symbol, Some(")"));
        self.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.expect(TokenType::Symbol, Some("}"));

        let token = self.tokenizer.peek();
        if match_token!(token, (TokenType::Keyword, "else")) {
            self.expect(TokenType::Keyword, Some("else"));
            self.expect(TokenType::Symbol, Some("{"));
            self.compile_statements();
            self.expect(TokenType::Symbol, Some("}"));
        }

        self.output.push_str("</ifStatement>\n");
    }

    fn compile_while_statement(&mut self) {
        self.output.push_str("<whileStatement>\n");
        self.expect(TokenType::Keyword, Some("while"));
        self.expect(TokenType::Symbol, Some("("));
        self.compile_expression();
        self.expect(TokenType::Symbol, Some(")"));
        self.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.expect(TokenType::Symbol, Some("}"));
        self.output.push_str("</whileStatement>\n");
    }

    fn compile_do_statement(&mut self) {
        self.output.push_str("<doStatement>\n");
        self.expect(TokenType::Keyword, Some("do"));
        self.expect(TokenType::Identifier, None);
        self.compile_subroutine_call();
        self.expect(TokenType::Symbol, Some(";"));
        self.output.push_str("</doStatement>\n");
    }

    fn compile_return_statement(&mut self) {
        self.output.push_str("<returnStatement>\n");
        self.expect(TokenType::Keyword, Some("return"));

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ";")) {
            self.compile_expression();
        }

        self.expect(TokenType::Symbol, Some(";"));
        self.output.push_str("</returnStatement>\n");
    }

    fn compile_expression(&mut self) {
        self.output.push_str("<expression>\n");
        self.compile_term();

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Symbol, "+" | "-" | "*" | "/" | "&" | "|" | "<" | ">" | "=") => {
                    self.tokenizer.consume();
                    self.output.push_str(token.output().as_str());
                    self.compile_term();
                }
                _ => break,
            }
        }

        self.output.push_str("</expression>\n");
    }

    fn compile_term(&mut self) {
        self.output.push_str("<term>\n");
        let token = self.tokenizer.consume();
        self.output.push_str(token.output().as_str());

        match (&token.type_, token.content.as_str()) {
            (TokenType::IntegerConstant, _)
            | (TokenType::StringConstant, _)
            | (TokenType::Keyword, "true" | "false" | "null" | "this") => {}
            (TokenType::Symbol, "-" | "~") => {
                self.compile_term();
            }
            (TokenType::Symbol, "(") => {
                self.compile_expression();
                self.expect(TokenType::Keyword, Some(")"));
            }
            (TokenType::Identifier, _) => {
                let next = self.tokenizer.peek();
                match (&next.type_, next.content.as_str()) {
                    (TokenType::Symbol, "[") => {
                        self.expect(TokenType::Symbol, Some("["));
                        self.compile_expression();
                        self.expect(TokenType::Symbol, Some("]"));
                    }
                    _ => self.compile_subroutine_call(),
                }
            }
            _ => self.error(token),
        }

        self.output.push_str("</term>\n");
    }

    fn compile_subroutine_call(&mut self) {
        let token = self.tokenizer.peek();
        match (&token.type_, token.content.as_str()) {
            (TokenType::Symbol, "(") => {
                self.expect(TokenType::Symbol, Some("("));
                self.compile_expression_list();
                self.expect(TokenType::Symbol, Some(")"));
            }
            (TokenType::Symbol, ".") => {
                self.expect(TokenType::Symbol, Some("."));
                self.expect(TokenType::Identifier, None);
                self.expect(TokenType::Symbol, Some("("));
                self.compile_expression_list();
                self.expect(TokenType::Symbol, Some(")"));
            }
            _ => self.error(token),
        }
    }

    fn compile_expression_list(&mut self) {
        self.output.push_str("<expressionList>\n");
        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ")")) {
            self.compile_expression();

            while self.tokenizer.has_more_tokens() {
                let token = self.tokenizer.peek();
                match (&token.type_, token.content.as_str()) {
                    (TokenType::Symbol, ",") => {
                        self.tokenizer.consume();
                        self.output.push_str(token.output().as_str());
                        self.compile_expression();
                    }
                    _ => break,
                }
            }
        }

        self.output.push_str("</expressionList>\n");
    }
}
