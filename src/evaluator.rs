use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::types::{Node, Token, object::*};

pub fn eval(node: &Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    match node {
        Node::Program { statements } => eval_program(statements, env),
        Node::ExpressionStatement { expression } => eval(expression, env),
        Node::IntegerLiteral { value } => Ok(Object::new_integer(*value)),
        Node::FloatLiteral { value } => Ok(Object::new_float(*value)),
        Node::BooleanLiteral { value } => Ok(Object::new_boolean(*value)),
        Node::StringLiteral { value } => Ok(Object::new_string(value.clone())),
        Node::ListLiteral { elements } => eval_list_literal(elements, env),
        Node::MapLiteral { entries } => eval_map_literal(entries, env),
        Node::PrefixExpression { operator, right } => {
            eval_prefix_expression(operator, &eval(right, env)?)
        }
        Node::InfixExpression {
            left,
            operator,
            right,
        } => eval_infix_expression(operator, &eval(left, Rc::clone(&env))?, &eval(right, env)?),
        Node::IndexExpression { collection, index } => {
            eval_index_expression(&eval(collection, Rc::clone(&env))?, &eval(index, env)?)
        }
        Node::BlockStatement { statements } => eval_block_statements(statements, env),
        Node::IfExpression {
            condition,
            consequence,
            alternative,
        } => eval_if_expression(condition, consequence, alternative, env),
        Node::WhileExpression { condition, body } => eval_while_expression(condition, body, env),
        Node::ContinueStatement => Ok(Object::new_continue()),
        Node::BreakStatement { value } => eval_break_statement(value, env),
        Node::ReturnStatement { value } => Ok(Object::new_return_value(eval(value, env)?)),
        Node::AssignExpression { identifier, value } => {
            eval_assign_expression(identifier, value, env)
        }
        Node::Identifier { value } => eval_identifier(value, env),
        Node::FunctionLiteral {
            name,
            parameters,
            body,
        } => {
            let function = Object::new_function(parameters.clone(), body.clone(), &env);
            if let Some(name) = name {
                env.borrow_mut().set(name.string(), function.clone());
            }
            Ok(function)
        }
        Node::StructDefinition { name, properties } => eval_type_definition(name, properties, env),
        Node::NewStructLiteral { name, properties } => {
            eval_new_struct_instance(name, properties, env)
        }
        Node::CallExpression {
            function,
            arguments,
        } => {
            let function = eval(function, Rc::clone(&env))?;
            let args = eval_expressions(arguments, env)?;
            apply_function(function, args)
        }
        Node::MemberExpression { instance, member } => {
            eval_member_expression(&mut eval(instance, env)?, member)
        }
    }
}

fn eval_program(statements: &Vec<Node>, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    if statements.is_empty() {
        return Err(String::from("Empty statements to evaluate values"));
    }

    let mut obj = Object::new_null();
    for statement in statements {
        obj = eval(statement, Rc::clone(&env))?;
        if let ObjectValue::ReturnValue(value) = obj.value() {
            return Ok(*value.clone());
        }
    }
    Ok(obj)
}

fn eval_block_statements(
    statements: &Vec<Node>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    if statements.is_empty() {
        return Err(String::from("Empty statements to evaluate values"));
    }

    let mut obj = Object::new_null();
    for statement in statements {
        obj = eval(statement, Rc::clone(&env))?;
        if let ObjectValue::ReturnValue(_) = obj.value() {
            return Ok(obj);
        }
        if let ObjectValue::Break(_) = obj.value() {
            return Ok(obj);
        }
        if let ObjectValue::Continue = obj.value() {
            return Ok(obj);
        }
    }
    Ok(obj)
}

