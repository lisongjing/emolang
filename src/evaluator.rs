use std::collections::HashMap;

use crate::types::{Node, Token, object::*};

pub fn eval(node: Node, env: &mut Environment) -> Result<Object, String> {
    match node {
        Node::Program { statements } => eval_program(statements, env),
        Node::ExpressionStatement { expression } => eval(*expression, env),
        Node::IntegerLiteral { value } => Ok(Object::new_integer(value)),
        Node::FloatLiteral { value } => Ok(Object::new_float(value)),
        Node::BooleanLiteral { value } => Ok(Object::new_boolean(value)),
        Node::StringLiteral { value } => Ok(Object::new_string(value)),
        Node::ListLiteral { elements } => eval_list_literal(elements, env),
        Node::MapLiteral { entries } => eval_map_literal(entries, env),
        Node::PrefixExpression { operator, right } => {
            eval_prefix_expression(operator, eval(*right, env)?)
        }
        Node::InfixExpression {
            left,
            operator,
            right,
        } => eval_infix_expression(operator, eval(*left, env)?, eval(*right, env)?),
        Node::IndexExpression {
            collection: left,
            index,
        } => eval_index_expression(eval(*left, env)?, eval(*index, env)?),
        Node::BlockStatement { statements } => eval_block_statements(statements, env),
        Node::IfExpression {
            condition,
            consequence,
            alternative,
        } => eval_if_expression(*condition, *consequence, alternative, env),
        Node::WhileExpression { condition, body } => eval_while_expression(*condition, *body, env),
        Node::BreakExpression { value } => eval_break_expression(value, env),
        Node::ReturnStatement { value } => Ok(Object::new_return_value(eval(*value, env)?)),
        Node::AssignExpression { identifier, value } => {
            eval_assign_expression(*identifier, *value, env)
        }
        Node::Identifier { value } => eval_identifier(&value, env),
        Node::FunctionLiteral {
            name,
            parameters,
            body,
        } => {
            let function = Object::new_function(parameters, body, env.clone());
            if let Some(name) = name {
                env.set(name.string(), function.clone());
            }
            Ok(function)
        }
        Node::CallExpression {
            function,
            arguments,
        } => {
            let function = eval(*function, env)?;
            let args = eval_expressions(arguments, env)?;
            apply_function(function, args)
        }
        Node::MemberExpression { instance, member } => {
            eval_member_expression(eval(*instance, env)?, *member)
        }
    }
}

fn eval_program(statements: Vec<Node>, env: &mut Environment) -> Result<Object, String> {
    let mut result = Err(String::from("Empty statements to evaluate values"));
    for statement in statements {
        result = eval(statement, env);
        if let Ok(ref obj) = result
            && let ObjectValue::ReturnValue(value) = obj.value()
        {
            return Ok(*value.clone());
        }
    }
    result
}

fn eval_block_statements(statements: Vec<Node>, env: &mut Environment) -> Result<Object, String> {
    let mut result = Err(String::from("Empty statements to evaluate values"));
    for statement in statements {
        result = eval(statement, env);
        if let Ok(ref obj) = result
            && let ObjectValue::ReturnValue(_) = obj.value()
        {
            return result;
        }
    }
    result
}

fn eval_assign_expression(
    identifier: Node,
    value: Node,
    env: &mut Environment,
) -> Result<Object, String> {
    let value_object = eval(value, env)?;
    match identifier {
        Node::Identifier { value } => {
            env.set(value, value_object.clone());
            Ok(value_object)
        }
        Node::IndexExpression {
            collection: left,
            index,
        } => {
            let mut collection_object = eval(*left.clone(), env)?;
            let index_object = eval(*index, env)?;
            match collection_object.value_mut() {
                ObjectValue::List(elements) => {
                    if let ObjectValue::Integer(index) = index_object.value()
                        && *index >= 0
                    {
                        if let Some(element) = elements.get_mut(*index as usize) {
                            *element = value_object.clone();
                            if let Node::Identifier { value } = *left {
                                env.set(value, Object::new_list(elements.to_owned()));
                            }
                            Ok(value_object)
                        } else {
                            Err(format!("Invalid index: {index}"))
                        }
                    } else {
                        Err(String::from(
                            "Index must be an integer greater than or equal to 0",
                        ))
                    }
                }
                ObjectValue::Map(entries) => {
                    if let Some(element) = entries.get_mut(&index_object) {
                        *element = value_object.clone();
                        if let Node::Identifier { value } = *left {
                            env.set(value, Object::new_map(entries.to_owned()));
                        }
                        Ok(value_object)
                    } else {
                        Err(format!("Invalid index: {index_object:?}"))
                    }
                }
                _ => Err(String::from("Invalid collection type in index expression")),
            }
        }
        Node::MemberExpression { instance, member } => {
            let mut instance_object = eval(*instance.clone(), env)?;
            if let Node::Identifier { value } = *member {
                let env = instance_object.associated_env_mut();
                env.set(value, value_object.clone());
                // instance_object.set_associated_env(env);
            }
            if let Node::Identifier { value } = *instance {
                env.set(value, instance_object);
            }
            Ok(value_object)
        }
        _ => Err(format!(
            "Expected identifier / index expression / member expression, but got {}",
            identifier.string()
        )),
    }
}

