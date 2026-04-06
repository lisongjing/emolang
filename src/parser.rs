use std::{
    collections::HashMap,
    num::{ParseFloatError, ParseIntError},
    rc::Rc,
};

use crate::{
    lexer::Lexer,
    types::{QUOTES, Token, TokenType, node::*},
    util::StatefulVector,
};

type PrefixParser = Rc<dyn Fn(&mut Parser) -> Result<Node, String>>;
type InfixParser = Rc<dyn Fn(&mut Parser, Node) -> Result<Node, String>>;

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

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn register_exp_parsers(&mut self) {
        self.prefix_exp_parsers
            .insert(TokenType::Identifier, Rc::new(|p| p.parse_identifier()));
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
            .insert(TokenType::LBracket, Rc::new(|p| p.parse_list_literal()));
        self.prefix_exp_parsers
            .insert(TokenType::LBrace, Rc::new(|p| p.parse_map_literal()));

        self.prefix_exp_parsers
            .insert(TokenType::Not, Rc::new(|p| p.parse_prefix_expression()));
        self.prefix_exp_parsers
            .insert(TokenType::Minus, Rc::new(|p| p.parse_prefix_expression()));

        self.prefix_exp_parsers
            .insert(TokenType::If, Rc::new(|p| p.parse_if_expression()));
        self.prefix_exp_parsers
            .insert(TokenType::While, Rc::new(|p| p.parse_while_expression()));
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

        self.infix_exp_parsers.insert(
            TokenType::LBracket,
            Rc::new(|p, left| p.parse_index_expression(left))
        );
        self.infix_exp_parsers.insert(
            TokenType::LParenthesis,
            Rc::new(|p, left| p.parse_call_expression(left)),
        );
        self.infix_exp_parsers.insert(
            TokenType::Member,
            Rc::new(|p, left| p.parse_member_expression(left)),
        );
    }

    pub fn parse_program(&mut self) -> Node {
        let mut statements = vec![];

        while self.tokens.to_next().is_some() {
            let statement = self.parse_statement();
            match statement {
                Ok(statement) => statements.push(statement),
                Err(error_msg) => self.errors.push(error_msg),
            }
        }

        Node::Program { statements }
    }

    fn parse_statement(&mut self) -> Result<Node, String> {
        match self.tokens.current().unwrap().token_type {
            TokenType::Identifier => self.parse_assign_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Semicolon => {
                self.tokens.to_next();
                self.parse_statement()
            },
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_assign_statement(&mut self) -> Result<Node, String> {
        if self
            .tokens
            .is_next_match(|tok| tok.token_type != TokenType::Assign)
        {
            return self.parse_expression_statement();
        }

        let identifier = self.tokens.current().unwrap().clone();
        let name = Box::new(Node::Identifier {
            token: identifier.clone(),
            value: identifier.literal,
        });
        let assign_token = self.tokens.to_next().unwrap().clone();

        self.tokens.to_next();
        let value = Box::new(self.parse_expression(Precedence::Lowest)?);
        while self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Node::AssignStatement {
            token: assign_token,
            name,
            value,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Node, String> {
        let return_token = self.tokens.current().unwrap().clone();

        self.tokens.to_next();
        let value = Box::new(self.parse_expression(Precedence::Lowest)?);
        while self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Node::ReturnStatement {
            token: return_token,
            value,
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Node, String> {
        let exp_token = self.tokens.current().unwrap().clone();
        let expression = Box::new(self.parse_expression(Precedence::Lowest)?);

        while self
            .tokens
            .is_next_match(|tok| tok.token_type == TokenType::Semicolon)
        {
            self.tokens.to_next();
        }

        Ok(Node::ExpressionStatement {
            token: exp_token,
            expression,
        })
    }

    fn parse_block_statement(&mut self) -> Result<Node, String> {
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

        Ok(Node::BlockStatement { token, statements })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Node, String> {
        let token = self.tokens.current().unwrap();

        let mut left = self
            .prefix_exp_parsers
            .get(&token.token_type)
            .ok_or_else(|| format!("Expected a expression, but got a {}", token.literal))?
            .clone()(self)?;

        while self.tokens.is_next_match(|next_token| {
            next_token.token_type != TokenType::Semicolon
                && precedence < Precedence::get_operator_precedence(next_token)
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

    fn parse_identifier(&self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        Ok(Node::Identifier {
            token: token.clone(),
            value: token.literal.clone(),
        })
    }

    fn parse_integer_literal(&self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token
            .literal
            .parse()
            .map_err(|err: ParseIntError| err.to_string())?;
        Ok(Node::IntegerLiteral { token, value })
    }

    fn parse_float_literal(&self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token
            .literal
            .parse()
            .map_err(|err: ParseFloatError| err.to_string())?;
        Ok(Node::FloatLiteral { token, value })
    }

    fn parse_bool_literal(&self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let value = token.token_type == TokenType::True;
        Ok(Node::BooleanLiteral { token, value })
    }

    fn parse_string_literal(&self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut value = token
            .literal
            .clone()
            .replace("🪄↩️", "\n")
            .replace("🪄➡️", "\t")
            .replace("🪄🗨️", "🗨️")
            .replace("🪄💬", "💬");

        let mut has_prefix = false;
        let mut has_suffix = false;

        for quote in QUOTES {
            if !has_prefix && let Some(val) = value.strip_prefix(quote) {
                value = String::from(val);
                has_prefix = true;
            }
            if !has_suffix && let Some(val) = value.strip_suffix(quote) {
                value = String::from(val);
                has_suffix = true;
            }
        }
        if !has_prefix {
            return Err(String::from("Expected 🗨️ or 💬 at the start of a string literal"));
        }
        if !has_suffix {
            return Err(String::from("Expected 🗨️ or 💬 at the end of a string literal"));
        }

        Ok(Node::StringLiteral { token, value })
    }

    fn parse_list_literal(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut elements = vec![];
        while self
            .tokens
            .to_next()
            .filter(|token| token.token_type != TokenType::RBracket)
            .is_some()
        {
            elements.push(self.parse_expression(Precedence::Lowest)?);

            if self
                .tokens
                .is_next_match(|token| token.token_type == TokenType::RBracket)
            {
                continue;
            }

            if let Some(token) = self
                .tokens
                .to_next()
                .filter(|token| token.token_type != TokenType::Comma)
            {
                return Err(format!("Expected a comma, but got a {}", token.literal));
            }
        }
        Ok(Node::ListLiteral { token, elements })
    }

    fn parse_map_literal(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut entries = vec![];
        while self
            .tokens
            .to_next()
            .filter(|token| token.token_type != TokenType::RBrace)
            .is_some()
        {
            let key = self.parse_expression(Precedence::Lowest)?;

            if self
                .tokens
                .is_next_match(|token| token.token_type != TokenType::Describe)
            {
                return Err(format!("Expected a ➡️, but got a {}", token.literal));
            }
            self.tokens.to_next();
            self.tokens.to_next();

            let value = self.parse_expression(Precedence::Lowest)?;

            entries.push((key, value));

            if self
                .tokens
                .is_next_match(|token| token.token_type == TokenType::RBrace)
            {
                continue;
            }

            if let Some(token) = self
                .tokens
                .to_next()
                .filter(|token| token.token_type != TokenType::Comma)
            {
                return Err(format!("Expected a comma, but got a {}", token.literal));
            }
        }
        Ok(Node::MapLiteral { token, entries })
    }

    fn parse_prefix_expression(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let operator = token.literal.clone();
        if self.tokens.to_next().is_some() {
            let right = Box::new(self.parse_expression(Precedence::Prefix)?);
            Ok(Node::PrefixExpression {
                token,
                operator,
                right,
            })
        } else {
            Err(format!("Expected a expression after operator {}", operator))
        }
    }

    fn parse_infix_expression(&mut self, left: Node) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let operator = token.literal.clone();
        let precedence = Precedence::get_operator_precedence(self.tokens.current().unwrap());
        self.tokens.to_next();
        let right = Box::new(self.parse_expression(precedence)?);
        Ok(Node::InfixExpression {
            token,
            left: Box::new(left),
            operator,
            right,
        })
    }

    fn parse_group_expression(&mut self) -> Result<Node, String> {
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

    fn parse_if_expression(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();

        self.tokens.to_next();
        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);

        if self
            .tokens
            .is_next_match(|token| token.token_type != TokenType::LBrace)
        {
            return Err(String::from(
                "Expected a block statement after if-condition",
            ));
        }

        self.tokens.to_next();
        let consequence = Box::new(self.parse_block_statement()?);

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
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };

        Ok(Node::IfExpression {
            token,
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_while_expression(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();

        self.tokens.to_next();
        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);

        if self
            .tokens
            .is_next_match(|token| token.token_type != TokenType::LBrace)
        {
            return Err(String::from(
                "Expected a block statement after while-condition",
            ));
        }

        self.tokens.to_next();
        let body = Box::new(self.parse_block_statement()?);

        Ok(Node::WhileExpression {
            token,
            condition,
            body,
        })
    }

    fn parse_function_literal(&mut self) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut name = None;
        let mut parameters = vec![];

        if self.tokens.is_next_match(|token| token.token_type == TokenType::Identifier) {
            self.tokens.to_next();
            name = Some(Box::new(self.parse_identifier()?));
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
        let body = Box::new(self.parse_block_statement()?);

        Ok(Node::FunctionLiteral {
            token,
            name,
            parameters,
            body,
        })
    }

    fn parse_index_expression(&mut self, list: Node) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        self.tokens.to_next();
        let index = self.parse_expression(Precedence::Lowest)?;
        if let Some(tok) = self.tokens.to_next() && tok.token_type == TokenType::RBracket {
            Ok(Node::IndexExpression {
                token,
                left: Box::new(list),
                index: Box::new(index),
            })
        } else {
            Err("Expected a right bracket".to_string())
        }
    }

    fn parse_call_expression(&mut self, function: Node) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let mut arguments = vec![];

        while self.tokens.to_next().filter(|token| token.token_type != TokenType::RParenthesis).is_some() {
            arguments.push(self.parse_expression(Precedence::Lowest)?);

            if self.tokens.is_next_match(|token| token.token_type == TokenType::RParenthesis) {
                continue;
            }

            if let Some(token) = self.tokens.to_next().filter(|token| token.token_type != TokenType::Comma) {
                return Err(format!("Expected a comma, but got a {}", token.literal));
            }
        }

        Ok(Node::CallExpression {
            token,
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_member_expression(&mut self, instance: Node) -> Result<Node, String> {
        let token = self.tokens.current().unwrap().clone();
        let member = if let Some(token) = self.tokens.to_next() {
            if token.token_type == TokenType::Identifier {
                let identifier = Node::Identifier {
                    token: token.clone(),
                    value: token.literal.clone(),
                };
                if self.tokens.is_next_match(|tok| tok.token_type == TokenType::LParenthesis) {
                    // method
                    self.tokens.to_next();
                    self.parse_call_expression(identifier)?
                } else if self.tokens.is_next_match(|tok| tok.token_type == TokenType::LBracket) {
                    // index expression
                    self.tokens.to_next();
                    self.parse_index_expression(identifier)?
                } else {
                    // field
                    identifier
                }
            } else {
                return Err(format!("Expected a identifier, but got a {}", token.literal))
            }
        } else {
            return Err(String::from("Expected a identifier, but arrived at the end"))
        };

        Ok(Node::MemberExpression { token, instance: Box::new(instance), member: Box::new(member) })
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
        ㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎🪄↩️💬 ↙️
        📛 🈯 🌜🅰️🦶 🅱️🌛 🫸
          ⭕ 🅰️ ▶️🟰 0️⃣ 🔁 🅱️ ◀️🟰 5️⃣ 🫸
            🅰️ ⬅️ 🅰️ ➕ 🅱️ ↙️
            🅱️ ⬅️ 🅱️ ➖ 🅰️ ↙️
          🫷
          🔙 ❓ 🅰️ ▶️ 🅱️ 🫸🅰️🫷 ❗ 🫸🅱️🫷 ↙️
        🫷
        ⬅️⏸️🌜❌🟰0️⃣◀️1️⃣🌛 ↙️
        🈯 🌜🅰️🦶 🅱️🌛
        👉🅰️🦶 🅱️👈👉0️⃣👈
        🫸 🗨️🅰️💬 ➡️ 1️⃣🦶 🗨️🅱️💬 ➡️ 9️⃣ 🫷
        🗨️🈶🅰️🈚🅱️🈲🆎
            ",
        );
        let target_statements = [
            "㊙️🔢 ⬅️ 🌜1️⃣ ➕ 🌜3️⃣⚪9️⃣ ✖️ 7️⃣2️⃣🌛🌛 ↙️",
            "㊙️🔡 ⬅️ 🗨️🈶🅰️🈚🅱️🈲🆎\n💬 ↙️",
            "📛 🈯 🌜🅰️🦶 🅱️🌛 🫸 ⭕ 🌜🌜🅰️ ▶️🟰 0️⃣🌛 🔁 🌜🅱️ ◀️🟰 5️⃣🌛🌛 🫸 🅰️ ⬅️ 🌜🅰️ ➕ 🅱️🌛 ↙️🅱️ ⬅️ 🌜🅱️ ➖ 🅰️🌛 ↙️ 🫷 ↙️🔙 ❓ 🌜🅰️ ▶️ 🅱️🌛 🫸 🅰️ ↙️ 🫷 ❗ 🫸 🅱️ ↙️ 🫷 ↙️ 🫷 ↙️",
            "🌜⏸️🌜❌ 🟰 🌜0️⃣ ◀️ 1️⃣🌛🌛🌛 ↙️",
            "🈯🌜🅰️🦶 🅱️🌛 ↙️",
            "👉🅰️🦶 🅱️👈👉0️⃣👈 ↙️",
            "🫸🗨️🅰️💬 ➡️ 1️⃣🦶 🗨️🅱️💬 ➡️ 9️⃣🫷 ↙️",
        ];
        let target_errors = vec![
            "Expected a expression, but got a ⬅️",
            "Expected 🗨️ or 💬 at the end of a string literal",
        ];

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        if let Node::Program { statements } = &program {
            assert_eq!(statements.len(), target_statements.len());
        }
        assert_eq!(program.string(), target_statements.join(""));
        assert_eq!(parser.errors.len(), target_errors.len());
        assert_eq!(parser.errors, target_errors);
    }
}
