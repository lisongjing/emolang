use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq)]
enum TokenType {
    Illegal,
    End,

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

    Number,
    String,
}

const RESERVED_SYMBOLS: [&str; 29] = [
    "⬅️", "➕", "➖", "✖️", "➗", "〰️", "🟰", "▶️", "◀️", "🔁", "🔀", "⏸️", "↙️", "🦶", "🌜", "🌛",
    "👉", "👈", "🫸", "🫷", "🗨️", "💬", "✔️", "❌", "❓", "❗", "⭕", "📛", "🔙",
];
const DIGITALS: [&str; 11] = [
    "0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟",
];
const DOTS: [&str; 9] = ["⚪", "⚫", "🟤", "🟣", "🔵", "🟢", "🟡", "🟠", "🔴"];
const SPACES: [&str; 4] = [" ", "\t", "\r", "\n"];

pub struct Lexer {
    input: String,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        Lexer { input }
    }

    pub fn tokenize(&self) -> Vec<Token> {
        let chars = self.input.graphemes(true).collect::<Vec<&str>>();
        let mut pos = 0usize;
        let mut tokens = vec![];

        while pos < chars.len() {
            let char = chars[pos];
            let token = match char {
                "⬅️" => Token::from_str(TokenType::Assign, char),
                "➕" => Token::from_str(TokenType::Plus, char),
                "➖" => Token::from_str(TokenType::Minus, char),
                "✖️" => Token::from_str(TokenType::Multiply, char),
                "➗" => Token::from_str(TokenType::Divide, char),
                "〰️" => Token::from_str(TokenType::Modulo, char),
                "🟰" => Token::from_str(TokenType::Equal, char),
                "▶️" => handle_two_chars_token(&chars, &mut pos),
                "◀️" => handle_two_chars_token(&chars, &mut pos),
                "🔁" => Token::from_str(TokenType::And, char),
                "🔀" => Token::from_str(TokenType::Or, char),
                "⏸️" => Token::from_str(TokenType::Not, char),
                "↙️" => Token::from_str(TokenType::Semicolon, char),
                "✔️" => Token::from_str(TokenType::True, char),
                "❌" => Token::from_str(TokenType::False, char),
                "❓" => Token::from_str(TokenType::If, char),
                "❗" => handle_two_chars_token(&chars, &mut pos),
                "⭕" => Token::from_str(TokenType::While, char),
                "📛" => Token::from_str(TokenType::Function, char),
                "🔙" => Token::from_str(TokenType::Return, char),
                "🦶" => Token::from_str(TokenType::Comma, char),
                "🌜" => Token::from_str(TokenType::LParenthesis, char),
                "🌛" => Token::from_str(TokenType::RParenthesis, char),
                "👉" => Token::from_str(TokenType::LBracket, char),
                "👈" => Token::from_str(TokenType::RBracket, char),
                "🫸" => Token::from_str(TokenType::LBrace, char),
                "🫷" => Token::from_str(TokenType::RBrace, char),
                "🗨️" => handle_string(&chars, &mut pos),
                _ if DIGITALS.contains(&char) => handle_number(&chars, &mut pos),
                _ if SPACES.contains(&char) => {
                    pos += 1;
                    continue;
                }
                _ if is_identifier_char(char) => handle_identifier(&chars, &mut pos),
                _ => Token::from_str(TokenType::Illegal, char),
            };
            pos += 1;
            tokens.push(token);
        }
        tokens.push(Token::from(TokenType::End, String::new()));
        tokens
    }
}

fn handle_two_chars_token(chars: &[&str], pos: &mut usize) -> Token {
    let first_char = chars[*pos];
    let mut literal = String::from(first_char);
    literal.push_str(chars[*pos + 1]);
    match &*literal {
        "❗🟰" => {
            *pos += 1;
            Token::from(TokenType::NotEqual, literal)
        }
        "▶️🟰" => {
            *pos += 1;
            Token::from(TokenType::GreaterThanOrEqual, literal)
        }
        "◀️🟰" => {
            *pos += 1;
            Token::from(TokenType::LessThanOrEqual, literal)
        }
        _ if first_char == "❗" => Token::from(TokenType::Else, literal),
        _ if first_char == "▶️" => Token::from(TokenType::GreaterThan, literal),
        _ if first_char == "◀️" => Token::from(TokenType::LessThan, literal),
        _ => Token::from_str(TokenType::Illegal, first_char),
    }
}

fn handle_string(chars: &[&str], pos: &mut usize) -> Token {
    let mut literal = String::new();
    *pos += 1;
    while chars[*pos] != "💬" {
        literal.push_str(chars[*pos]);
        *pos += 1;
    }
    Token::from(TokenType::String, literal)
}

fn handle_number(chars: &[&str], pos: &mut usize) -> Token {
    let mut literal = String::from(chars[*pos]);
    while DIGITALS.contains(&chars[*pos + 1]) || DOTS.contains(&chars[*pos + 1]) {
        *pos += 1;
        literal.push_str(chars[*pos]);
    }
    Token::from(TokenType::Number, literal)
}

fn handle_identifier(chars: &[&str], pos: &mut usize) -> Token {
    let mut literal = String::from(chars[*pos]);
    while is_identifier_char(chars[*pos + 1]) {
        *pos += 1;
        literal.push_str(chars[*pos]);
    }
    Token::from(TokenType::Identifier, literal)
}

fn is_identifier_char(char: &str) -> bool {
    !RESERVED_SYMBOLS.contains(&char)
        && !DIGITALS.contains(&char)
        && !DOTS.contains(&char)
        && !SPACES.contains(&char)
}

#[derive(Debug, PartialEq)]
pub struct Token {
    token_type: TokenType,
    literal: String,
}

impl Token {
    fn from(token_type: TokenType, literal: String) -> Token {
        Token {
            token_type,
            literal,
        }
    }

    fn from_str(token_type: TokenType, literal: &str) -> Token {
        Self::from(token_type, String::from(literal))
    }
}

#[cfg(test)]
mod lexer_test {
    use super::*;

    #[test]
    fn test() {
        let source = String::from("㊙️🔢 ⬅️ 3️⃣ ✖️ 2️⃣ ↙️ ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️");
        let target = vec![
            Token::from_str(TokenType::Identifier, "㊙️🔢"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::Number, "3️⃣"),
            Token::from_str(TokenType::Multiply, "✖️"),
            Token::from_str(TokenType::Number, "2️⃣"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::Identifier, "㊙️🔡"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::String, "🈶🅰️🈚🅱️🈲🆎"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from(TokenType::End, String::new()),
        ];
        let lexer = Lexer::new(source);
        assert_eq!(lexer.tokenize(), target);
    }
}
