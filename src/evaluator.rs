use std::collections::HashMap;

use crate::types::{Node, object::*};

pub struct Environment {
    map: HashMap<String, Object>,
}

impl<'a> Environment {
    pub fn new() -> Self {
        Environment { map: HashMap::new() }
    }

    pub fn set(&mut self, identifier: String, value: Object) {
        self.map.insert(identifier, value);
    }

    pub fn get(&self, identifier: &String) -> Option<&Object> {
        self.map.get(identifier)
    }
}

pub fn eval(node: Node, env: &mut Environment) -> Result<Object, String> {
    match node {
        Node::Program { statements } => eval_program(statements, env),
        Node::ExpressionStatement { token: _, expression } => eval(*expression, env),
        Node::IntegerLiteral { token: _, value } => Ok(Object::Integer(value)),
        Node::FloatLiteral { token: _, value } => Ok(Object::Float(value)),
        Node::BooleanLiteral { token: _, value } => Ok(Object::Boolean(value)),
        Node::StringLiteral { token: _, value } => Ok(Object::String(value)),
        Node::PrefixExpression { token: _, operator, right } => eval_prefix_expression(operator, eval(*right, env)?),
        Node::InfixExpression { token: _, left, operator, right } => eval_infix_expression(operator, eval(*left, env)?, eval(*right, env)?),
        Node::BlockStatement { token: _, statements } => eval_block_statements(statements, env),
        Node::IfExpression { token: _, condition, consequence, alternative } => eval_if_expression(condition, consequence, alternative, env),
        Node::ReturnStatement { token: _, value } => Ok(Object::ReturnValue(Box::new(eval(*value, env)?))),
        Node::AssignStatement { token: _, name, value } => {
            let value = eval(*value, env)?;
            // todo assert name is identifier
            env.set(name.string(), value.clone());
            Ok(value)
        },
        Node::Identifier { token: _, value } => eval_identifier(&value, env),
        _ => Err(String::from("Invalid expressions or statements to evaluate values"))
    }
}

fn eval_program(statements: Vec<Node>, env: &mut Environment) -> Result<Object, String> {
    let mut result = Err(String::from("Empty statements to evaluate values"));
    for statement in statements {
        result = eval(statement, env);
        if let Ok(Object::ReturnValue(value)) = result {
            return Ok(*value);
        }
    }
    result
}

fn eval_block_statements(statements: Vec<Node>, env: &mut Environment) -> Result<Object, String> {
    let mut result = Err(String::from("Empty statements to evaluate values"));
    for statement in statements {
        result = eval(statement, env);
        if let Ok(Object::ReturnValue(_)) = &result {
            return result;
        }
    }
    result
}

fn eval_prefix_expression(operator: String, right: Object) -> Result<Object, String> {
    match operator.as_str() {
        "⏸️" => eval_prefix_not_expression(&right),
        "➖" => eval_prefix_minus_expression(&right),
        _ => Err(String::from("Invalid prefix expressions to evaluate values"))
    }
}

fn eval_prefix_not_expression(obj: &Object) -> Result<Object, String> {
    if let Object::ReturnValue(_) = &obj {
        return Err(String::from("Invalid prefix not expression to evaluate return expression"));
    }
    let value = match obj {
        Object::Integer(value) => *value > 0 ,
        Object::Float(value) => *value > 0.0,
        Object::Boolean(value) => *value,
        Object::String(value) => !value.is_empty(),
        Object::Null => false,
        _ => false,
    };
    Ok(to_bool_object(!value))
}

fn eval_prefix_minus_expression(obj: &Object) -> Result<Object, String> {
    match obj {
        Object::Integer(value) => Ok(Object::Integer(-value)) ,
        Object::Float(value) => Ok(Object::Float(-value)) ,
        _ => Err(String::from("Invalid prefix minus expression to evaluate non-numeric value")),
    }
}

fn eval_infix_expression(operator: String, left: Object, right: Object) -> Result<Object, String> {
    if let Object::Integer(left) = left && let Object::Integer(right) = right {
        eval_integer_infix_expression(operator, left, right)
    } else if let Object::Integer(left) = left && let Object::Float(right) = right {
        eval_float_infix_expression(operator, left as f64, right)
    } else if let Object::Float(left) = left && let Object::Float(right) = right {
        eval_float_infix_expression(operator, left, right)
    } else if let Object::Float(left) = left && let Object::Integer(right) = right {
        eval_float_infix_expression(operator, left, right as f64)
    } else if operator == "🟰" {
        Ok(to_bool_object(left == right))
    } else if operator == "❗🟰" {
        Ok(to_bool_object(left != right))
    } else {
        Err(String::from("Invalid infix expression"))
    }
}

fn eval_integer_infix_expression(operator: String, left: i64, right: i64) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => Ok(Object::Integer(left + right)),
        "➖" => Ok(Object::Integer(left - right)),
        "✖️" => Ok(Object::Integer(left * right)),
        "➗" => Ok(Object::Integer(left / right)),
        "〰️" => Ok(Object::Integer(left % right)),
        "🟰" => Ok(to_bool_object(left == right)),
        "❗🟰" => Ok(to_bool_object(left != right)),
        "▶️" => Ok(to_bool_object(left > right)),
        "▶️🟰" => Ok(to_bool_object(left >= right)),
        "◀️" => Ok(to_bool_object(left < right)),
        "◀️🟰" => Ok(to_bool_object(left <= right)),
        _ => Err(String::from("Invalid infix expression operator"))
    }
}

fn eval_float_infix_expression(operator: String, left: f64, right: f64) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => Ok(Object::Float(left + right)),
        "➖" => Ok(Object::Float(left - right)),
        "✖️" => Ok(Object::Float(left * right)),
        "➗" => Ok(Object::Float(left / right)),
        "〰️" => Ok(Object::Float(left % right)),
        "🟰" => Ok(to_bool_object(left == right)),
        "❗🟰" => Ok(to_bool_object(left != right)),
        "▶️" => Ok(to_bool_object(left > right)),
        "▶️🟰" => Ok(to_bool_object(left >= right)),
        "◀️" => Ok(to_bool_object(left < right)),
        "◀️🟰" => Ok(to_bool_object(left <= right)),
        _ => Err(String::from("Invalid infix expression operator"))
    }
}

fn eval_if_expression(condition: Box<Node>, consequence: Box<Node>, alternative: Option<Box<Node>>, env: &mut Environment) -> Result<Object, String> {
    let condition = match eval(*condition, env)? {
        Object::Null => false,
        Object::Boolean(boolean) => boolean,
        _ => true,
    };

    if condition {
        eval(*consequence, env)
    } else if let Some(alternative) = alternative {
        eval(*alternative, env)
    } else {
        Ok(NULL)
    }
}

fn eval_identifier(value: &String, env: &Environment) -> Result<Object, String> {
    env.get(value)
        .map(|obj| obj.clone())
        .ok_or_else(|| format!("identifier not found: {value}"))
}

fn to_bool_object(value: bool) -> Object {
    if value { TRUE } else { FALSE }
}


#[cfg(test)]
mod evaluator_test {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test() {
        let source = String::from(
                "
        1️⃣⚪3️⃣ ➕ 9️⃣ ↙️
        #️⃣ ⏸️❌↙️
        🅰️ ⬅️ ❓ 1️⃣ ▶️🟰 3️⃣ 🫸 9️⃣ 🫷 ❗ 🫸 1️⃣ 🫷 
        🅰️ ↙️
        ",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let mut env = Environment::new();
        let evaluated = eval(program, &mut env);

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::Integer(1));
    }
}
