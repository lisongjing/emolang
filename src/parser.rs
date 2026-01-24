use std::{cell::RefCell, collections::HashMap, fmt::Debug, num::{ParseFloatError, ParseIntError}, rc::Rc};

use crate::{
    lexer::{Lexer, Token, TokenType},
    util::StatefulVector,
};

pub enum Precedence {
    Lowest,
    Equals,      // 🟰/❗🟰
    LessGreater, // ▶️/▶️🟰/◀️/◀️🟰
    Plus,        // ➕/➖
    Product,     // ✖️/➗/〰️
    Prefix,      // ➖x/⏸️x
    Call,        // fn🌜🌛
}

pub trait Node: Debug {
    fn token_literal(&self) -> &str;
    fn string(&self) -> String;
}

pub trait Statement: Node {}

pub trait Expression: Node {}

#[derive(Debug)]
pub struct Program {
    statements: Vec<Box<dyn Statement>>,
}

impl Node for Program {
    fn token_literal(&self) -> &str {
        self.statements
            .first()
            .map(|stmt| stmt.token_literal())
            .unwrap_or_default()
    }

    fn string(&self) -> String {
        self.statements.iter().map(|stmt| stmt.string()).collect()
    }
}

#[derive(Debug)]
pub struct AssignStatement {
    token: Token,
    name: Identifier,
    value: Option<Box<dyn Expression>>,
}

impl Node for AssignStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {} {} ↙️",
            self.name.string(),
            self.token.literal,
            self.value
                .as_ref()
                .map(|exp| exp.string())
                .unwrap_or_default()
        )
    }
}

impl Statement for AssignStatement {}

#[derive(Debug)]
pub struct ReturnStatement {
    token: Token,
    value: Option<Box<dyn Expression>>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {} ↙️",
            self.token.literal,
            self.value
                .as_ref()
                .map(|exp| exp.string())
                .unwrap_or_default()
        )
    }
}

impl Statement for ReturnStatement {}

#[derive(Debug)]
pub struct ExpressionStatement {
    token: Token,
    expression: Box<dyn Expression>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.expression.string()
    }
}

impl Statement for ExpressionStatement {}

#[derive(Debug)]
pub struct Identifier {
    token: Token,
    value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.clone()
    }
}

impl Expression for Identifier {}

#[derive(Debug)]
pub struct IntegerLiteral {
    token: Token,
    value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

impl Expression for IntegerLiteral {}

#[derive(Debug)]
pub struct FloatLiteral {
    token: Token,
    value: f64,
}

impl Node for FloatLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

impl Expression for FloatLiteral {}

#[derive(Debug)]
pub struct PrefixExpression {
    token: Token,
    operator: String,
    right: Box<dyn Expression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }
}

impl Expression for PrefixExpression {}


type PrefixParser = Rc<dyn Fn(&mut Parser) -> Result<Box<dyn Expression>, String>>;
type InfixParser = Rc<dyn Fn(&mut Parser, Box<dyn Expression>) -> Result<Box<dyn Expression>, String>>;

pub struct Parser {
    tokens: StatefulVector<Token>,
    errors: Vec<String>,
    prefix_exp_parsers: HashMap<TokenType, PrefixParser>,
    infix_exp_parsers: HashMap<TokenType, InfixParser>,
}

impl Parser {
    pub fn new(lexer: &mut Lexer<'_>) -> Parser {
        let prefix_exp_parsers = HashMap::new();
        let infix_exp_parsers = HashMap::new();
        let mut parser = Parser {
            tokens: lexer.tokenize(),
            errors: vec![],
            prefix_exp_parsers,
            infix_exp_parsers,
        };
        parser.register_exp_parsers();
        parser
    }

    fn register_exp_parsers(&mut self) {
        self.prefix_exp_parsers.insert(TokenType::Identifier, Rc::new(|p| p.parse_identifier()));
        self.prefix_exp_parsers.insert(TokenType::Integer, Rc::new(|p| p.parse_integer_literal()));
        self.prefix_exp_parsers.insert(TokenType::Float, Rc::new(|p| p.parse_float_literal()));

        self.prefix_exp_parsers.insert(TokenType::Not, Rc::new(|p| p.parse_prefix_expression()));
        self.prefix_exp_parsers.insert(TokenType::Minus, Rc::new(|p| p.parse_prefix_expression()));
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: vec![] };

        while let Some(token) = self.tokens.to_next() {
            let statement = match token.token_type {
                TokenType::Identifier => self.parse_assign_statement(),
                TokenType::Return => self.parse_return_statement(),
                _ => self.parse_expression_statement(),
            };
            match statement {
                Ok(statement) => program.statements.push(statement),
                Err(error_msg) => self.errors.push(error_msg),
            }
        }

        program
    }

    fn parse_assign_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        if self
            .tokens
            .is_next_match(|tok| tok.token_type != TokenType::Assign)
        {
            return self.parse_expression_statement();
        }

        let identifier = self.tokens.current().unwrap().clone();
        let identifier = Identifier {
            token: identifier.clone(),
            value: identifier.literal,
        };
        let assign_token = self.tokens.to_next().unwrap().clone();
        while let Some(token) = self.tokens.to_next()
            && token.token_type != TokenType::Semicolon
        {
            &token.literal;
        }
        Ok(Box::new(AssignStatement {
            token: assign_token,
            name: identifier,
            value: None,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let return_token = self.tokens.current().unwrap().clone();
        while let Some(token) = self.tokens.to_next()
            && token.token_type != TokenType::Semicolon
        {
            &token.literal;
        }
        Ok(Box::new(ReturnStatement {
            token: return_token,
            value: None,
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let exp_token = self.tokens.current().unwrap().clone();
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Box::new(ExpressionStatement {
            token: exp_token,
            expression: exp,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap();

        self.prefix_exp_parsers
            .get(&token.token_type)
            .ok_or_else(|| format!("Expected a expression, but got a {}", token.literal))?
            .clone()
            (self)
    }

    fn parse_identifier(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        Ok(Box::new(Identifier {
            token: token.clone(),
            value: token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token.literal.parse().map_err(|err: ParseIntError| err.to_string())?;
        Ok(Box::new(IntegerLiteral {
            token: token.clone(),
            value,
        }))
    }

    fn parse_float_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token.literal.parse().map_err(|err: ParseFloatError| err.to_string())?;
        Ok(Box::new(FloatLiteral {
            token: token.clone(),
            value,
        }))
    }

    fn parse_prefix_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let operator = token.literal.clone();
        if self.tokens.to_next().is_some() {
            let right = self.parse_expression(Precedence::Prefix)?;
            Ok(Box::new(PrefixExpression {
                token, operator, right
            }))
        } else {
            Err(format!("Expected a expression after operator {}", operator))
        }
    }
}

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn test() {
        let source = String::from(
            "
        ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️
        ⬅️ 3️⃣ ↙️
        ㊙️🔢 ⬅️ 3️⃣⚪9️⃣ ✖️ 2️⃣ ↙️ 
        ➖6️⃣0️⃣ ↙️
        ⏸️ 🈲
        ",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 5);
        assert_eq!(program.statements[0].token_literal(), "⬅️");
        assert_eq!(program.statements[1].token_literal(), "3");
        assert_eq!(program.statements[2].token_literal(), "⬅️");
        assert_eq!(program.statements[3].token_literal(), "➖");
        assert_eq!(program.statements[4].token_literal(), "⏸️");
        assert_eq!(parser.errors.len(), 1);
        assert!(parser.errors[0].contains("⬅️"));
    }
}
