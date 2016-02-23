use tree::{LeafNode,Context,VisitResult,BehaviourTreeNode};
use standard::{Value,LeafNodeFactory};
use standard::expressions::{self,PostfixedExpression};

#[derive(Debug,Clone,Copy)]
enum CondOp {
    SuperiorStrict,
    InferiorStrict,
    Equal,
    Superior,
    Inferior,
}

struct ConditionChecker {
    exp1: PostfixedExpression,
    exp2: PostfixedExpression,
    operator: CondOp,
}

impl BehaviourTreeNode for ConditionChecker {
    fn visit(&mut self, context: &mut Context) -> VisitResult {
        let result_1 = expressions::evaluate_expression_int(context, &self.exp1);
        let result_2 = expressions::evaluate_expression_int(context, &self.exp2);
        if check_condition(result_1, result_2, self.operator) {
            VisitResult::Success
        } else {
            VisitResult::Failure
        }
    }
}

fn check_condition(exp1: i64, exp2: i64, operator: CondOp) -> bool {
    match operator {
        CondOp::SuperiorStrict => exp1 > exp2,
        CondOp::InferiorStrict => exp1 < exp2,
        CondOp::Equal => exp1 == exp2,
        CondOp::Superior => exp1 >= exp2,
        CondOp::Inferior => exp1 <= exp2,
    }
}

pub fn check_condition_node(options: &Option<Value>) -> Result<LeafNodeFactory, String> {
    let options_map = match options {
        &Some(Value::Map(ref map)) => map,
        other => return Err(format!("Expected hashmap, found {:?}", other)),
    };
    let exp1 = match options_map.get("exp1") {
        None => return Err("Expected value for key exp1".to_string()),
        Some(&Value::Array(ref array)) => try!(expressions::generate_postfixed_expression(array)),
        Some(other) => return Err(format!("Expected array of operands, found {:?}", other)),
    };
    let exp2 = match options_map.get("exp2") {
        None => return Err("Expected value for key exp2".to_string()),
        Some(&Value::Array(ref array)) => try!(expressions::generate_postfixed_expression(array)),
        Some(other) => return Err(format!("Expected array of operands, found {:?}", other)),
    };
    let operator = match options_map.get("operator") {
        None => return Err("Expected value for key operator".to_string()),
        Some(&Value::Unknown(op)) => {
            match op {
                '>' => CondOp::SuperiorStrict,
                '<' => CondOp::InferiorStrict,
                '=' => CondOp::Equal,
                other => return Err(format!("Expected operator, found {}", other)),
            }
        }
        Some(&Value::String(ref op)) => {
            match op.as_ref() {
                ">" => CondOp::SuperiorStrict,
                "<" => CondOp::InferiorStrict,
                "=" => CondOp::Equal,
                ">=" => CondOp::Superior,
                "<=" => CondOp::Inferior,
                other => return Err(format!("Expected operator, found {}", other)),
            }
        }
        Some(other) => return Err(format!("Expected operator, found {:?}", other)),
    };
    Ok(Box::new(move || LeafNode::new(Box::new(ConditionChecker {
        exp1: exp1.clone(),
        exp2: exp2.clone(),
        operator: operator.clone(),
    }))))
}
