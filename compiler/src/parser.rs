use core::net;
use std::path::{Path, PathBuf};

use crate::{
    symbol_table::{Kind, Symbol, SymbolTable},
    tokenizer::{Token, TokenType, Tokenizer},
    writer::{Segment, Writer},
};

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
    writer: Writer,
    class_symbols: SymbolTable,
    func_symbols: SymbolTable,
}

impl Parser {
    fn translate_op(op: &str) -> &'static str {
        match op {
            "+" => "add",
            "-" => "sub",
            "*" => "call Math.multiply 2",
            "/" => "call Math.divide 2",
            "&" => "and",
            "|" => "or",
            "<" => "lt",
            ">" => "gt",
            "=" => "eq",
            _ => panic!(),
        }
    }

    pub fn new(path: &Path, content: String) -> Self {
        let tokenizer = Tokenizer::new(path, content);
        let writer = Writer::new(path);
        Parser {
            tokenizer,
            writer,
            class_symbols: SymbolTable::new(),
            func_symbols: SymbolTable::new(),
        }
    }

    pub fn parse(&mut self) -> () {
        self.compile_class();
    }

    fn error(&mut self, token: Token) -> ! {
        self.tokenizer.error(&format!(
            "unexpected token {:?}({})",
            token.type_, token.content
        ));
    }

    fn lookup_symbol(&mut self, name: &str) -> (Kind, usize) {
        let kind = self.func_symbols.kind_of(name);
        if kind != Kind::None {
            return (kind, self.func_symbols.index_of(name));
        }

        let kind = self.class_symbols.kind_of(name);
        if kind != Kind::None {
            return (kind, self.class_symbols.index_of(name));
        }

        (Kind::None, 0)
    }

    fn compile_class(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("class"));
        self.tokenizer.expect(TokenType::Identifier, None);
        self.tokenizer.expect(TokenType::Symbol, Some("{"));

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

        self.tokenizer.expect(TokenType::Symbol, Some("}"));
    }

    fn compile_var_list(&mut self) -> Vec<String> {
        let mut var_list: Vec<String> = Vec::new();
        let token = self.tokenizer.expect(TokenType::Identifier, None);
        var_list.push(token.content);

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Symbol, ",") => {
                    self.tokenizer.consume();
                    let token = self.tokenizer.expect(TokenType::Identifier, None);
                    var_list.push(token.content);
                }
                _ => break,
            }
        }
        var_list
    }

    fn compile_class_var_dec(&mut self) {
        let kind_ = self.tokenizer.consume();
        if !match_token!(kind_, (TokenType::Keyword, "static" | "field")) {
            self.error(kind_);
        }

        let kind = Kind::from_str(kind_.content.as_str());
        let type_ = self.compile_type();
        let var_list = self.compile_var_list();
        self.tokenizer.expect(TokenType::Symbol, Some(";"));

        for name in var_list.into_iter() {
            self.class_symbols.define(name, type_.clone(), kind.clone());
        }
    }

    fn compile_type(&mut self) -> String {
        let token = self.tokenizer.consume();
        if !match_token!(
            token,
            (TokenType::Keyword, "int" | "char" | "boolean") | (TokenType::Identifier, _)
        ) {
            self.error(token);
        }
        token.content
    }

    fn compile_subroutine_dec(&mut self) {
        let token = self.tokenizer.consume();
        if !match_token!(
            token,
            (TokenType::Keyword, "constructor" | "method" | "function")
        ) {
            self.error(token);
        }

        let token = self.tokenizer.peek();
        if match_token!(token, (TokenType::Keyword, "void")) {
            self.tokenizer.consume();
        } else {
            self.compile_type();
        }

        let name = self.tokenizer.expect(TokenType::Identifier, None);
        self.compile_parameter_list();
        self.compile_subroutine_body();

        let n_vars = self.func_symbols.var_count(Kind::Var);
        self.writer.write_function(&name.content, n_vars);
        self.func_symbols.reset();
    }

    fn compile_parameter(&mut self) {
        let type_ = self.compile_type();
        let name = self.tokenizer.expect(TokenType::Identifier, None);
        self.func_symbols.define(name.content, type_, Kind::Arg);
    }

    fn compile_parameter_list(&mut self) {
        self.tokenizer.expect(TokenType::Symbol, Some("("));
        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ")")) {
            self.compile_parameter();

            while self.tokenizer.has_more_tokens() {
                let token = self.tokenizer.peek();
                match (&token.type_, token.content.as_str()) {
                    (TokenType::Symbol, ",") => {
                        self.tokenizer.consume();
                        self.compile_parameter();
                    }
                    _ => break,
                }
            }
        }
        self.tokenizer.expect(TokenType::Symbol, Some(")"));
    }

    fn compile_subroutine_body(&mut self) {
        self.tokenizer.expect(TokenType::Symbol, Some("{"));

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
        self.tokenizer.expect(TokenType::Symbol, Some("}"));
    }

    fn compile_var_dec(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("var"));
        let type_ = self.compile_type();
        let var_list = self.compile_var_list();
        self.tokenizer.expect(TokenType::Symbol, Some(";"));

        for name in var_list.into_iter() {
            self.func_symbols.define(name, type_.clone(), Kind::Var);
        }
    }

    fn compile_statements(&mut self) {
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
    }

    fn compile_let_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("let"));
        let name = self.tokenizer.expect(TokenType::Identifier, None).content;

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, "=")) {
            self.tokenizer.expect(TokenType::Symbol, Some("["));
            self.compile_expression();
            self.tokenizer.expect(TokenType::Symbol, Some("]"));
        }

        self.tokenizer.expect(TokenType::Symbol, Some("="));
        self.compile_expression();
        let (kind, index) = self.lookup_symbol(&name);
        self.writer.write_pop(kind.into(), index); // pop to local variable
        self.tokenizer.expect(TokenType::Symbol, Some(";"));
    }

    fn compile_if_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("if"));
        self.tokenizer.expect(TokenType::Symbol, Some("(`"));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(")"));
        self.tokenizer.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.tokenizer.expect(TokenType::Symbol, Some("}"));

        let token = self.tokenizer.peek();
        if match_token!(token, (TokenType::Keyword, "else")) {
            self.tokenizer.expect(TokenType::Keyword, Some("else"));
            self.tokenizer.expect(TokenType::Symbol, Some("{"));
            self.compile_statements();
            self.tokenizer.expect(TokenType::Symbol, Some("}"));
        }
    }

    fn compile_while_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("while"));
        self.tokenizer.expect(TokenType::Symbol, Some("("));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(")"));
        self.tokenizer.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.tokenizer.expect(TokenType::Symbol, Some("}"));
    }

    // done
    fn compile_do_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("do"));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(";"));
        self.writer.write_pop(Segment::Temp, 0); // discard return value
    }

    // done
    fn compile_return_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("return"));

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ";")) {
            self.compile_expression();
        } else {
            self.writer.write_push(Segment::Constant, 0);
        }

        self.tokenizer.expect(TokenType::Symbol, Some(";"));
        self.writer.write_return();
    }

    fn compile_expression(&mut self) {
        self.compile_term();

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            match (&token.type_, token.content.as_str()) {
                (TokenType::Symbol, "+" | "-" | "*" | "/" | "&" | "|" | "<" | ">" | "=") => {
                    let op = self.tokenizer.consume();
                    self.compile_term();
                    let command = Self::translate_op(&op.content);
                    self.writer.write_arithmetic(command);
                }
                _ => break,
            }
        }
    }

    fn compile_term(&mut self) {
        let token = self.tokenizer.consume();

        match (&token.type_, token.content.as_str()) {
            (TokenType::IntegerConstant, _) => {
                let value: usize = token.content.parse().unwrap();
                self.writer.write_push(Segment::Constant, value);
            }
            (TokenType::StringConstant, _)
            | (TokenType::Keyword, "true" | "false" | "null" | "this") => {}
            (TokenType::Symbol, "-") => {
                self.compile_term();
                self.writer.write_arithmetic("neg");
            }
            (TokenType::Symbol, "~") => {
                self.compile_term();
                self.writer.write_arithmetic("not");
            }
            (TokenType::Symbol, "(") => {
                self.compile_expression();
                self.tokenizer.expect(TokenType::Symbol, Some(")"));
            }
            (TokenType::Identifier, _) => {
                let next = self.tokenizer.peek();
                match (&next.type_, next.content.as_str()) {
                    (TokenType::Symbol, "[") => {
                        self.tokenizer.expect(TokenType::Symbol, Some("["));
                        self.compile_expression();
                        self.tokenizer.expect(TokenType::Symbol, Some("]"));
                    }
                    (TokenType::Symbol, "(") => {
                        // pass 'this' as the first argument, this = pointer 0
                        self.writer.write_push(Segment::Pointer, 0);
                        let n_args = self.compile_argument_list();
                        self.writer.write_call(&token.content, n_args + 1);
                    }
                    (TokenType::Symbol, ".") => {
                        let (kind, index) = self.lookup_symbol(&token.content);

                        // If the identifier is an object instance, hence a method call,
                        // we need to push the object instance onto the stack as 'this'.
                        let is_method = kind != Kind::None;
                        if is_method {
                            self.writer.write_push(kind.into(), index);
                        }

                        self.tokenizer.expect(TokenType::Symbol, Some("."));
                        let func_name = self.tokenizer.expect(TokenType::Identifier, None);
                        let mut n_args = self.compile_argument_list();

                        if is_method {
                            n_args += 1; // pass 'this' as the first argument
                        }

                        self.writer.write_call(
                            &format!("{}.{}", token.content, func_name.content),
                            n_args,
                        );
                    }
                    _ => {
                        let (kind, index) = self.lookup_symbol(&token.content);
                        if kind == Kind::None {
                            self.tokenizer
                                .error(&format!("Undefined identifier: {}", token.content));
                        }
                        self.writer.write_push(kind.into(), index);
                    }
                }
            }
            _ => self.error(token),
        }
    }

    fn compile_argument_list(&mut self) -> usize {
        self.tokenizer.expect(TokenType::Symbol, Some("("));
        let mut n_args = 0;

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Symbol, ")")) {
            self.compile_expression();
            n_args += 1;

            while self.tokenizer.has_more_tokens() {
                let token = self.tokenizer.peek();
                match (&token.type_, token.content.as_str()) {
                    (TokenType::Symbol, ",") => {
                        self.tokenizer.consume();
                        self.compile_expression();
                        n_args += 1;
                    }
                    _ => break,
                }
            }
        }

        self.tokenizer.expect(TokenType::Symbol, Some(")"));
        n_args
    }
}