fn eval_list_literal(elements: Vec<Node>, env: &mut Environment) -> Result<Object, String> {
    let mut value = vec![];
    for node in elements {
        value.push(eval(node, env)?);
    }
    Ok(Object::new_list(value))
}

fn eval_map_literal(entries: Vec<(Node, Node)>, env: &mut Environment) -> Result<Object, String> {
    let mut value = HashMap::new();
    for (key, val) in entries {
        value.insert(eval(key, env)?, eval(val, env)?);
    }
    Ok(Object::new_map(value))
}

fn eval_prefix_expression(operator: String, right: Object) -> Result<Object, String> {
    match operator.as_str() {
        "⏸️" => eval_prefix_not_expression(&right),
        "➖" => eval_prefix_minus_expression(&right),
        _ => Err(String::from(
            "Invalid prefix expressions to evaluate values",
        )),
    }
}

fn eval_prefix_not_expression(obj: &Object) -> Result<Object, String> {
    if let ObjectValue::ReturnValue(_) = &obj.value() {
        return Err(String::from(
            "Invalid prefix not expression to evaluate return expression",
        ));
    }
    let value = match obj.value() {
        ObjectValue::Integer(value) => *value > 0,
        ObjectValue::Float(value) => *value > 0.0,
        ObjectValue::Boolean(value) => *value,
        ObjectValue::String(value) => !value.is_empty(),
        ObjectValue::Null => false,
        ObjectValue::List(value) => !value.is_empty(),
        ObjectValue::Map(value) => !value.is_empty(),
        _ => false,
    };
    Ok(Object::new_boolean(!value))
}

fn eval_prefix_minus_expression(obj: &Object) -> Result<Object, String> {
    match obj.value() {
        ObjectValue::Integer(value) => Ok(Object::new_integer(-value)),
        ObjectValue::Float(value) => Ok(Object::new_float(-value)),
        _ => Err(String::from(
            "Invalid prefix minus expression to evaluate non-numeric value",
        )),
    }
}

fn eval_infix_expression(operator: String, left: Object, right: Object) -> Result<Object, String> {
    if let ObjectValue::Integer(left) = left.value()
        && let ObjectValue::Integer(right) = right.value()
    {
        eval_integer_infix_expression(operator, *left, *right)
    } else if let ObjectValue::Integer(left) = left.value()
        && let ObjectValue::Float(right) = right.value()
    {
        eval_float_infix_expression(operator, *left as f64, *right)
    } else if let ObjectValue::Float(left) = left.value()
        && let ObjectValue::Float(right) = right.value()
    {
        eval_float_infix_expression(operator, *left, *right)
    } else if let ObjectValue::Float(left) = left.value()
        && let ObjectValue::Integer(right) = right.value()
    {
        eval_float_infix_expression(operator, *left, *right as f64)
    } else if let ObjectValue::Boolean(left) = left.value()
        && let ObjectValue::Boolean(right) = right.value()
    {
        eval_boolean_infix_expression(operator, *left, *right)
    } else if let ObjectValue::String(left) = left.value()
        && let ObjectValue::String(right) = right.value()
    {
        eval_string_infix_expression(operator, left, right)
    } else if let ObjectValue::List(left) = left.value()
        && let ObjectValue::List(right) = right.value()
    {
        eval_list_infix_expression(operator, left, right)
    } else if operator == "🟰" {
        Ok(Object::new_boolean(left == right))
    } else if operator == "❗🟰" {
        Ok(Object::new_boolean(left != right))
    } else {
        Err(format!(
            "Invalid infix expression: {:?} {} {:?}",
            left, operator, right
        ))
    }
}

