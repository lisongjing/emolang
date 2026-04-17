use crate::{
    types::{Token, TokenType},
    util::emoji_convert::{boolean_to_emoji, float_to_emoji, integer_to_emoji},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest,
    Assign,      // ⬅️
    Or,          // 🔀
    And,         // 🔁
    Equals,      // 🟰/❗🟰
    LessGreater, // ▶️/▶️🟰/◀️/◀️🟰
    Sum,         // ➕/➖
    Product,     // ✖️/➗/〰️
    Prefix,      // ➖x/⏸️x
    Call,        // fn🌜🌛
    Index,       // list/map👉 👈 instance❇️
}

impl Precedence {
    pub fn get_operator_precedence(token: &Token) -> Precedence {
        match token.token_type {
            TokenType::Assign => Precedence::Assign,
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
            TokenType::Member => Precedence::Index,
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
    ReturnStatement {
        value: Box<Node>,
    },
    ExpressionStatement {
        expression: Box<Node>,
    },
    BlockStatement {
        statements: Vec<Node>,
    },
    // Expression
    Identifier {
        value: String,
    },
    IntegerLiteral {
        value: i64,
    },
    FloatLiteral {
        value: f64,
    },
    BooleanLiteral {
        value: bool,
    },
    StringLiteral {
        value: String,
    },
    ListLiteral {
        elements: Vec<Node>,
    },
    MapLiteral {
        entries: Vec<(Node, Node)>,
    },
    PrefixExpression {
        operator: String,
        right: Box<Node>,
    },
    InfixExpression {
        left: Box<Node>,
        operator: String,
        right: Box<Node>,
    },
    AssignExpression {
        identifier: Box<Node>,
        value: Box<Node>,
    },
    IndexExpression {
        left: Box<Node>,
        index: Box<Node>,
    },
    IfExpression {
        condition: Box<Node>,
        consequence: Box<Node>,
        alternative: Option<Box<Node>>,
    },
    WhileExpression {
        condition: Box<Node>,
        body: Box<Node>,
    },
    BreakExpression {
        value: Option<Box<Node>>,
    },
    FunctionLiteral {
        name: Option<Box<Node>>,
        parameters: Vec<Node>,
        body: Box<Node>,
    },
    CallExpression {
        function: Box<Node>,
        arguments: Vec<Node>,
    },
    MemberExpression {
        instance: Box<Node>,
        member: Box<Node>,
    },
}

impl Node {
    pub fn string(&self) -> String {
        match self {
            Node::Program { statements } => statements.iter().map(|stmt| stmt.string()).collect(),
            Node::ReturnStatement { value } => {
                format!("🔙 {} ↙️", value.string())
            }
            Node::ExpressionStatement { expression } => {
                format!("{} ↙️", expression.string())
            }
            Node::BlockStatement { statements } => format!(
                "🫸 {} 🫷",
                statements
                    .iter()
                    .map(|stmt| stmt.string())
                    .collect::<String>()
            ),
            Node::Identifier { value } => value.clone(),
            Node::IntegerLiteral { value } => integer_to_emoji(value),
            Node::FloatLiteral { value } => float_to_emoji(value),
            Node::BooleanLiteral { value } => boolean_to_emoji(value),
            Node::StringLiteral { value } => format!("🗨️{}💬", value),
            Node::ListLiteral { elements } => format!(
                "👉{}👈",
                elements
                    .iter()
                    .map(|exp| exp.string())
                    .collect::<Vec<String>>()
                    .join("🦶 ")
            ),
            Node::MapLiteral { entries } => format!(
                "🫸{}🫷",
                entries
                    .iter()
                    .map(|(key, value)| format!("{} ➡️ {}", key.string(), value.string()))
                    .collect::<Vec<String>>()
                    .join("🦶 ")
            ),
            Node::PrefixExpression {
                operator,
                right,
            } => format!("🌜{}{}🌛", operator, right.string()),
            Node::InfixExpression {
                left,
                operator,
                right,
            } => format!("🌜{} {} {}🌛", left.string(), operator, right.string()),
            Node::AssignExpression {
                identifier,
                value
            } => format!("{} ⬅️ {}", identifier.string(), value.string()),
            Node::IndexExpression {
                left,
                index
            } => format!("{}👉{}👈", left.string(), index.string()),
            Node::IfExpression {
                condition,
                consequence,
                alternative,
            } => format!(
                "❓ {} {}{}",
                condition.string(),
                consequence.string(),
                if let Some(stmt) = alternative {
                    format!(" ❗ {}", &stmt.string())
                } else {
                    String::new()
                }
            ),
            Node::WhileExpression {
                condition,
                body,
            } => format!("⭕ {} {}", condition.string(), body.string(),),
            Node::BreakExpression {
                value
            } => format!("🔚{}", value.as_ref().map_or(String::new(), |v| format!(" {}", v.string()))),
            Node::FunctionLiteral {
                name,
                parameters,
                body,
            } => format!(
                "📛 {}🌜{}🌛 {}",
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
            Node::MemberExpression {
                instance,
                member,
            } => format!(
                "{}❇️{}",
                instance.string(),
                member.string(),
            ),
        }
    }
}
