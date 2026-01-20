use std::collections::HashMap;

use crate::lexer::{Lexer, Token, TokenType};

pub enum Precedence {
    Lowest,
    Equals,      // 🟰/❗🟰
    LessGreater, // ▶️/▶️🟰/◀️/◀️🟰
    Plus,        // ➕/➖
    Product,     // ✖️/➗/〰️
    Prefix,      // ➖x/⏸️x
    Call,        // fn🌜🌛
}

pub trait Node {
    fn token_literal(&self) -> &str;
    fn string(&self) -> String;
}

pub trait Statement: Node {}

pub trait Expression: Node {}

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
        format!("{} {} {} ↙️", self.name.string(), self.token.literal, self.value.as_ref().map(|exp| exp.string()).unwrap_or_default())
    }
}

impl Statement for AssignStatement {}

pub struct ReturnStatement {
    token: Token,
    value: Option<Box<dyn Expression>>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    
    fn string(&self) -> String {
        format!("{} {} ↙️", self.token.literal, self.value.as_ref().map(|exp| exp.string()).unwrap_or_default())
    }
}

impl Statement for ReturnStatement {}

pub struct ExpressionStatement {
    token: Token,
    expression: Option<Box<dyn Expression>>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
    
    fn string(&self) -> String {
        self.expression.as_ref().map(|exp| exp.string()).unwrap_or_default()
    }
}

impl Statement for ExpressionStatement {}

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

type PrefixParser = Box<dyn Fn(&[Token], &mut usize) -> Box<dyn Expression>>;
type InfixParser = Box<dyn Fn(&[Token], &mut usize, Box<dyn Expression>) -> Box<dyn Expression>>;

pub struct Parser {
    lexer: Lexer,
    errors: Vec<String>,
    prefix_exp_parsers: HashMap<TokenType, PrefixParser>,
    infix_exp_parsers: HashMap<TokenType, InfixParser>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        let mut prefix_exp_parsers = HashMap::new();
        let identifier_parser: PrefixParser = Box::new(parse_identifier);
        prefix_exp_parsers.insert(TokenType::Identifier, identifier_parser);

        Parser { lexer, errors: vec![], prefix_exp_parsers, infix_exp_parsers: HashMap::new() }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: vec![],
        };
        let tokens = self.lexer.tokenize();
        let mut pos = 0usize;

        while pos < tokens.len() {
            let next_token = if pos < tokens.len() + 1 {
                &tokens[pos + 1]
            } else {
                &Token::end()
            };

            let statement = match &tokens[pos].token_type {
                TokenType::Identifier if is_token_type(TokenType::Assign, next_token) => {
                    parse_assign_statement(&tokens, &mut pos)
                }
                TokenType::Return => parse_return_statement(&tokens, &mut pos),
                _ => parse_expression_statement(&tokens, &mut pos),
            };
            match statement {
                Ok(statement) => program.statements.push(statement),
                Err(error_msg) => self.errors.push(error_msg),
            }
            pos += 1;
        }

        program
    }
}

fn is_token_type(expected_token_type: TokenType, token: &Token) -> bool {
    expected_token_type == token.token_type
}

fn parse_assign_statement(tokens: &[Token], pos: &mut usize) -> Result<Box<dyn Statement>, String> {
    let identifier = tokens[*pos].clone();
    let identifier = Identifier {
        token: identifier.clone(),
        value: identifier.literal,
    };
    *pos += 1;
    let assign_token = tokens[*pos].clone();
    while tokens[*pos].token_type != TokenType::Semicolon {
        *pos += 1;
    }
    Ok(Box::new(AssignStatement {
        token: assign_token,
        name: identifier,
        value: None,
    }))
}

fn parse_return_statement(tokens: &[Token], pos: &mut usize) -> Result<Box<dyn Statement>, String> {
    let return_token = tokens[*pos].clone();
    *pos += 1;
    while tokens[*pos].token_type != TokenType::Semicolon {
        *pos += 1;
    }
    Ok(Box::new(ReturnStatement {
        token: return_token,
        value: None,
    }))
}

fn parse_expression_statement(tokens: &[Token], pos: &mut usize) -> Result<Box<dyn Statement>, String> {
    let exp_token = tokens[*pos].clone();

    if *pos < tokens.len() - 1 && is_token_type(TokenType::Semicolon, &tokens[*pos + 1]) {
        *pos += 1;
    }

    Ok(Box::new(ExpressionStatement {
        token: exp_token,
        expression: None,
    }))
}

fn parse_identifier(tokens: &[Token], pos: &mut usize) -> Box<dyn Expression> {
    Box::new(Identifier { token: tokens[*pos].clone(), value: tokens[*pos].literal.clone() })
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
        ",
        );

        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 2);
        assert_eq!(program.statements[0].token_literal(), "⬅️");
        assert_eq!(program.statements[1].token_literal(), "⬅️");
        assert_eq!(parser.errors.len(), 3);
    }
}
