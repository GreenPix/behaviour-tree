use tree::{LeafNode,Context,VisitResult,StoreKind,BehaviourTreeNode};
use standard::{Value,LeafNodeFactory,Operator};
use self::PostfixedExpressionMember::*;

// Postfixed expression notation
// member1 member2 operator to do a conventional member1 operator member2
// A member can itself be an expression
//
// Few examples:
// 1 3 + 3 4 + *    => (1 + 3) * (3 + 4)
// 1 2 3 4 5 6 + * + * + => 1 + (2 * (3 + (4 * (5 + 6))))
#[derive(Clone)]
pub enum PostfixedExpressionMember {
    Op(Operator),
    Constant(i64),
    Variable(String),
}

struct ExpressionEvaluator {
    expression: Vec<PostfixedExpressionMember>,
    variable: String,
}

impl BehaviourTreeNode for ExpressionEvaluator {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        let result = evaluate_expression_int(context, &self.expression);
        let old = context.map.insert(self.variable.clone(),StoreKind::I64(result));
        if let Some(other) = old {
            println!("WARNING: replaced variable {}, which contained {:?} by {}", self.variable, other, result);
        } else {
            println!("Set variable {} to {}", self.variable, result);
        }
        VisitResult::Success
    }
}

pub type PostfixedExpression = Vec<PostfixedExpressionMember>;

pub fn evaluate_int_node(options: &Option<Value>) -> Result<LeafNodeFactory, String> {
    let options_map = match options {
        &Some(Value::Map(ref map)) => map,
        other => return Err(format!("Expected hashmap, found {:?}", other)),
    };
    if options_map.len() != 2 {
        return Err(format!("Expected options with 2 key / value pairs, found {}", options_map.len()));
    }
    let expression = match options_map.get("expression") {
        Some(&Value::Array(ref array)) => try!(generate_postfixed_expression(array)),
        other => return Err(format!("Expected expression array, found {:?}", other)),
    };
    let variable = match options_map.get("result") {
        Some(&Value::String(ref key)) => key.clone(),
        other => return Err(format!("Expected variable name, found {:?}", other)),
    };
    Ok(Box::new(move || LeafNode::new(Box::new(ExpressionEvaluator {
        variable: variable.clone(),
        expression: expression.clone(),
    }))))
}

pub fn generate_postfixed_expression(array: &[Value]) -> Result<Vec<PostfixedExpressionMember>,String> {
    let mut res = Vec::new();
    for operand in array.iter() {
        match *operand {
            Value::String(ref op) => {
                res.push(Variable(op.clone()))
            }
            Value::Integer(value) => res.push(Constant(value)),
            Value::Operator(op) => {
                res.push(Op(op));
            }
            ref other => return Err(format!("Expected operand, found {:?}", other)),
        }
    }
    Ok(res)
}

pub fn evaluate_expression_int(context: &Context, expression: &[PostfixedExpressionMember]) -> i64 {
    let mut stack = Vec::new();
    for member in expression.iter() {
        match *member {
            Constant(value) => stack.push(value),
            Variable(ref name) => {
                let value = match context.map.get::<str>(name.as_ref()) {
                    Some(&StoreKind::I64(value)) => value,
                    other => panic!("Expected I64 as variable value, found {:?}", other),
                };
                stack.push(value);
            },
            Op(operator) => {
                // First member will be the second one in the stack
                let member2 = stack.pop().expect("Expected first expression member");
                let member1 = stack.pop().expect("Expected second expression member");
                let result = match operator {
                    Operator::Plus => member1 + member2,
                    Operator::Minus => member1 - member2,
                    Operator::Multiply => member1 * member2,
                    Operator::Divide => member1 / member2,
                };
                stack.push(result);
            }
        }
    }
    let result = stack.pop().expect("Unexpected absence of result!");
    assert!(stack.is_empty());
    result
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use tree::{Context,StoreKind};
    use standard::Operator;
    use super::PostfixedExpressionMember::*;
    #[test]
    fn evaluate_int() {
        let context = Context::new(HashMap::new());
        let expression = vec! [
            Constant(1),
            Constant(2),
            Op(Operator::Plus),
            ];
        assert!(super::evaluate_expression_int(&context,&expression) == 3);
    }

    #[test]
    #[should_panic]
    fn incorrect_expression() {
        let context = Context::new(HashMap::new());
        let expression = vec! [
            Constant(1),
            Constant(2),
            Op(Operator::Plus),
            Op(Operator::Multiply),
            ];
        super::evaluate_expression_int(&context,&expression);
    }

    #[test]
    fn evaluate_int_variable() {
        let mut hashmap = HashMap::new();
        hashmap.insert("forty_two".to_string(), StoreKind::I64(42));
        hashmap.insert("two".to_string(), StoreKind::I64(2));
        let context = Context::new(hashmap);
        // Calculates 2 * (forty_two / two) - 3
        let expression = vec! [
            Constant(2),
            Variable("forty_two".to_string()),
            Variable("two".to_string()),
            Op(Operator::Divide),
            Op(Operator::Multiply),
            Constant(3),
            Op(Operator::Minus),
            ];
        assert!(super::evaluate_expression_int(&context,&expression) == 39);
    }
}
