#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TokenType {
    Illegal,
    Start,

    Assign,

    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,

    And,
    Or,
    Not,

    Comma,
    Semicolon,
    LParenthesis,
    RParenthesis,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    Identifier,

    True,
    False,

    If,
    Else,
    While,
    Function,
    Return,

    Integer,
    Float,
    String,
}

pub const RESERVED_SYMBOLS: [&str; 30] = [
    "⬅️", "➕", "➖", "✖️", "➗", "〰️", "🟰", "▶️", "◀️", "🔁", "🔀", "⏸️", "↙️", "🦶", "🌜", "🌛",
    "👉", "👈", "🫸", "🫷", "🪄", "✔️", "❌", "❓", "❗", "⁉️", "⭕", "📛", "🔙", "#️⃣",
];
pub const DIGITALS: [&str; 10] = ["0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣"];
pub const DOTS: [&str; 9] = ["⚪", "⚫", "🟤", "🟣", "🔵", "🟢", "🟡", "🟠", "🔴"];
pub const SPACES: [&str; 2] = [" ", "\t"];
pub const NEWLINES: [&str; 3] = ["\r", "\n", "\r\n"];
pub const QUOTES: [&str; 2] = ["🗨️", "💬"];

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn from(token_type: TokenType, literal: String) -> Token {
        Token {
            token_type,
            literal,
        }
    }

    pub fn from_str(token_type: TokenType, literal: &str) -> Token {
        Self::from(token_type, String::from(literal))
    }

    pub fn start() -> Token {
        Token::from(TokenType::Start, String::new())
    }
}
