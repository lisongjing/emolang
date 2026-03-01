use unicode_segmentation::UnicodeSegmentation;

use crate::types::token::*;
use crate::util::StatefulVector;

pub struct Lexer<'a> {
    chars: StatefulVector<&'a str>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            chars: StatefulVector::from_vec(input.graphemes(true).collect::<Vec<&str>>()),
        }
    }

    pub fn tokenize(&mut self) -> StatefulVector<Token> {
        let mut tokens = StatefulVector::<Token>::new();
        let start_token = Token::start();
        tokens.push(start_token);
        self.chars.insert(0, " ");

        while let Some(char) = self.chars.to_next() {
            let token = match *char {
                "#️⃣" => {
                    self.skip_comment();
                    continue;
                }
                "⬅️" => Token::from_str(TokenType::Assign, char),
                "➕" => Token::from_str(TokenType::Plus, char),
                "➖" => Token::from_str(TokenType::Minus, char),
                "✖️" => Token::from_str(TokenType::Multiply, char),
                "➗" => Token::from_str(TokenType::Divide, char),
                "〰️" => Token::from_str(TokenType::Modulo, char),
                "🟰" => Token::from_str(TokenType::Equal, char),
                "▶️" => self.handle_two_chars_token(
                    TokenType::GreaterThan,
                    "🟰",
                    TokenType::GreaterThanOrEqual,
                ),
                "◀️" => self.handle_two_chars_token(
                    TokenType::LessThan,
                    "🟰",
                    TokenType::LessThanOrEqual,
                ),
                "🔁" => Token::from_str(TokenType::And, char),
                "🔀" => Token::from_str(TokenType::Or, char),
                "⏸️" => Token::from_str(TokenType::Not, char),
                "↙️" => Token::from_str(TokenType::Semicolon, char),
                "✔️" => Token::from_str(TokenType::True, char),
                "❌" => Token::from_str(TokenType::False, char),
                "❓" => Token::from_str(TokenType::If, char),
                "❗" => self.handle_two_chars_token(TokenType::Else, "🟰", TokenType::NotEqual),
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
                "🗨️" => self.handle_string(),
                _ if DIGITALS.contains(char) => self.handle_number(),
                _ if SPACES.contains(char) => continue,
                _ if is_identifier_char(char) => self.handle_identifier(),
                _ => Token::from_str(TokenType::Illegal, char),
            };
            tokens.push(token);
        }
        tokens
    }

    fn handle_two_chars_token(
        &mut self,
        single_char_token_type: TokenType,
        expected_next_char: &str,
        two_chars_token_type: TokenType,
    ) -> Token {
        let mut current_char = String::from(*self.chars.current().unwrap());
        let mut token_type = single_char_token_type;

        if self.chars.is_next_eq(&expected_next_char) {
            token_type = two_chars_token_type;
            current_char.push_str(self.chars.to_next().unwrap());
        }

        Token::from(token_type, current_char)
    }

    fn handle_string(&mut self) -> Token {
        let mut literal = String::from(*self.chars.current().unwrap());
        while self.chars.to_next().is_some_and(|&char| char != "💬") {
            literal.push_str(self.chars.current().unwrap());
        }
        if self.chars.current().is_some() {
            literal.push_str(self.chars.current().unwrap());
        }
        Token::from(TokenType::String, literal)
    }

    fn handle_number(&mut self) -> Token {
        let current_char = self.chars.current().unwrap().chars().next().unwrap();
        let mut literal = String::from(current_char);
        let mut token_type = TokenType::Integer;
        loop {
            let is_digital = self.chars.is_next_match(|char| DIGITALS.contains(char));
            let is_dot = self.chars.is_next_match(|char| DOTS.contains(char));

            if is_digital {
                let next_char = self.chars.to_next().unwrap();
                literal.push(next_char.chars().next().unwrap());
            } else if is_dot {
                self.chars.to_next().unwrap();
                token_type = TokenType::Float;
                literal.push('.');
            } else {
                break;
            }
        }
        Token::from(token_type, literal)
    }

    fn handle_identifier(&mut self) -> Token {
        let mut literal = String::from(*self.chars.current().unwrap());
        while self.chars.is_next_match(|char| is_identifier_char(char)) {
            literal.push_str(self.chars.to_next().unwrap());
        }
        Token::from(TokenType::Identifier, literal)
    }

    fn skip_comment(&mut self) {
        while self
            .chars
            .to_next()
            .is_some_and(|&char| !NEWLINE.contains(&char))
        {}
    }
}