fn eval_integer_infix_expression(
    operator: String,
    left: i64,
    right: i64,
) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => Ok(Object::new_integer(left + right)),
        "➖" => Ok(Object::new_integer(left - right)),
        "✖️" => Ok(Object::new_integer(left * right)),
        "➗" => Ok(Object::new_integer(left / right)),
        "〰️" => Ok(Object::new_integer(left % right)),
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        "▶️" => Ok(Object::new_boolean(left > right)),
        "▶️🟰" => Ok(Object::new_boolean(left >= right)),
        "◀️" => Ok(Object::new_boolean(left < right)),
        "◀️🟰" => Ok(Object::new_boolean(left <= right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_float_infix_expression(operator: String, left: f64, right: f64) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => Ok(Object::new_float(left + right)),
        "➖" => Ok(Object::new_float(left - right)),
        "✖️" => Ok(Object::new_float(left * right)),
        "➗" => Ok(Object::new_float(left / right)),
        "〰️" => Ok(Object::new_float(left % right)),
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        "▶️" => Ok(Object::new_boolean(left > right)),
        "▶️🟰" => Ok(Object::new_boolean(left >= right)),
        "◀️" => Ok(Object::new_boolean(left < right)),
        "◀️🟰" => Ok(Object::new_boolean(left <= right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_boolean_infix_expression(
    operator: String,
    left: bool,
    right: bool,
) -> Result<Object, String> {
    match operator.as_str() {
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        "🔁" => Ok(Object::new_boolean(left && right)),
        "🔀" => Ok(Object::new_boolean(left || right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_string_infix_expression(
    operator: String,
    left: &str,
    right: &str,
) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => {
            let mut join = String::from(left);
            join.push_str(right);
            Ok(Object::new_string(join))
        }
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_list_infix_expression(
    operator: String,
    left: &Vec<Object>,
    right: &Vec<Object>,
) -> Result<Object, String> {
    match operator.as_str() {
        "➕" => {
            let mut union = left.clone();
            union.extend_from_slice(right);
            Ok(Object::new_list(union))
        }
        "➖" => {
            let difference = left
                .clone()
                .into_iter()
                .filter(|x| !right.contains(x))
                .collect::<Vec<Object>>();
            Ok(Object::new_list(difference))
        }
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_index_expression(left: Object, index: Object) -> Result<Object, String> {
    match left.value() {
        ObjectValue::List(elements) => {
            if let ObjectValue::Integer(index) = index.value()
                && *index >= 0
            {
                elements
                    .get(*index as usize)
                    .cloned()
                    .ok_or_else(|| format!("Invalid index: {index}"))
            } else {
                Err(String::from(
                    "Index must be an integer greater than or equal to 0",
                ))
            }
        }
        ObjectValue::Map(entries) => entries
            .get(&index)
            .cloned()
            .ok_or_else(|| format!("Invalid index: {index:?}")),
        _ => Err(String::from("Invalid collection type to index")),
    }
}

fn eval_if_expression(
    condition: Node,
    consequence: Node,
    alternative: Option<Box<Node>>,
    env: &mut Environment,
) -> Result<Object, String> {
    if eval_condition(condition, env)? {
        eval(consequence, env)
    } else if let Some(alternative) = alternative {
        eval(*alternative, env)
    } else {
        Ok(Object::new_null())
    }
}

fn eval_while_expression(
    condition: Node,
    body: Node,
    env: &mut Environment,
) -> Result<Object, String> {
    while eval_condition(condition.clone(), env)? {
        eval(body.clone(), env)?;
    }
    Ok(Object::new_null())
}

fn eval_break_expression(
    _break_value: Option<Box<Node>>,
    _env: &mut Environment,
) -> Result<Object, String> {
    // let value = if let Some(value) = break_value {
    //     Some(Box::new(eval(*value, env)?))
    // } else {
    //     None
    // };
    // Ok(Object::new_break(value))
    Ok(Object::new_null())
}

fn eval_condition(condition: Node, env: &mut Environment) -> Result<bool, String> {
    Ok(match eval(condition, env)?.value() {
        ObjectValue::Null => false,
        ObjectValue::Boolean(boolean) => *boolean,
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

fn eval_member_expression(mut instance: Object, right: Node) -> Result<Object, String> {
    let env = instance.associated_env_mut();
    let right = if let Node::CallExpression {
        function,
        mut arguments,
    } = right
    {
        let self_token = Token::this();
        arguments.insert(
            0,
            Node::Identifier {
                value: self_token.literal,
            },
        );

        Node::CallExpression {
            function,
            arguments,
        }
    } else {
        right
    };
    eval(right, env)
}

fn apply_function(function: Object, args: Vec<Object>) -> Result<Object, String> {
    match function.value() {
        ObjectValue::Function {
            parameters,
            body,
            env,
        } => {
            if parameters.len() != args.len() {
                return Err(format!(
                    "Expected {} argument(s), but got {}",
                    parameters.len(),
                    args.len()
                ));
            }
            let mut env = Environment::new_enclosed(env.clone());
            for (index, param) in parameters.iter().enumerate() {
                if let Node::Identifier { value } = param {
                    env.set(value.clone(), args.get(index).unwrap().clone());
                } else {
                    return Err(format!("Not a identifier: {}", param.string()));
                }
            }
            let return_val = eval(*body.clone(), &mut env)?;
            if let ObjectValue::ReturnValue(value) = return_val.value() {
                Ok(*value.clone())
            } else {
                Ok(return_val)
            }
        }
        ObjectValue::BuiltinFunction(function) => function.call(&args),
        _ => Err(format!("Not a function: {}", function.inspect())),
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
        🅰️ ⬅️ 👉🅰️🦶 1️⃣🦶 3️⃣👈
        🅰️ ⬅️ 🅰️👉3️⃣ ➖ 3️⃣👈
        🅱️ ⬅️ 🗨️🅰️ 🟰 💬 ➕ 👁️‍🗨️🌜🅰️🌛
        🅱️ ⬅️ 🫸 🅱️ ➡️ 🅰️🦶 🗨️9️⃣💬 ➡️ 9️⃣ 🫷👉🅱️👈
        🅱️❇️💕🌜3️⃣🌛 ➖ 🗨️🅰️ 🟰 💬❇️📏🌜🌛
        ",
        );

        let mut lexer = Lexer::new(&source);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        let mut env = Environment::new_default();
        let evaluated = eval(program, &mut env);

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::new_integer(121));
    }
}
