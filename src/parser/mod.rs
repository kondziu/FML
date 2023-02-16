use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum AST {
    Integer(i32),
    Boolean(bool),
    Null,

    Variable { name: Identifier, value: Box<AST> },
    Array { size: Box<AST>, value: Box<AST> },
    Object { extends: Box<AST>, members: Vec<AST> },

    AccessVariable { name: Identifier },
    AccessField { object: Box<AST>, field: Identifier },
    AccessArray { array: Box<AST>, index: Box<AST> },

    AssignVariable { name: Identifier, value: Box<AST> },
    AssignField { object: Box<AST>, field: Identifier, value: Box<AST> },
    AssignArray { array: Box<AST>, index: Box<AST>, value: Box<AST> },

    Function { name: Identifier, parameters: Vec<Identifier>, body: Box<AST> },
    //Operator { operator: Operator, parameters: Vec<Identifier>, body: Box<AST> },    // TODO Consider merging with function
    CallFunction { name: Identifier, arguments: Vec<AST> },
    CallMethod { object: Box<AST>, name: Identifier, arguments: Vec<AST> },
    //CallOperator { object: Box<AST>, operator: Operator, arguments: Vec<Box<AST>> }, // TODO Consider removing
    //Operation { operator: Operator, left: Box<AST>, right: Box<AST> },               // TODO Consider removing
    Top(Vec<AST>),
    Block(Vec<AST>),
    Loop { condition: Box<AST>, body: Box<AST> },
    Conditional { condition: Box<AST>, consequent: Box<AST>, alternative: Box<AST> },

    Print { format: String, arguments: Vec<AST> },
}

impl AST {
    pub fn integer(i: i32) -> Self {
        Self::Integer(i)
    }
    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn variable(name: Identifier, value: AST) -> Self {
        Self::Variable { name, value: Box::new(value) }
    }

    pub fn array(size: AST, value: AST) -> Self {
        Self::Array { size: Box::new(size), value: Box::new(value) }
    }

    pub fn object(extends: AST, members: Vec<AST>) -> Self {
        Self::Object { extends: Box::new(extends), members }
    }

    pub fn access_variable(name: Identifier) -> Self {
        Self::AccessVariable { name }
    }

    pub fn access_field(object: AST, field: Identifier) -> Self {
        Self::AccessField { object: Box::new(object), field }
    }

    pub fn access_array(array: AST, index: AST) -> Self {
        Self::AccessArray { array: Box::new(array), index: Box::new(index) }
    }

    pub fn assign_variable(name: Identifier, value: AST) -> Self {
        Self::AssignVariable { name, value: Box::new(value) }
    }

    pub fn assign_field(object: AST, field: Identifier, value: AST) -> Self {
        Self::AssignField { object: Box::new(object), field, value: Box::new(value) }
    }

    pub fn assign_array(array: AST, index: AST, value: AST) -> Self {
        Self::AssignArray { array: Box::new(array), index: Box::new(index), value: Box::new(value) }
    }

    pub fn function(name: Identifier, parameters: Vec<Identifier>, body: AST) -> Self {
        Self::Function { name, parameters, body: Box::new(body) }
    }

    pub fn operator(operator: Operator, parameters: Vec<Identifier>, body: AST) -> Self {
        Self::Function { name: Identifier::from(operator), parameters, body: Box::new(body) }
    }

    pub fn call_function(name: Identifier, arguments: Vec<AST>) -> Self {
        Self::CallFunction { name, arguments }
    }

    pub fn call_method(object: AST, name: Identifier, arguments: Vec<AST>) -> Self {
        Self::CallMethod { object: Box::new(object), name, arguments }
    }

    pub fn call_operator(object: AST, operator: Operator, arguments: Vec<AST>) -> Self {
        Self::CallMethod { object: Box::new(object), name: Identifier::from(operator), arguments }
    }

    pub fn operation(operator: Operator, left: AST, right: AST) -> Self {
        //Self::Operation { operator, left: left, right: right }
        Self::CallMethod {
            object: Box::new(left),
            name: Identifier::from(operator),
            arguments: vec![right],
        }
    }

    pub fn top(statements: Vec<AST>) -> Self {
        Self::Top(statements)
    }

    pub fn block(statements: Vec<AST>) -> Self {
        Self::Block(statements)
    }

    pub fn loop_de_loop(condition: AST, body: AST) -> Self {
        Self::Loop { condition: Box::new(condition), body: Box::new(body) }
    }

    pub fn conditional(condition: AST, consequent: AST, alternative: AST) -> Self {
        Self::Conditional {
            condition: Box::new(condition),
            consequent: Box::new(consequent),
            alternative: Box::new(alternative),
        }
    }

    pub fn print(format: String, arguments: Vec<AST>) -> Self {
        Self::Print { format, arguments }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl From<Operator> for Identifier {
    fn from(op: Operator) -> Self {
        Identifier(op.to_string())
    }
}

impl From<&str> for Identifier {
    fn from(s: &str) -> Self {
        Identifier(s.to_owned())
    }
}

impl From<String> for Identifier {
    fn from(s: String) -> Self {
        Identifier(s)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Identifier {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Operator {
    Multiplication,
    Division,
    Module,
    Addition,
    Subtraction,
    Inequality,
    Equality,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Disjunction,
    Conjunction,
}

impl Operator {
    pub fn as_str(&self) -> &str {
        match self {
            Operator::Multiplication => "*",
            Operator::Division => "/",
            Operator::Module => "%",
            Operator::Addition => "+",
            Operator::Subtraction => "-",
            Operator::Inequality => "!=",
            Operator::Equality => "==",
            Operator::Less => "<",
            Operator::LessEqual => "<=",
            Operator::Greater => ">",
            Operator::GreaterEqual => ">=",
            Operator::Disjunction => "|",
            Operator::Conjunction => "&",
        }
    }
}

impl From<&str> for Operator {
    fn from(s: &str) -> Self {
        match s {
            "*" => Operator::Multiplication,
            "/" => Operator::Division,
            "%" => Operator::Module,
            "+" => Operator::Addition,
            "-" => Operator::Subtraction,
            "!=" => Operator::Inequality,
            "==" => Operator::Equality,
            "<" => Operator::Less,
            "<=" => Operator::LessEqual,
            ">" => Operator::Greater,
            ">=" => Operator::GreaterEqual,
            "|" => Operator::Disjunction,
            "&" => Operator::Conjunction,

            other => panic!("Cannot parse {} as Operator", other),
        }
    }
}

impl From<String> for Operator {
    fn from(s: String) -> Self {
        Operator::from(s.as_str())
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[macro_export]
macro_rules! make_operator_ast {
    ( $head:expr, $tail:expr ) => {
        ($tail).into_iter().fold($head, |left, right| {
            let (operator, value) = right;
            AST::Operation { operator: operator, left: Box::new(left), right: Box::new(value) }
        })
    };
}

impl AST {
    pub fn from_binary_expression(
        first_operand: AST,
        other_operators_and_operands: Vec<(Operator, AST)>,
    ) -> Self {
        other_operators_and_operands
            .into_iter()
            .fold(first_operand, |left, (operator, right)| AST::operation(operator, left, right))
    }
}