fn eval_assign_expression(
    identifier: &Node,
    value: &Node,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let value_object = eval(value, Rc::clone(&env))?;
    match identifier {
        Node::Identifier { value } => {
            env.borrow_mut().set(value.clone(), value_object.clone());
            Ok(value_object)
        }
        Node::IndexExpression { collection, index } => {
            let collection_object = eval(collection, Rc::clone(&env))?;
            let index_object = eval(index, Rc::clone(&env))?;
            match collection_object.value_owned() {
                ObjectValue::List(mut elements) => {
                    if let ObjectValue::Integer(index) = index_object.value()
                        && *index >= 0
                    {
                        if let Some(element) = elements.get_mut(*index as usize) {
                            *element = value_object.clone();
                            if let Node::Identifier { value } = &**collection {
                                env.borrow_mut()
                                    .set(value.clone(), Object::new_list(elements));
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
                ObjectValue::Map(mut entries) => {
                    if let Some(element) = entries.get_mut(&index_object) {
                        *element = value_object.clone();
                        if let Node::Identifier { value } = &**collection {
                            env.borrow_mut()
                                .set(value.clone(), Object::new_map(entries));
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
            let instance_object = eval(instance, Rc::clone(&env))?;
            if let Node::Identifier { value } = &**member {
                let env = instance_object.associated_env();
                env.borrow_mut().set(value.clone(), value_object.clone());
            }
            if let Node::Identifier { value } = &**instance {
                env.borrow_mut().set(value.clone(), instance_object);
            }
            Ok(value_object)
        }
        _ => Err(format!(
            "Expected identifier / index expression / member expression, but got {}",
            identifier.string()
        )),
    }
}

fn eval_list_literal(
    elements: &Vec<Node>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut value = vec![];
    for node in elements {
        value.push(eval(node, Rc::clone(&env))?);
    }
    Ok(Object::new_list(value))
}

#[allow(clippy::mutable_key_type)]
fn eval_map_literal(
    entries: &Vec<(Node, Node)>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut value = HashMap::new();
    for (key, val) in entries {
        value.insert(eval(key, Rc::clone(&env))?, eval(val, Rc::clone(&env))?);
    }
    Ok(Object::new_map(value))
}

fn eval_prefix_expression(operator: &str, right: &Object) -> Result<Object, String> {
    match operator {
        "⏸️" => eval_prefix_not_expression(right),
        "➖" => eval_prefix_minus_expression(right),
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

fn eval_infix_expression(operator: &str, left: &Object, right: &Object) -> Result<Object, String> {
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

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Result<Object, String> {
    match operator {
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

fn eval_float_infix_expression(operator: &str, left: f64, right: f64) -> Result<Object, String> {
    match operator {
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
    operator: &str,
    left: bool,
    right: bool,
) -> Result<Object, String> {
    match operator {
        "🟰" => Ok(Object::new_boolean(left == right)),
        "❗🟰" => Ok(Object::new_boolean(left != right)),
        "🔁" => Ok(Object::new_boolean(left && right)),
        "🔀" => Ok(Object::new_boolean(left || right)),
        _ => Err(String::from("Invalid infix expression operator")),
    }
}

fn eval_string_infix_expression(operator: &str, left: &str, right: &str) -> Result<Object, String> {
    match operator {
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
    operator: &str,
    left: &Vec<Object>,
    right: &Vec<Object>,
) -> Result<Object, String> {
    match operator {
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

fn eval_index_expression(collection: &Object, index: &Object) -> Result<Object, String> {
    match collection.value() {
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
            .get(index)
            .cloned()
            .ok_or_else(|| format!("Invalid index: {index:?}")),
        _ => Err(String::from("Invalid collection type to index")),
    }
}

fn eval_if_expression(
    condition: &Node,
    consequence: &Node,
    alternative: &Option<Box<Node>>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    if eval_condition(condition, Rc::clone(&env))? {
        eval(consequence, env)
    } else if let Some(alternative) = alternative {
        eval(alternative, env)
    } else {
        Ok(Object::new_null())
    }
}

fn eval_while_expression(
    condition: &Node,
    body: &Node,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let mut return_val = Object::new_null();
    while eval_condition(condition, Rc::clone(&env))? {
        let body_obj = eval(body, Rc::clone(&env))?;
        if let ObjectValue::Break(value) = body_obj.value() {
            if let Some(obj) = value {
                return_val = *obj.clone();
            }
            break;
        } else if let ObjectValue::Continue = body_obj.value() {
            continue;
        }
    }
    Ok(return_val)
}

fn eval_break_statement(
    break_value: &Option<Box<Node>>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let value = if let Some(value) = break_value {
        Some(Box::new(eval(value, env)?))
    } else {
        None
    };
    Ok(Object::new_break(value))
}

fn eval_condition(condition: &Node, env: Rc<RefCell<Environment>>) -> Result<bool, String> {
    Ok(match eval(condition, env)?.value() {
        ObjectValue::Null => false,
        ObjectValue::Boolean(boolean) => *boolean,
        _ => true,
    })
}

fn eval_identifier(value: &String, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    env.borrow()
        .get(value)
        .ok_or_else(|| format!("identifier not found: {value}"))
}

fn eval_expressions(
    arguments: &Vec<Node>,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Object>, String> {
    let mut args = vec![];
    for arg in arguments {
        args.push(eval(arg, Rc::clone(&env))?);
    }
    Ok(args)
}

fn eval_member_expression(instance: &mut Object, member: &Node) -> Result<Object, String> {
    let env = instance.associated_env();
    let member = if let Node::CallExpression {
        function,
        arguments,
    } = member
    {
        let this_token = Token::this();
        let mut arguments = arguments.clone();
        arguments.insert(
            0,
            Node::Identifier {
                value: this_token.literal,
            },
        );

        &Node::CallExpression {
            function: function.clone(),
            arguments,
        }
    } else {
        member
    };
    eval(member, env)
}

fn eval_type_definition(
    name: &Node,
    properties_list: &Vec<Node>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let name = if let Node::Identifier { value } = name {
        Some(value.clone())
    } else {
        return Err(format!("Expected identifier, but got {}", name.string()));
    };
    let mut properties = HashMap::new();
    for property in properties_list {
        // just support identifier(property name) now, ignore property type
        let prop_name = if let Node::Identifier { value } = property {
            value
        } else {
            return Err(format!(
                "Expected identifier, but got {}",
                property.string()
            ));
        };
        properties.insert(prop_name.clone(), ObjectType::Any);
    }
    let type_definition = Object::new_custom_type_definition(name.clone(), properties);
    env.borrow_mut().set(
        name.unwrap_or("anonymous".to_string()),
        type_definition.clone(),
    );
    Ok(type_definition)
}

fn eval_new_struct_instance(
    name: &Node,
    init_properties: &HashMap<String, Node>,
    env: Rc<RefCell<Environment>>,
) -> Result<Object, String> {
    let struct_definition = eval(name, Rc::clone(&env))?;
    if let ObjectValue::CustomTypeDefinition { name, properties } = struct_definition.value() {
        let missing_props = properties
            .keys()
            .filter(|k| !init_properties.contains_key(*k))
            .cloned()
            .collect::<Vec<String>>();

        if !missing_props.is_empty() {
            return Err(format!(
                "Missing properties: \"{}\" for struct {}",
                missing_props.join(","),
                name.clone().unwrap_or("anonymous".to_string())
            ));
        }

        let unknown_props = init_properties
            .keys()
            .filter(|k| !properties.contains_key(*k))
            .cloned()
            .collect::<Vec<String>>();

        if !unknown_props.is_empty() {
            return Err(format!(
                "Unknown properties: \"{}\" for struct {}",
                unknown_props.join(","),
                name.clone().unwrap_or("anonymous".to_string())
            ));
        }

        let mut env_map = HashMap::new();
        for (prop_name, prop_node) in init_properties {
            let prop_obj = eval(prop_node, Rc::clone(&env))?;
            // todo check type compatibility: properties.get(name)
            env_map.insert(prop_name.clone(), prop_obj);
        }
        Ok(Object::new_struct(env_map))
    } else {
        Err(format!("Can not find type {}", name.string()))
    }
}

fn apply_function(function: Object, args: Vec<Object>) -> Result<Object, String> {
    match function.value_owned() {
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
            let mut env = Environment::new_enclosed(&env);
            let mut arg_iter = args.into_iter();
            for param in parameters.into_iter() {
                if let Node::Identifier { value } = param {
                    env.set(
                        value,
                        arg_iter.next().ok_or(String::from("miss parameter"))?,
                    );
                } else {
                    return Err(format!("Not a identifier: {}", param.string()));
                }
            }
            let return_val = eval(&body, env.to_ref())?;
            if let ObjectValue::ReturnValue(value) = return_val.value() {
                Ok(*value.clone())
            } else {
                Ok(return_val)
            }
        }
        ObjectValue::BuiltinFunction(function) => function.call(&args),
        _ => Err(String::from("Not a function")),
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
        let env = Environment::new_default();
        let evaluated = eval(&program, env.to_ref());

        assert!(evaluated.is_ok());
        assert_eq!(evaluated.unwrap(), Object::new_integer(121));
    }
}
