use flux_ast::{
    BinaryOp, ExceptHandler, Expr, Literal, Param, Program, Span, Stmt, UnaryOp,
};
use flux_lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("parse error at line {line}, column {column}: {message}")]
    SyntaxError {
        line: usize,
        column: usize,
        message: String,
    },
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let start = self.current_span();
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.check(&TokenKind::Eof) {
            statements.push(self.parse_statement()?);
            self.skip_newlines();
        }

        let end = self.previous_span();
        Ok(Program {
            statements,
            span: Span::merge(&start, &end),
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let span = self.current_span();

        match &self.peek_kind() {
            TokenKind::Print => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Stmt::Print {
                    expr,
                    span: self.span_from(&span),
                })
            }
            TokenKind::Ask => {
                self.advance();
                // Support both `ASK "prompt"` and `ASK = "prompt"` — the
                // '=' is optional so people who think of ASK like an
                // assignment (`ASK = "What is your name?"`) still work.
                if self.check(&TokenKind::Assign) {
                    self.advance();
                }
                let message = self.parse_expression()?;
                Ok(Stmt::Ask {
                    message,
                    span: self.span_from(&span),
                })
            }
            TokenKind::Def => self.parse_function_def(span),
            TokenKind::Return => {
                self.advance();
                let expr = if self.is_expression_start() {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                Ok(Stmt::Return {
                    expr,
                    span: self.span_from(&span),
                })
            }
            TokenKind::If => self.parse_if(span),
            TokenKind::While => self.parse_while(span),
            TokenKind::For => self.parse_for(span),
            TokenKind::Break => {
                self.advance();
                Ok(Stmt::Break {
                    span: self.span_from(&span),
                })
            }
            TokenKind::Continue => {
                self.advance();
                Ok(Stmt::Continue {
                    span: self.span_from(&span),
                })
            }
            TokenKind::Pass => {
                self.advance();
                Ok(Stmt::Pass {
                    span: self.span_from(&span),
                })
            }
            TokenKind::Import => self.parse_import(span),
            TokenKind::From => self.parse_from_import(span),
            TokenKind::Class => self.parse_class_def(span),
            TokenKind::Try => self.parse_try(span),
            TokenKind::Raise => {
                self.advance();
                let expr = if self.is_expression_start() {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                Ok(Stmt::Raise {
                    expr,
                    span: self.span_from(&span),
                })
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.check(&TokenKind::Assign) {
                    self.advance();
                    let expr = self.parse_expression()?;
                    Ok(Stmt::Assign {
                        name,
                        expr,
                        span: self.span_from(&span),
                    })
                } else {
                    self.pos -= 1;
                    self.parse_assign_or_expr(span)
                }
            }
            _ if self.is_expression_start() => self.parse_assign_or_expr(span),
            _ => self.error("expected statement"),
        }
    }

    fn parse_assign_or_expr(&mut self, span: Span) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression()?;

        if self.check(&TokenKind::Assign) {
            self.advance();
            let value = self.parse_expression()?;

            return match expr {
                Expr::Variable { name, .. } => Ok(Stmt::Assign {
                    name,
                    expr: value,
                    span: self.span_from(&span),
                }),
                Expr::Member { object, member, .. } => Ok(Stmt::MemberAssign {
                    object: *object,
                    member,
                    expr: value,
                    span: self.span_from(&span),
                }),
                Expr::Index { object, index, .. } => Ok(Stmt::IndexAssign {
                    object: *object,
                    index: *index,
                    expr: value,
                    span: self.span_from(&span),
                }),
                _ => self.error("invalid assignment target"),
            };
        }

        Ok(Stmt::Expr {
            expr,
            span: self.span_from(&span),
        })
    }

        fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(&TokenKind::Colon, "expected ':' after block header")?;

        // Block headers require a newline before indentation
        self.expect(&TokenKind::Newline, "expected newline after ':'")?;

        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut body = Vec::new();

        self.skip_newlines();

        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::Eof) {
            body.push(self.parse_statement()?);
            self.skip_newlines();
        }

        if self.check(&TokenKind::Dedent) {
            self.advance();
        }

        Ok(body)
    }

    fn parse_function_def(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let name = self.expect_identifier("expected function name")?;
        self.expect(&TokenKind::LParen, "expected '(' after function name")?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;
        let body = self.parse_block()?;

        Ok(Stmt::FunctionDef {
            name,
            params,
            body,
            span: self.span_from(&span),
        })
    }

    fn parse_class_def(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let name = self.expect_identifier("expected class name")?;
        let body = self.parse_block()?;

        Ok(Stmt::ClassDef {
            name,
            body,
            span: self.span_from(&span),
        })
    }

    fn parse_if(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let condition = self.parse_expression()?;
        let then_body = self.parse_block()?;

        let mut elif_branches = Vec::new();
        while self.check(&TokenKind::Elif) {
            self.advance();
            let elif_cond = self.parse_expression()?;
            let elif_body = self.parse_block()?;
            elif_branches.push((elif_cond, elif_body));
        }

        let else_body = if self.check(&TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_body,
            elif_branches,
            else_body,
            span: self.span_from(&span),
        })
    }

    fn parse_while(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Stmt::While {
            condition,
            body,
            span: self.span_from(&span),
        })
    }

    fn parse_for(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let variable = self.expect_identifier("expected loop variable")?;
        self.expect(&TokenKind::In, "expected 'IN' after loop variable")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Stmt::For {
            variable,
            iterable,
            body,
            span: self.span_from(&span),
        })
    }

    fn parse_import(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let module = self.expect_identifier("expected module name")?;
        let alias = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier("expected alias after AS")?)
        } else {
            None
        };
        Ok(Stmt::Import {
            module,
            alias,
            span: self.span_from(&span),
        })
    }

    fn parse_from_import(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let module = self.expect_identifier("expected module name")?;
        self.expect(&TokenKind::Import, "expected 'IMPORT' after module name")?;
        let names = self.parse_import_names()?;
        Ok(Stmt::FromImport {
            module,
            names,
            span: self.span_from(&span),
        })
    }

    fn parse_import_names(&mut self) -> Result<Vec<(String, Option<String>)>, ParseError> {
        let mut names = Vec::new();
        loop {
            let name = self.expect_identifier("expected import name")?;
            let alias = if self.check(&TokenKind::As) {
                self.advance();
                Some(self.expect_identifier("expected alias after AS")?)
            } else {
                None
            };
            names.push((name, alias));
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(names)
    }

    fn parse_try(&mut self, span: Span) -> Result<Stmt, ParseError> {
        self.advance();
        let body = self.parse_block()?;
        let mut handlers = Vec::new();

        while self.check(&TokenKind::Except) {
            handlers.push(self.parse_except_handler()?);
        }

        let finally_body = if self.check(&TokenKind::Finally) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::Try {
            body,
            handlers,
            finally_body,
            span: self.span_from(&span),
        })
    }

    fn parse_except_handler(&mut self) -> Result<ExceptHandler, ParseError> {
        let span = self.current_span();
        self.advance();

        let exception_type = if self.is_identifier() {
            Some(self.expect_identifier("expected exception type")?)
        } else {
            None
        };

        let binding = if self.check(&TokenKind::As) {
            self.advance();
            Some(self.expect_identifier("expected binding after AS")?)
        } else {
            None
        };

        let body = self.parse_block()?;
        Ok(ExceptHandler {
            exception_type,
            binding,
            body,
            span: self.span_from(&span),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let span = self.current_span();
            let name = self.expect_identifier("expected parameter name")?;
            let type_hint = if self.check(&TokenKind::Colon) {
                self.advance();
                Some(self.expect_identifier("expected type hint")?)
            } else {
                None
            };
            let default = if self.check(&TokenKind::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            params.push(Param {
                name,
                type_hint,
                default,
                span: self.span_from(&span),
            });
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_and()?;
        while self.check(&TokenKind::Or) {
            let op = BinaryOp::Or;
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_and()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not()?;
        while self.check(&TokenKind::And) {
            let op = BinaryOp::And;
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_not()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Not) {
            let span = self.current_span();
            self.advance();
            let operand = self.parse_not()?;
            let end = operand.span().clone();
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
                span: Span::merge(&span, &end),
            });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_term()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::NotEq => BinaryOp::NotEq,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => break,
            };
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_term()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_factor()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_factor()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_power()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::FloorDiv => BinaryOp::FloorDiv,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_power()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;
        if self.check(&TokenKind::StarStar) {
            let span = expr.span().clone();
            self.advance();
            let right = self.parse_power()?;
            let end = right.span().clone();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Pow,
                right: Box::new(right),
                span: Span::merge(&span, &end),
            };
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(&TokenKind::Minus) {
            let span = self.current_span();
            self.advance();
            let operand = self.parse_unary()?;
            let end = operand.span().clone();
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                operand: Box::new(operand),
                span: Span::merge(&span, &end),
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
                TokenKind::LParen => {
                    let span = expr.span().clone();
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&TokenKind::RParen, "expected ')' after arguments")?;
                    let end = self.previous_span();
                    expr = Expr::Call {
                        callee: Box::new(expr),
                        args,
                        span: Span::merge(&span, &end),
                    };
                }
                TokenKind::LBracket => {
                    let span = expr.span().clone();
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(&TokenKind::RBracket, "expected ']' after index")?;
                    let end = self.previous_span();
                    expr = Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                        span: Span::merge(&span, &end),
                    };
                }
                TokenKind::Dot => {
                    let span = expr.span().clone();
                    self.advance();
                    let member = self.expect_identifier("expected member name after '.'")?;
                    let end = self.previous_span();
                    expr = Expr::Member {
                        object: Box::new(expr),
                        member,
                        span: Span::merge(&span, &end),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.current_span();
        match self.peek_kind() {
            TokenKind::String(value) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::String(value),
                    span: self.span_from(&span),
                })
            }
            TokenKind::Integer(value) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Integer(value),
                    span: self.span_from(&span),
                })
            }
            TokenKind::Float(value) => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Float(value),
                    span: self.span_from(&span),
                })
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Boolean(true),
                    span: self.span_from(&span),
                })
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::Boolean(false),
                    span: self.span_from(&span),
                })
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::Literal {
                    value: Literal::None,
                    span: self.span_from(&span),
                })
            }
            TokenKind::Type => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after TYPE")?;
                let value = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')' after TYPE argument")?;
                let end_span = self.previous_span();
                Ok(Expr::Call {
                    callee: Box::new(Expr::Variable {
                        name: "type".to_string(),
                        span: self.span_from(&span),
                    }),
                    args: vec![value],
                    span: Span::merge(&span, &end_span),
                })
            }
            TokenKind::Range => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after RANGE")?;
                let start = self.parse_expression()?;
                let end = if self.check(&TokenKind::Comma) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(&TokenKind::RParen, "expected ')' after RANGE arguments")?;
                let end_span = self.previous_span();
                Ok(Expr::Call {
                    callee: Box::new(Expr::Variable {
                        name: "__range__".to_string(),
                        span: self.span_from(&span),
                    }),
                    args: if let Some(end_expr) = end {
                        vec![start, end_expr]
                    } else {
                        vec![start]
                    },
                    span: Span::merge(&span, &end_span),
                })
            }
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable {
                    name,
                    span: self.span_from(&span),
                })
            }
            TokenKind::SelfKw => {
                self.advance();
                Ok(Expr::Variable {
                    name: "self".to_string(),
                    span: self.span_from(&span),
                })
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')' after expression")?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let elements = self.parse_list()?;
                self.expect(&TokenKind::RBracket, "expected ']' after array")?;
                Ok(Expr::Array {
                    elements,
                    span: self.span_from(&span),
                })
            }
            TokenKind::LBrace => {
                self.advance();
                let entries = self.parse_dict()?;
                self.expect(&TokenKind::RBrace, "expected '}' after dictionary")?;
                Ok(Expr::Dict {
                    entries,
                    span: self.span_from(&span),
                })
            }
            _ => self.error("expected expression"),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expression()?);
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(args)
    }

    fn parse_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut items = Vec::new();
        if self.check(&TokenKind::RBracket) {
            return Ok(items);
        }
        loop {
            items.push(self.parse_expression()?);
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(items)
    }

    fn parse_dict(&mut self) -> Result<Vec<(Expr, Expr)>, ParseError> {
        let mut entries = Vec::new();
        if self.check(&TokenKind::RBrace) {
            return Ok(entries);
        }
        loop {
            let key = self.parse_expression()?;
            self.expect(&TokenKind::Colon, "expected ':' in dictionary entry")?;
            let value = self.parse_expression()?;
            entries.push((key, value));
            if self.check(&TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(entries)
    }

    fn is_expression_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::String(_)
                | TokenKind::Integer(_)
                | TokenKind::Float(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None
                | TokenKind::Identifier(_)
                | TokenKind::SelfKw
                | TokenKind::Not
                | TokenKind::Minus
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::Range
                | TokenKind::Type
        )
    }

    fn is_identifier(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Identifier(_))
    }

    fn expect_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        match self.peek_kind() {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            TokenKind::Init => {
                self.advance();
                Ok("init".to_string())
            }
            _ => self.error(message),
        }
    }

    fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<(), ParseError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            self.error(message)
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(kind)
            && Parser::kind_matches(&self.peek_kind(), kind)
    }

    fn kind_matches(a: &TokenKind, b: &TokenKind) -> bool {
        match (a, b) {
            (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (TokenKind::Integer(_), TokenKind::Integer(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            _ => a == b,
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn peek_kind(&self) -> TokenKind {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].kind.clone()
        } else {
            TokenKind::Eof
        }
    }

    fn previous_span(&self) -> Span {
        let token = &self.tokens[self.pos.saturating_sub(1)];
        Span {
            start_line: token.line,
            start_col: token.column,
            end_line: token.line,
            end_col: token.column + token.lexeme.len(),
        }
    }

    fn current_span(&self) -> Span {
        let token = &self.tokens[self.pos.min(self.tokens.len().saturating_sub(1))];
        Span {
            start_line: token.line,
            start_col: token.column,
            end_line: token.line,
            end_col: token.column,
        }
    }

    fn span_from(&self, start: &Span) -> Span {
        Span::merge(start, &self.previous_span())
    }

    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    fn error<T>(&self, message: &str) -> Result<T, ParseError> {
        let token = &self.tokens[self.pos.min(self.tokens.len().saturating_sub(1))];
        Err(ParseError::SyntaxError {
            line: token.line,
            column: token.column,
            message: message.to_string(),
        })
    }
}

trait ExprSpan {
    fn span(&self) -> &Span;
}

impl ExprSpan for Expr {
    fn span(&self) -> &Span {
        match self {
            Expr::Literal { span, .. }
            | Expr::Variable { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Index { span, .. }
            | Expr::Member { span, .. }
            | Expr::Array { span, .. }
            | Expr::Dict { span, .. }
            | Expr::Lambda { span, .. } => span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flux_lexer::lex;

    #[test]
    fn parses_print_statement() {
        let tokens = lex(r#"PRINT "Hello""#).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parses_ask_statement_without_equals() {
        let tokens = lex(r#"ASK "What is your name?""#).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Ask { .. }));
    }

    #[test]
    fn parses_ask_statement_with_equals() {
        let tokens = lex(r#"ASK = "What is your name?""#).unwrap();
        let program = parse(tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Ask { .. }));
    }
}
