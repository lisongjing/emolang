use crate::lexer::{Lexer, Token, TokenType};

pub trait Expression {
    fn token_literal(&self) -> &str;
}

pub struct Program {
    expressions: Vec<Box<dyn Expression>>,
}

impl Expression for Program {
    fn token_literal(&self) -> &str {
        self.expressions
            .first()
            .map(|exp| exp.token_literal())
            .unwrap_or_default()
    }
}

pub struct Identifier {
    token: Token,
    value: String,
}

impl Expression for Identifier {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

pub struct AssignExpression {
    token: Token,
    name: Identifier,
    value: Option<Box<dyn Expression>>,
}

impl Expression for AssignExpression {
    fn token_literal(&self) -> &str {
        &self.token.literal
    }
}

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Parser {
        Parser { lexer }
    }

    pub fn parse_program(&self) -> Program {
        let mut program = Program { expressions: vec![] };
        let tokens = self.lexer.tokenize();
        let mut pos = 0usize;

        while pos < tokens.len() {
            if let Ok(expression) = match tokens[pos].token_type {
                TokenType::Assign => parse_assign_expression(&tokens, &mut pos),
                _ => Err(String::from("Unkown token"))
            } {
                program.expressions.push(Box::new(expression));
            }
            pos += 1;
        }

        program
    }
}

fn parse_assign_expression(tokens: &[Token], pos: &mut usize) -> Result<AssignExpression, String> {
    let assign_token = tokens[*pos].clone();
    let identifier = if *pos > 0 && tokens[*pos - 1].token_type == TokenType::Identifier {
        Ok(Identifier { token: tokens[*pos - 1].clone(), value: tokens[*pos - 1].clone().literal })
    } else {
        Err(String::from("Missing Identifier before ⬅️"))
    }?;
    *pos += 1;
    while tokens[*pos].token_type != TokenType::Semicolon {
        *pos += 1;
    }
    Ok(AssignExpression{ token: assign_token, name: identifier, value: None })
}

#[cfg(test)]
mod parser_test {
    use super::*;

    #[test]
    fn test() {
        let source = String::from(
            "
        ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎💬 ↙️
        ㊙️🔢 ⬅️ 3️⃣⚪9️⃣ ✖️ 2️⃣ ↙️ 
        ",
        );

        let lexer = Lexer::new(source);
        let parser = Parser::new(lexer);
        let program = parser.parse_program();

        assert_eq!(program.expressions.len(), 2);
        assert_eq!(program.expressions[0].token_literal(), "⬅️");
        assert_eq!(program.expressions[1].token_literal(), "⬅️");
    }
}
