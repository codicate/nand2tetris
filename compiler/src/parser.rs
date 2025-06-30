use std::path::Path;

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
        self.writer.close();
    }

    fn error(&mut self, token: Token) -> ! {
        self.tokenizer.error(&format!(
            "unexpected token {:?}({})",
            token.type_, token.content
        ));
    }

    fn lookup_symbol(&mut self, name: &str) -> Option<Symbol> {
        if let Some(symbol) = self.func_symbols.get(name) {
            return Some(symbol);
        }
        if let Some(symbol) = self.class_symbols.get(name) {
            return Some(symbol);
        }
        None
    }

    fn compile_class(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("class"));
        self.tokenizer.expect(TokenType::Identifier, None);
        self.tokenizer.expect(TokenType::Symbol, Some("{"));

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            if match_token!(token, (TokenType::Keyword, "static" | "field")) {
                self.compile_class_var_dec();
            } else {
                break;
            }
        }

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            if match_token!(
                token,
                (TokenType::Keyword, "constructor" | "function" | "method")
            ) {
                self.compile_subroutine_dec();
            } else {
                break;
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
            if match_token!(token, (TokenType::Symbol, ",")) {
                self.tokenizer.consume();
                let token = self.tokenizer.expect(TokenType::Identifier, None);
                var_list.push(token.content);
            } else {
                break;
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
        match (&token.type_, token.content.as_str()) {
            (TokenType::Keyword, "function") => {}
            (TokenType::Keyword, "method") => {
                // For methods, we need to define 'this' as the first argument
                self.func_symbols
                    .define("this".to_string(), "int".to_string(), Kind::Arg);
                self.writer.write_push(Segment::Argument, 0);
                self.writer.write_pop(Segment::Pointer, 0); // this = pointer 0
            }
            (TokenType::Keyword, "constructor") => {
                // For constructors, we need to allocate memory for the object's fields
                let n_fields = self.class_symbols.var_count(Kind::Field);
                self.writer.write_push(Segment::Constant, n_fields);
                self.writer.write_call("Memory.alloc", 1);
                self.writer.write_pop(Segment::Pointer, 0); // this = pointer 0
            }
            _ => self.error(token),
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
        if match_token!(token, (TokenType::Symbol, ")")) {
            self.tokenizer.consume();
            return;
        }

        self.compile_parameter();
        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            if match_token!(token, (TokenType::Symbol, ",")) {
                self.tokenizer.consume();
                self.compile_parameter();
            } else {
                break;
            }
        }

        self.tokenizer.expect(TokenType::Symbol, Some(")"));
    }

    fn compile_subroutine_body(&mut self) {
        self.tokenizer.expect(TokenType::Symbol, Some("{"));

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            if match_token!(token, (TokenType::Keyword, "var")) {
                self.compile_var_dec();
            } else {
                break;
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

    /// computes base address + index and pushes the address onto the stack.
    /// For example, for `a[i]`, it computes `a + i` and pushes the address of `a[i]`
    fn compile_array_access(&mut self, name: &str) {
        self.tokenizer.expect(TokenType::Symbol, Some("["));
        let symbol = self
            .lookup_symbol(name)
            .expect(&format!("Undefined identifier: {}", name));
        self.writer
            .write_push(symbol.kind.clone().into(), symbol.index);
        self.compile_expression();
        self.writer.write_arithmetic("add"); // add index to base address
        self.tokenizer.expect(TokenType::Symbol, Some("]"));
    }

    fn compile_let_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("let"));
        let name = self.tokenizer.expect(TokenType::Identifier, None).content;

        let token = self.tokenizer.peek();
        let is_array_access = match_token!(token, (TokenType::Symbol, "["));
        if is_array_access {
            self.compile_array_access(&name);
        }

        self.tokenizer.expect(TokenType::Symbol, Some("="));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(";"));

        if is_array_access {
            // stack: [..., LHS (a[i]), RHS (value)]
            self.writer.write_pop(Segment::Temp, 0); // store RHS to temp
            self.writer.write_pop(Segment::Pointer, 1); // store LHS to pointer 1
            self.writer.write_push(Segment::Temp, 0); // push RHS back to stack
            self.writer.write_pop(Segment::That, 0); // store RHS to LHS (a[i])
        } else {
            let symbol = self
                .lookup_symbol(&name)
                .expect(&format!("Undefined identifier: {}", name));
            self.writer.write_pop(symbol.kind.into(), symbol.index);
        }
    }

    fn compile_if_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("if"));
        self.tokenizer.expect(TokenType::Symbol, Some("("));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(")"));

        self.writer.write_arithmetic("not");
        let label1 = self.writer.new_label();
        self.writer.write_if(&label1);

        self.tokenizer.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.tokenizer.expect(TokenType::Symbol, Some("}"));

        let token = self.tokenizer.peek();
        if !match_token!(token, (TokenType::Keyword, "else")) {
            self.writer.write_label(&label1);
        } else {
            self.tokenizer.expect(TokenType::Keyword, Some("else"));
            let label2 = self.writer.new_label();
            self.writer.write_goto(&label2);
            self.writer.write_label(&label1);

            self.tokenizer.expect(TokenType::Symbol, Some("{"));
            self.compile_statements();
            self.tokenizer.expect(TokenType::Symbol, Some("}"));
            self.writer.write_label(&label2);
        }
    }

    fn compile_while_statement(&mut self) {
        let label1 = self.writer.new_label();
        self.writer.write_label(&label1);

        self.tokenizer.expect(TokenType::Keyword, Some("while"));
        self.tokenizer.expect(TokenType::Symbol, Some("("));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(")"));

        self.writer.write_arithmetic("not");
        let label2 = self.writer.new_label();
        self.writer.write_if(&label2);

        self.tokenizer.expect(TokenType::Symbol, Some("{"));
        self.compile_statements();
        self.tokenizer.expect(TokenType::Symbol, Some("}"));
        self.writer.write_goto(&label1);
        self.writer.write_label(&label2);
    }

    fn compile_do_statement(&mut self) {
        self.tokenizer.expect(TokenType::Keyword, Some("do"));
        self.compile_expression();
        self.tokenizer.expect(TokenType::Symbol, Some(";"));
        self.writer.write_pop(Segment::Temp, 0); // discard return value
    }

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
            if match_token!(
                token,
                (
                    TokenType::Symbol,
                    "+" | "-" | "*" | "/" | "&" | "|" | "<" | ">" | "="
                )
            ) {
                let op = self.tokenizer.consume();
                self.compile_term();
                let command = Self::translate_op(&op.content);
                self.writer.write_arithmetic(command);
            } else {
                break;
            }
        }
    }

    fn compile_identifier(&mut self, identifier: String) {
        let next = self.tokenizer.peek();
        match (&next.type_, next.content.as_str()) {
            (TokenType::Symbol, "[") => {
                self.compile_array_access(&identifier);
                self.writer.write_pop(Segment::Pointer, 1);
                self.writer.write_push(Segment::That, 0); // push actual value onto stack
            }
            (TokenType::Symbol, "(") => {
                // pass 'this' as the first argument, this = pointer 0
                self.writer.write_push(Segment::Pointer, 0);
                let n_args = self.compile_argument_list();
                self.writer.write_call(
                    &format!("{}.{}", self.writer.class_name, identifier),
                    n_args + 1,
                );
            }
            (TokenType::Symbol, ".") => {
                let mut class_name = identifier;
                let symbol = self.lookup_symbol(&class_name);

                // If the identifier is an object instance, hence a method call,
                // we need to push the object instance onto the stack as 'this'.
                let is_method = symbol.is_some();
                if is_method {
                    let symbol = symbol.clone().unwrap();
                    self.writer.write_push(symbol.kind.into(), symbol.index);
                }

                self.tokenizer.expect(TokenType::Symbol, Some("."));
                let func_name = self.tokenizer.expect(TokenType::Identifier, None).content;
                let mut n_args = self.compile_argument_list();

                if is_method {
                    n_args += 1; // pass 'this' as the first argument
                    class_name = symbol.unwrap().type_;
                }

                self.writer
                    .write_call(&format!("{}.{}", class_name, func_name), n_args);
            }
            _ => {
                let symbol = self
                    .lookup_symbol(&identifier)
                    .expect(&format!("Undefined identifier: {}", identifier));
                self.writer.write_push(symbol.kind.into(), symbol.index);
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
            (TokenType::StringConstant, _) => {
                let value = token.content.trim_matches('"');
                self.writer.write_push(Segment::Constant, value.len());
                self.writer.write_call("String.new", 1);

                for c in value.chars() {
                    self.writer.write_push(Segment::Constant, c as u32 as usize);
                    self.writer.write_call("String.appendChar", 2);
                }
            }
            (TokenType::Keyword, "true") => {
                self.writer.write_push(Segment::Constant, 1);
                self.writer.write_arithmetic("neg");
            }
            (TokenType::Keyword, "false" | "null") => {
                self.writer.write_push(Segment::Constant, 0);
            }
            (TokenType::Keyword, "this") => {
                self.writer.write_push(Segment::Pointer, 0); // this = pointer 0
            }
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
                self.compile_identifier(token.content);
            }
            _ => self.error(token),
        }
    }

    fn compile_argument_list(&mut self) -> usize {
        self.tokenizer.expect(TokenType::Symbol, Some("("));

        let token = self.tokenizer.peek();
        if match_token!(token, (TokenType::Symbol, ")")) {
            self.tokenizer.consume();
            return 0;
        }

        self.compile_expression();
        let mut n_args = 1;

        while self.tokenizer.has_more_tokens() {
            let token = self.tokenizer.peek();
            if match_token!(token, (TokenType::Symbol, ",")) {
                self.tokenizer.consume();
                self.compile_expression();
                n_args += 1;
            } else {
                break;
            }
        }

        self.tokenizer.expect(TokenType::Symbol, Some(")"));
        n_args
    }
}
