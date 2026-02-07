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
        "⏸️" => Ok(match right {
            Object::Integer(value) => Object::Boolean(value <= 0),
            Object::Float(value) => Object::Boolean(value <= 0.0),
            Object::Boolean(value) => Object::Boolean(!value),
            Object::String(value) => Object::Boolean(!value.is_empty()),
            Object::Null => Object::Boolean(true),
        }),
        _ => Err(String::from("Invalid prefix expressions to evaluate values"))
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
        1️⃣ ↙️
        ⏸️❌↙️",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let evaluated = eval(program);

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::Boolean(true));
    }
}
