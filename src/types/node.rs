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

#[derive(Debug, Clone)]
pub enum Node {
    Program {
        statements: Vec<Box<Node>>,
    },
    // Statement
    AssignStatement {
        token: Token,
        name: Box<Node>,
        value: Box<Node>,
    },
    ReturnStatement {
        token: Token,
        value: Box<Node>,
    },
    ExpressionStatement {
        token: Token,
        expression: Box<Node>,
    },
    BlockStatement {
        token: Token,
        statements: Vec<Box<Node>>,
    },
    // Expression
    Identifier {
        token: Token,
        value: String,
    },
    IntegerLiteral {
        token: Token,
        value: i64,
    },
    FloatLiteral {
        token: Token,
        value: f64,
    },
    BooleanLiteral {
        token: Token,
        value: bool,
    },
    StringLiteral {
        token: Token,
        value: String,
    },
    PrefixExpression {
        token: Token,
        operator: String,
        right: Box<Node>,
    },
    InfixExpression {
        token: Token,
        left: Box<Node>,
        operator: String,
        right: Box<Node>,
    },
    IfExpression {
        token: Token,
        condition: Box<Node>,
        consequence: Box<Node>,
        alternative: Option<Box<Node>>,
    },
    WhileExpression {
        token: Token,
        condition: Box<Node>,
        body: Box<Node>,
    },
    FunctionLiteral {
        token: Token,
        name: Option<Box<Node>>,
        parameters: Vec<Box<Node>>,
        body: Box<Node>,
    },
    CallExpression {
        token: Token,
        function: Box<Node>,
        arguments: Vec<Box<Node>>,
    },
}

impl Node {
    pub fn token_literal(&self) -> &str {
        match self {
            Node::Program { statements } => statements
                .first()
                .map(|stmt| stmt.token_literal())
                .unwrap_or_default(),
            Node::AssignStatement { token, name, value } => &token.literal,
            Node::ReturnStatement { token, value } => &token.literal,
            Node::ExpressionStatement { token, expression } => &token.literal,
            Node::BlockStatement { token, statements } => &token.literal,
            Node::Identifier { token, value } => &token.literal,
            Node::IntegerLiteral { token, value } => &token.literal,
            Node::FloatLiteral { token, value } => &token.literal,
            Node::BooleanLiteral { token, value } => &token.literal,
            Node::StringLiteral { token, value } => &token.literal,
            Node::PrefixExpression {
                token,
                operator,
                right,
            } => &token.literal,
            Node::InfixExpression {
                token,
                left,
                operator,
                right,
            } => &token.literal,
            Node::IfExpression {
                token,
                condition,
                consequence,
                alternative,
            } => &token.literal,
            Node::WhileExpression {
                token,
                condition,
                body,
            } => &token.literal,
            Node::FunctionLiteral {
                token,
                name,
                parameters,
                body,
            } => &token.literal,
            Node::CallExpression {
                token,
                function,
                arguments,
            } => &token.literal,
        }
    }

    pub fn string(&self) -> String {
        match self {
            Node::Program { statements } => statements.iter().map(|stmt| stmt.string()).collect(),
            Node::AssignStatement { token, name, value } => {
                format!("{} {} {} ↙️", name.string(), token.literal, value.string())
            }
            Node::ReturnStatement { token, value } => {
                format!("{} {} ↙️", token.literal, value.string())
            }
            Node::ExpressionStatement { token, expression } => {
                format!("{} ↙️", expression.string())
            }
            Node::BlockStatement { token, statements } => format!(
                "{} {} 🫷",
                token.literal,
                statements
                    .iter()
                    .map(|stmt| stmt.string())
                    .collect::<String>()
            ),
            Node::Identifier { token, value } => value.clone(),
            Node::IntegerLiteral { token, value } => value
                .to_string()
                .chars()
                .map(|digital| format!("{digital}\u{fe0f}\u{20e3}"))
                .collect(),
            Node::FloatLiteral { token, value } => value
                .to_string()
                .chars()
                .map(|char| {
                    if char == '.' {
                        "\u{26aa}".to_string()
                    } else {
                        format!("{char}\u{fe0f}\u{20e3}")
                    }
                })
                .collect(),
            Node::BooleanLiteral { token, value } => token.literal.clone(),
            Node::StringLiteral { token, value } => format!("🗨️{}💬", value),
            Node::PrefixExpression {
                token,
                operator,
                right,
            } => format!("🌜{}{}🌛", operator, right.string()),
            Node::InfixExpression {
                token,
                left,
                operator,
                right,
            } => format!("🌜{} {} {}🌛", left.string(), operator, right.string()),
            Node::IfExpression {
                token,
                condition,
                consequence,
                alternative,
            } => format!(
                "{} {} {}{}",
                token.literal,
                condition.string(),
                consequence.string(),
                if let Some(stmt) = alternative {
                    [" ❗", &stmt.string()].join(" ")
                } else {
                    String::new()
                }
            ),
            Node::WhileExpression {
                token,
                condition,
                body,
            } => format!("{} {} {}", token.literal, condition.string(), body.string(),),
            Node::FunctionLiteral {
                token,
                name,
                parameters,
                body,
            } => format!(
                "{} {}🌜{}🌛 {}",
                token.literal,
                name.as_ref()
                    .map_or(String::new(), |ident| ident.string() + " "),
                parameters
                    .iter()
                    .map(|ident| ident.string())
                    .collect::<Vec<String>>()
                    .join("🦶 "),
                body.string(),
            ),
            Node::CallExpression {
                token,
                function,
                arguments,
            } => format!(
                "{}🌜{}🌛",
                function.string(),
                arguments
                    .iter()
                    .map(|exp| exp.string())
                    .collect::<Vec<String>>()
                    .join("🦶 "),
            ),
        }
    }
}