fn is_identifier_char(char: &str) -> bool {
    !RESERVED_SYMBOLS.contains(&char)
        && !DIGITALS.contains(&char)
        && !DOTS.contains(&char)
        && !SPACES.contains(&char)
}

#[cfg(test)]
mod lexer_test {
    use super::*;

    #[test]
    fn test() {
        let source = String::from(
            "
        ㊙️🔢 ⬅️ 1️⃣ ➕  3️⃣⚪9️⃣ ✖️ 7️⃣2️⃣ ↙️ #️⃣test assign statement
        ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️
        📛 🈯 🌜🅰️🦶 🅱️🌛 🫸
          ⭕ 🅰️ ▶️🟰 0️⃣ 🔁 🅱️ ◀️🟰 5️⃣ 🫸
            🅰️ ⬅️ 🅰️ ➕ 🅱️ ↙️
            🅱️ ⬅️ 🅱️ ➖ 🅰️ ↙️
          🫷
          🔙 ❓ 🅰️ ▶️ 🅱️ 🫸🅰️🫷 ❗ 🫸🅱️🫷 ↙️
        🫷
        ⏸️🌜❌🟰0️⃣◀️1️⃣🌛
        ",
        );
        let target = vec![
            Token::start(),
            Token::from_str(TokenType::Identifier, "㊙️🔢"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::Integer, "1"),
            Token::from_str(TokenType::Plus, "➕"),
            Token::from_str(TokenType::Float, "3.9"),
            Token::from_str(TokenType::Multiply, "✖️"),
            Token::from_str(TokenType::Integer, "72"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::Identifier, "㊙️🔡"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::String, "🗨️🈶🅰️🈚🅱️🈲🆎💬"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::Function, "📛"),
            Token::from_str(TokenType::Identifier, "🈯"),
            Token::from_str(TokenType::LParenthesis, "🌜"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::Comma, "🦶"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::RParenthesis, "🌛"),
            Token::from_str(TokenType::LBrace, "🫸"),
            Token::from_str(TokenType::While, "⭕"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::GreaterThanOrEqual, "▶️🟰"),
            Token::from_str(TokenType::Integer, "0"),
            Token::from_str(TokenType::And, "🔁"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::LessThanOrEqual, "◀️🟰"),
            Token::from_str(TokenType::Integer, "5"),
            Token::from_str(TokenType::LBrace, "🫸"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::Plus, "➕"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::Assign, "⬅️"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::Minus, "➖"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::RBrace, "🫷"),
            Token::from_str(TokenType::Return, "🔙"),
            Token::from_str(TokenType::If, "❓"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::GreaterThan, "▶️"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::LBrace, "🫸"),
            Token::from_str(TokenType::Identifier, "🅰️"),
            Token::from_str(TokenType::RBrace, "🫷"),
            Token::from_str(TokenType::Else, "❗"),
            Token::from_str(TokenType::LBrace, "🫸"),
            Token::from_str(TokenType::Identifier, "🅱️"),
            Token::from_str(TokenType::RBrace, "🫷"),
            Token::from_str(TokenType::Semicolon, "↙️"),
            Token::from_str(TokenType::RBrace, "🫷"),
            Token::from_str(TokenType::Not, "⏸️"),
            Token::from_str(TokenType::LParenthesis, "🌜"),
            Token::from_str(TokenType::False, "❌"),
            Token::from_str(TokenType::Equal, "🟰"),
            Token::from_str(TokenType::Integer, "0"),
            Token::from_str(TokenType::LessThan, "◀️"),
            Token::from_str(TokenType::Integer, "1"),
            Token::from_str(TokenType::RParenthesis, "🌛"),
        ];
        let mut lexer = Lexer::new(&source);
        assert_eq!(lexer.tokenize().to_vec(), target);
    }
}
