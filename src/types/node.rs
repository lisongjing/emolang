use crate::{
    types::{Token, TokenType},
    util::emoji_convert::{float_to_emoji, integer_to_emoji},
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
    Index,       // list👉 👈
}

impl Precedence {
    pub fn get_operator_precedence(token: &Token) -> Precedence {
        match token.token_type {
            TokenType::Or => Precedence::Or,
            TokenType::And => Precedence::And,
            TokenType::Equal => Precedence::Equals,
            TokenType::NotEqual => Precedence::Equals,
            TokenType::LessThan => Precedence::LessGreater,
            TokenType::LessThanOrEqual => Precedence::LessGreater,
            TokenType::GreaterThan => Precedence::LessGreater,
            TokenType::GreaterThanOrEqual => Precedence::LessGreater,
            TokenType::Plus => Precedence::Sum,
            TokenType::Minus => Precedence::Sum,
            TokenType::Multiply => Precedence::Product,
            TokenType::Divide => Precedence::Product,
            TokenType::Modulo => Precedence::Product,
            TokenType::LParenthesis => Precedence::Call,
            TokenType::LBracket => Precedence::Index,
            _ => Precedence::Lowest,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Node {
    Program {
        statements: Vec<Node>,
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
        statements: Vec<Node>,
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
    ListLiteral {
        token: Token,
        elements: Vec<Node>,
    },
    MapLiteral {
        token: Token,
        entries: Vec<(Node, Node)>,
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
    IndexExpression {
        token: Token,
        left: Box<Node>,
        index: Box<Node>,
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
        parameters: Vec<Node>,
        body: Box<Node>,
    },
    CallExpression {
        token: Token,
        function: Box<Node>,
        arguments: Vec<Node>,
    },
    FieldExpression {
        token: Token,
        instance: Box<Node>,
        field: Box<Node>,
    },
    MethodCallExpression {
        token: Token,
        instance: Box<Node>,
        method: Box<Node>,
    },
}

impl Node {
    pub fn token_literal(&self) -> &str {
        match self {
            Node::Program { statements } => statements
                .first()
                .map(|stmt| stmt.token_literal())
                .unwrap_or_default(),
            Node::AssignStatement { token, name: _, value: _ } => &token.literal,
            Node::ReturnStatement { token, value: _ } => &token.literal,
            Node::ExpressionStatement { token, expression: _ } => &token.literal,
            Node::BlockStatement { token, statements: _ } => &token.literal,
            Node::Identifier { token, value: _ } => &token.literal,
            Node::IntegerLiteral { token, value: _ } => &token.literal,
            Node::FloatLiteral { token, value: _ } => &token.literal,
            Node::BooleanLiteral { token, value: _ } => &token.literal,
            Node::StringLiteral { token, value: _ } => &token.literal,
            Node::ListLiteral { token, elements: _ } => &token.literal,
            Node::MapLiteral { token, entries: _ } => &token.literal,
            Node::PrefixExpression {
                token,
                operator: _,
                right: _,
            } => &token.literal,
            Node::InfixExpression {
                token,
                left: _,
                operator: _,
                right: _,
            } => &token.literal,
            Node::IndexExpression {
                token,
                left: _,
                index: _
            } => &token.literal,
            Node::IfExpression {
                token,
                condition: _,
                consequence: _,
                alternative: _,
            } => &token.literal,
            Node::WhileExpression {
                token,
                condition: _,
                body: _,
            } => &token.literal,
            Node::FunctionLiteral {
                token,
                name: _,
                parameters: _,
                body: _,
            } => &token.literal,
            Node::CallExpression {
                token,
                function: _,
                arguments: _,
            } => &token.literal,
            Node::FieldExpression {
                token,
                instance: _,
                field: _
            } => &token.literal,
            Node::MethodCallExpression {
                token,
                instance: _,
                method: _
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
            Node::ExpressionStatement { token: _, expression } => {
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
            Node::Identifier { token: _, value } => value.clone(),
            Node::IntegerLiteral { token: _, value } => integer_to_emoji(value),
            Node::FloatLiteral { token: _, value } => float_to_emoji(value),
            Node::BooleanLiteral { token, value: _ } => token.literal.clone(),
            Node::StringLiteral { token: _, value } => format!("🗨️{}💬", value),
            Node::ListLiteral { token: _, elements } => format!(
                "👉{}👈",
                elements
                    .iter()
                    .map(|exp| exp.string())
                    .collect::<Vec<String>>()
                    .join("🦶 ")
            ),
            Node::MapLiteral { token: _, entries } => format!(
                "🫸{}🫷",
                entries
                    .iter()
                    .map(|(key, value)| format!("{} ➡️ {}", key.string(), value.string()))
                    .collect::<Vec<String>>()
                    .join("🦶 ")
            ),
            Node::PrefixExpression {
                token: _,
                operator,
                right,
            } => format!("🌜{}{}🌛", operator, right.string()),
            Node::InfixExpression {
                token: _,
                left,
                operator,
                right,
            } => format!("🌜{} {} {}🌛", left.string(), operator, right.string()),
            Node::IndexExpression {
                token: _,
                left,
                index
            } => format!("{}👉{}👈", left.string(), index.string()),
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
                token: _,
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
            Node::FieldExpression {
                token: _,
                instance,
                field,
            } => format!(
                "{}➡️{}",
                instance.string(),
                field.string(),
            ),
            Node::MethodCallExpression {
                token: _,
                instance,
                method,
            } => format!(
                "{}➡️{}",
                instance.string(),
                method.string(),
            ),
        }
    }
}
