use crate::types::{Node, object::*};


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
        Node::IfExpression { token: _, condition, consequence, alternative } => eval_if_expression(*condition, *consequence, alternative, env),
        Node::WhileExpression { token: _, condition, body } => eval_while_expression(*condition, *body, env),
        Node::ReturnStatement { token: _, value } => Ok(Object::ReturnValue(Box::new(eval(*value, env)?))),
        Node::AssignStatement { token: _, name, value } => {
            let value = eval(*value, env)?;
            // todo assert name is identifier
            env.set(name.string(), value.clone());
            Ok(value)
        },
        Node::Identifier { token: _, value } => eval_identifier(&value, env),
        Node::FunctionLiteral { token: _, name, parameters, body } => {
            let function = Object::Function { parameters, body, env: Box::new(env.clone()) };
            if let Some(name) = name {
                env.set(name.string(), function.clone());
            }
            Ok(function)
        },
        Node::CallExpression { token: _, function, arguments } => {
            let function = eval(*function, env)?;
            let args = eval_expressions(arguments, env)?;
            apply_function(function, args)
        },
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
    } else if let Object::Boolean(left) = left && let Object::Boolean(right) = right {
        eval_boolean_infix_expression(operator, left, right)
    } else if let Object::String(ref left) = left && let Object::String(ref right) = right {
        eval_string_infix_expression(operator, left, right)
    } else if operator == "🟰" {
        Ok(to_bool_object(left == right))
    } else if operator == "❗🟰" {
        Ok(to_bool_object(left != right))
    } else {
        Err(format!("Invalid infix expression: {:?} {} {:?}", left, operator, right))
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

fn eval_boolean_infix_expression(operator: String, left: bool, right: bool) -> Result<Object, String> {
    match operator.as_str() {
        "🟰" => Ok(to_bool_object(left == right)),
        "❗🟰" => Ok(to_bool_object(left != right)),
        "🔁" => Ok(to_bool_object(left && right)),
        "🔀" => Ok(to_bool_object(left || right)),
        _ => Err(String::from("Invalid infix expression operator"))
    }
}

fn eval_string_infix_expression(operator: String, left: &str, right: &str) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => {
            let mut join = String::from(left);
            join.push_str(right);
            Ok(Object::String(join))
        },
        "🟰" => Ok(to_bool_object(left == right)),
        "❗🟰" => Ok(to_bool_object(left != right)),
        _ => Err(String::from("Invalid infix expression operator"))
    }
}

fn eval_if_expression(condition: Node, consequence: Node, alternative: Option<Box<Node>>, env: &mut Environment) -> Result<Object, String> {
    if eval_condition(condition, env)? {
        eval(consequence, env)
    } else if let Some(alternative) = alternative {
        eval(*alternative, env)
    } else {
        Ok(NULL)
    }
}

fn eval_while_expression(condition: Node, body: Node, env: &mut Environment) -> Result<Object, String> {
    while eval_condition(condition.clone(), env)? {
        eval(body.clone(), env)?;
    }
    Ok(NULL)
}

fn eval_condition(condition: Node, env: &mut Environment) -> Result<bool, String> {
    Ok(match eval(condition, env)? {
        Object::Null => false,
        Object::Boolean(boolean) => boolean,
        _ => true,
    })
}

fn eval_identifier(value: &String, env: &Environment) -> Result<Object, String> {
    env.get(value)
        .cloned()
        .ok_or_else(|| format!("identifier not found: {value}"))
}

fn eval_expressions(arguments: Vec<Node>, env: &mut Environment) -> Result<Vec<Object>, String> {
    let mut args = vec![];
    for arg in arguments {
        args.push(eval(arg, env)?);
    }
    Ok(args)
}

fn apply_function(function: Object, args: Vec<Object>) -> Result<Object, String> {
    if let Object::Function { parameters, body, env } = function {
        let mut env = Environment::new_enclosed(env);
        for (index, param) in parameters.iter().enumerate() {
            if let Node::Identifier { token: _, value } = param {
                env.set(value.clone(), args.get(index).unwrap().clone());
            } else {
                return Err(format!("Not a identifier: {}", param.string()))
            }
        }
        let return_val = eval(*body, &mut env)?;
        if let Object::ReturnValue(value) = return_val {
            Ok(*value)
        } else {
            Ok(return_val)
        }
    } else {
        Err(format!("Not a function: {}", function.inspect()))
    }
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
        1️⃣⚪3️⃣ ➕ 9️⃣
        #️⃣ ⏸️❌
        📛 🈯 🌜🅰️🦶 🅱️🌛 🫸
          ⭕ 🅰️ ▶️🟰 0️⃣ 🔁 🅱️ ◀️🟰 5️⃣ 🫸
            🅰️ ⬅️ 🅰️ ➖ 1️⃣
            🅱️ ⬅️ 🅱️ ➕ 1️⃣
          🫷
          🔙 ❓ 🅰️ ▶️ 🅱️ 🫸🅰️🫷 ❗ 🫸🅱️🫷
        🫷
        🅰️ ⬅️ 🈯🌜1️⃣🦶 3️⃣🌛
        🅰️
        #️⃣ 🗨️🈶🅰️🈚🅱️💬 ➕ 🗨️🈲🆎💬
        ",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let mut env = Environment::new();
        let evaluated = eval(program, &mut env);

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::Integer(5));
    }
}
