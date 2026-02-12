use crate::types::{Node, object::*};

pub fn eval(node: Node) -> Result<Object, String> {
    match node {
        Node::Program { statements } => eval_statements(statements),
        Node::ExpressionStatement { token: _, expression } => eval(*expression),
        Node::IntegerLiteral { token: _, value } => Ok(Object::Integer(value)),
        Node::FloatLiteral { token: _, value } => Ok(Object::Float(value)),
        Node::BooleanLiteral { token: _, value } => Ok(Object::Boolean(value)),
        Node::StringLiteral { token: _, value } => Ok(Object::String(value)),
        Node::PrefixExpression { token: _, operator, right } => eval_prefix_expression(operator, eval(*right)?),
        Node::InfixExpression { token: _, left, operator, right } => eval_infix_expression(operator, eval(*left)?, eval(*right)?),
        _ => Err(String::from("Invalid expressions or statements to evaluate values"))
    }
}

fn eval_statements(statements: Vec<Node>) -> Result<Object, String> {
    let mut result = Err(String::from("Empty statements to evaluate values"));
    for statement in statements {
        result = eval(statement);
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
    let value = match obj {
        Object::Integer(value) => *value > 0 ,
        Object::Float(value) => *value > 0.0,
        Object::Boolean(value) => *value,
        Object::String(value) => !value.is_empty(),
        Object::Null => false,
    };
    Ok(if value { FALSE } else { TRUE })
}

fn eval_prefix_minus_expression(obj: &Object) -> Result<Object, String> {
    match obj {
        Object::Integer(value) => Ok(Object::Integer(-value)) ,
        Object::Float(value) => Ok(Object::Float(-value)) ,
        _ => Err(String::from("Invalid prefix minus expressions to evaluate non-numeric values")),
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
        _ => Err(String::from("Invalid infix expression operator"))
    }
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
        #️⃣ ⏸️❌↙️",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let evaluated = eval(program);

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::Float(10.3));
    }
}
