use std::{
    collections::HashMap,
    fmt::Debug,
    num::{ParseFloatError, ParseIntError},
    rc::Rc,
    sync::OnceLock,
};

use crate::{
    lexer::{Lexer, Token, TokenType},
    util::StatefulVector,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest,
    Or,          // 🔀
    And,         // 🔁
    Equals,      // 🟰/❗🟰
    LessGreater, // ▶️/▶️🟰/◀️/◀️🟰
    Sum,         // ➕/➖
    Product,     // ✖️/➗/〰️
    Prefix,      // ➖x/⏸️x
    Call,        // fn🌜🌛
}

static OPERATOR_PRECEDENCES: OnceLock<HashMap<TokenType, Precedence>> = OnceLock::new();

fn get_operator_precedence(token: &Token) -> &Precedence {
    let map = OPERATOR_PRECEDENCES.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(TokenType::Equal, Precedence::Equals);
        map.insert(TokenType::NotEqual, Precedence::Equals);

        map.insert(TokenType::LessThan, Precedence::LessGreater);
        map.insert(TokenType::LessThanOrEqual, Precedence::LessGreater);
        map.insert(TokenType::GreaterThan, Precedence::LessGreater);
        map.insert(TokenType::GreaterThanOrEqual, Precedence::LessGreater);

        map.insert(TokenType::Plus, Precedence::Sum);
        map.insert(TokenType::Minus, Precedence::Sum);

        map.insert(TokenType::Multiply, Precedence::Product);
        map.insert(TokenType::Divide, Precedence::Product);
        map.insert(TokenType::Modulo, Precedence::Product);

        map
    });
    map.get(&token.token_type).unwrap_or(&Precedence::Lowest)
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
    value: Box<dyn Expression>,
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
            self.value.string(),
        )
    }
}

impl Statement for AssignStatement {}

#[derive(Debug)]
pub struct ReturnStatement {
    token: Token,
    value: Box<dyn Expression>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("{} {} ↙️", self.token.literal, self.value.string(),)
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
        format!("{} ↙️", self.expression.string())
    }
}

impl Statement for ExpressionStatement {}

#[derive(Debug)]
pub struct BlockStatement {
    token: Token,
    statements: Vec<Box<dyn Statement>>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("{} {} 🫷", self.token_literal(), self.statements.iter().map(|stmt| stmt.string()).collect::<String>())
    }
}

impl Statement for BlockStatement {}

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
        self.value
            .to_string()
            .chars()
            .map(|digital| format!("{digital}\u{fe0f}\u{20e3}"))
            .collect()
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
        self.value
            .to_string()
            .chars()
            .map(|char| {
                if char == '.' {
                    "\u{26aa}".to_string()
                } else {
                    format!("{char}\u{fe0f}\u{20e3}")
                }
            })
            .collect()
    }
}

impl Expression for FloatLiteral {}

#[derive(Debug)]
pub struct BooleanLiteral {
    token: Token,
    value: bool,
}

impl Node for BooleanLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        String::from(self.token_literal())
    }
}

impl Expression for BooleanLiteral {}

#[derive(Debug)]
pub struct StringLiteral {
    token: Token,
    value: String,
}

impl Node for StringLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!("🗨️{}💬", self.value)
    }
}

impl Expression for StringLiteral {}

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
        format!("🌜{}{}🌛", self.operator, self.right.string())
    }
}

impl Expression for PrefixExpression {}

#[derive(Debug)]
pub struct InfixExpression {
    token: Token,
    left: Box<dyn Expression>,
    operator: String,
    right: Box<dyn Expression>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "🌜{} {} {}🌛",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }
}

impl Expression for InfixExpression {}

#[derive(Debug)]
pub struct IfExpression {
    token: Token,
    condition: Box<dyn Expression>,
    consequence: Box<BlockStatement>,
    alternative: Option<Box<BlockStatement>>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        let mut string = format!(
            "{} {} {}",
            self.token_literal(),
            self.condition.string(),
            self.consequence.string()
        );
        if let Some(stmt) = &self.alternative {
            string.push_str(" ❗ ");
            string.push_str(&stmt.string());
        }
        string
    }
}

impl Expression for IfExpression {}

#[derive(Debug)]
pub struct FunctionLiteral {
    token: Token,
    name: Option<Box<Identifier>>,
    parameters: Vec<Box<Identifier>>,
    body: Box<BlockStatement>,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {}🌜{}🌛 {}",
            self.token_literal(),
            self.name.as_ref().map_or(String::new(), |ident| ident.string() + " "),
            self.parameters.iter().map(|ident| ident.string()).collect::<Vec<String>>().join("🦶 "),
            self.body.string(),
        )
    }
}

impl Expression for FunctionLiteral {}

