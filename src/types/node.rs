use std::{collections::HashMap, fmt::Debug, sync::OnceLock};

use crate::types::{Token, TokenType};

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

pub fn get_operator_precedence(token: &Token) -> &Precedence {
    let map = OPERATOR_PRECEDENCES.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(TokenType::Or, Precedence::Or);
        map.insert(TokenType::And, Precedence::And);

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

        map.insert(TokenType::LParenthesis, Precedence::Call);

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
    pub statements: Vec<Box<dyn Statement>>,
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
    pub token: Token,
    pub name: Identifier,
    pub value: Box<dyn Expression>,
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
    pub token: Token,
    pub value: Box<dyn Expression>,
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
    pub token: Token,
    pub expression: Box<dyn Expression>,
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
    pub token: Token,
    pub statements: Vec<Box<dyn Statement>>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {} 🫷",
            self.token_literal(),
            self.statements
                .iter()
                .map(|stmt| stmt.string())
                .collect::<String>()
        )
    }
}

impl Statement for BlockStatement {}

#[derive(Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
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
    pub token: Token,
    pub value: i64,
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
    pub token: Token,
    pub value: f64,
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
    pub token: Token,
    pub value: bool,
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
    pub token: Token,
    pub value: String,
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
    pub token: Token,
    pub operator: String,
    pub right: Box<dyn Expression>,
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
    pub token: Token,
    pub left: Box<dyn Expression>,
    pub operator: String,
    pub right: Box<dyn Expression>,
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
    pub token: Token,
    pub condition: Box<dyn Expression>,
    pub consequence: Box<BlockStatement>,
    pub alternative: Option<Box<BlockStatement>>,
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
pub struct WhileExpression {
    pub token: Token,
    pub condition: Box<dyn Expression>,
    pub body: Box<BlockStatement>,
}

impl Node for WhileExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {} {}",
            self.token_literal(),
            self.condition.string(),
            self.body.string(),
        )
    }
}

impl Expression for WhileExpression {}

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub name: Option<Box<Identifier>>,
    pub parameters: Vec<Box<Identifier>>,
    pub body: Box<BlockStatement>,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{} {}🌜{}🌛 {}",
            self.token_literal(),
            self.name
                .as_ref()
                .map_or(String::new(), |ident| ident.string() + " "),
            self.parameters
                .iter()
                .map(|ident| ident.string())
                .collect::<Vec<String>>()
                .join("🦶 "),
            self.body.string(),
        )
    }
}

impl Expression for FunctionLiteral {}

#[derive(Debug)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<dyn Expression>,
    pub arguments: Vec<Box<dyn Expression>>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }

    fn string(&self) -> String {
        format!(
            "{}🌜{}🌛",
            self.function.string(),
            self.arguments
                .iter()
                .map(|exp| exp.string())
                .collect::<Vec<String>>()
                .join("🦶 "),
        )
    }
}

impl Expression for CallExpression {}