type PrefixParser = Rc<dyn Fn(&mut Parser) -> Result<Box<dyn Expression>, String>>;
type InfixParser =
    Rc<dyn Fn(&mut Parser, Box<dyn Expression>) -> Result<Box<dyn Expression>, String>>;

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
        self.prefix_exp_parsers
            .insert(TokenType::Identifier, Rc::new(|p| p.parse_identifier().map(|exp| exp as Box<dyn Expression>)));
        self.prefix_exp_parsers
            .insert(TokenType::Integer, Rc::new(|p| p.parse_integer_literal()));
        self.prefix_exp_parsers
            .insert(TokenType::Float, Rc::new(|p| p.parse_float_literal()));
        self.prefix_exp_parsers
            .insert(TokenType::True, Rc::new(|p| p.parse_bool_literal()));
        self.prefix_exp_parsers
            .insert(TokenType::False, Rc::new(|p| p.parse_bool_literal()));
        self.prefix_exp_parsers
            .insert(TokenType::String, Rc::new(|p| p.parse_string_literal()));

        self.prefix_exp_parsers
            .insert(TokenType::Not, Rc::new(|p| p.parse_prefix_expression()));
        self.prefix_exp_parsers
            .insert(TokenType::Minus, Rc::new(|p| p.parse_prefix_expression()));

        self.prefix_exp_parsers
            .insert(TokenType::If, Rc::new(|p| p.parse_if_expression()));
        self.prefix_exp_parsers
            .insert(TokenType::Function, Rc::new(|p| p.parse_function_literal()));

        self.prefix_exp_parsers.insert(
            TokenType::LParenthesis,
            Rc::new(|p| p.parse_group_expression()),
        );

        self.infix_exp_parsers.insert(
            TokenType::Or,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::And,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Equal,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::NotEqual,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::LessThan,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::LessThanOrEqual,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::GreaterThan,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::GreaterThanOrEqual,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Plus,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Minus,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Multiply,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Divide,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Modulo,
            Rc::new(|p, left| p.parse_infix_expression(left)),
        );
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: vec![] };

        while self.tokens.to_next().is_some() {
            let statement = self.parse_statement();
            match statement {
                Ok(statement) => program.statements.push(statement),
                Err(error_msg) => self.errors.push(error_msg),
            }
        }

        program
    }

    fn parse_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        match self.tokens.current().unwrap().token_type {
            TokenType::Identifier => self.parse_assign_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::LBrace => self.parse_block_statement().map(|stmt| stmt as Box<dyn Statement>),
            _ => self.parse_expression_statement(),
        }
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

        self.tokens.to_next();
        let value = self.parse_expression(Precedence::Lowest)?;
        while self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Box::new(AssignStatement {
            token: assign_token,
            name: identifier,
            value,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let return_token = self.tokens.current().unwrap().clone();

        self.tokens.to_next();
        let value = self.parse_expression(Precedence::Lowest)?;
        while self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Box::new(ReturnStatement {
            token: return_token,
            value,
        }))
    }

    fn parse_expression_statement(&mut self) -> Result<Box<dyn Statement>, String> {
        let exp_token = self.tokens.current().unwrap().clone();
        let exp = self.parse_expression(Precedence::Lowest)?;

        while self
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

    fn parse_block_statement(&mut self) -> Result<Box<BlockStatement>, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut statements = vec![];

        self.tokens.to_next();

        while self
            .tokens
            .current()
            .is_some_and(|token| token.token_type != TokenType::RBrace)
        {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.tokens.to_next();
        }

        Ok(Box::new(BlockStatement { token, statements }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap();

        let mut left = self
            .prefix_exp_parsers
            .get(&token.token_type)
            .ok_or_else(|| format!("Expected a expression, but got a {}", token.literal))?
            .clone()(self)?;

        while self.tokens.is_next_match(|next_token| {
            next_token.token_type != TokenType::Semicolon
                && precedence < *get_operator_precedence(next_token)
        }) {
            if let Some(infix) = self
                .tokens
                .to_next()
                .and_then(|token| self.infix_exp_parsers.get(&token.token_type))
            {
                left = infix.clone()(self, left)?;
            } else {
                self.tokens.to_previous();
            }
        }

        Ok(left)
    }

    fn parse_identifier(&self) -> Result<Box<Identifier>, String> {
        let token = self.tokens.current().unwrap().clone();
        Ok(Box::new(Identifier {
            token: token.clone(),
            value: token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token
            .literal
            .parse()
            .map_err(|err: ParseIntError| err.to_string())?;
        Ok(Box::new(IntegerLiteral { token, value }))
    }

    fn parse_float_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token
            .literal
            .parse()
            .map_err(|err: ParseFloatError| err.to_string())?;
        Ok(Box::new(FloatLiteral { token, value }))
    }

    fn parse_bool_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token.token_type == TokenType::True;
        Ok(Box::new(BooleanLiteral { token, value }))
    }

    fn parse_string_literal(&self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token.literal.clone();
        Ok(Box::new(StringLiteral { token, value }))
    }

    fn parse_prefix_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let operator = token.literal.clone();
        if self.tokens.to_next().is_some() {
            let right = self.parse_expression(Precedence::Prefix)?;
            Ok(Box::new(PrefixExpression {
                token,
                operator,
                right,
            }))
        } else {
            Err(format!("Expected a expression after operator {}", operator))
        }
    }

    fn parse_infix_expression(
        &mut self,
        left: Box<dyn Expression>,
    ) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let operator = token.literal.clone();
        let precedence = *get_operator_precedence(self.tokens.current().unwrap());
        self.tokens.to_next();
        let right = self.parse_expression(precedence)?;
        Ok(Box::new(InfixExpression {
            token,
            left,
            operator,
            right,
        }))
    }

    fn parse_group_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        self.tokens.to_next();
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self
            .tokens
            .is_next_match(|token| token.token_type != TokenType::RParenthesis)
        {
            Err(String::from("Expected a right parenthesis"))
        } else {
            self.tokens.to_next();
            Ok(exp)
        }
    }

    fn parse_if_expression(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();

        self.tokens.to_next();
        let condition = self.parse_expression(Precedence::Lowest)?;


        if self
            .tokens
            .is_next_match(|token| token.token_type != TokenType::LBrace)
        {
            return Err(String::from(
                "Expected a block statement after if-condition",
            ));
        }

        self.tokens.to_next();
        let consequence = self.parse_block_statement()?;

        let alternative = if self
            .tokens
            .is_next_match(|token| token.token_type == TokenType::Else)
        {
            self.tokens.to_next();
            if self
                .tokens
                .is_next_match(|token| token.token_type != TokenType::LBrace)
            {
                return Err(String::from("Expected a block statement after else"));
            }
            self.tokens.to_next();
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(Box::new(IfExpression {
            token,
            condition,
            consequence,
            alternative,
        }))
    }

    fn parse_function_literal(&mut self) -> Result<Box<dyn Expression>, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut name = None;
        let mut parameters = vec![];

        if self.tokens.is_next_match(|token| token.token_type == TokenType::Identifier) {
            self.tokens.to_next();
            name = Some(self.parse_identifier()?);
        }

        if self.tokens.is_next_match(|token| token.token_type != TokenType::LParenthesis) {
            return Err("Expected a left parenthesis".to_string());
        }

        self.tokens.to_next();
        
        while let Some(token) = self.tokens.to_next().filter(|token| token.token_type != TokenType::RParenthesis) {
            if token.token_type != TokenType::Identifier {
                return Err(format!("Expected a identifier, but got a {}", token.literal));
            }
            parameters.push(self.parse_identifier()?);

            if self.tokens.is_next_match(|token| token.token_type == TokenType::RParenthesis) {
                continue;
            }

            if let Some(token) = self.tokens.to_next().filter(|token| token.token_type != TokenType::Comma) {
                return Err(format!("Expected a comma, but got a {}", token.literal));
            }
        }

        if self.tokens.is_next_match(|token| token.token_type != TokenType::LBrace) {
            return Err("Expected a left brace".to_string());
        }

        self.tokens.to_next();
        let body = self.parse_block_statement()?;

        Ok(Box::new(FunctionLiteral {
            token,
            name,
            parameters,
            body,
            
        }))
    }
}

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn test() {
        let source = String::from(
                "
        ㊙️🔢 ⬅️ 1️⃣ ➕  3️⃣⚪9️⃣ ✖️ 7️⃣2️⃣ ↙️
        ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️
        📛 🈯 🌜🅰️🦶 🅱️🌛 🫸
          🔙 ❓ 🅰️ ▶️ 🅱️ 🫸🅰️🫷 ❗ 🫸🅱️🫷 ↙️
        🫷
        ⬅️⏸️🌜❌🟰0️⃣◀️1️⃣🌛
            ",
        );
        let target_statements = vec![
            "㊙️🔢 ⬅️ 🌜1️⃣ ➕ 🌜3️⃣⚪9️⃣ ✖️ 7️⃣2️⃣🌛🌛 ↙️",
            "㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️",
            "📛 🈯 🌜🅰️🦶 🅱️🌛 🫸 🔙 ❓ 🌜🅰️ ▶️ 🅱️🌛 🫸 🅰️ ↙️ 🫷 ❗ 🫸 🅱️ ↙️ 🫷 ↙️ 🫷 ↙️",
            "🌜⏸️🌜❌ 🟰 🌜0️⃣ ◀️ 1️⃣🌛🌛🌛 ↙️"
        ];
        let target_errors = vec![
            "Expected a expression, but got a ⬅️",
        ];

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        
        assert_eq!(program.statements.len(), target_statements.len());
        assert_eq!(program.string(), target_statements.join(""));
        assert_eq!(parser.errors.len(), target_errors.len());
        assert_eq!(parser.errors, target_errors);
    }
}
