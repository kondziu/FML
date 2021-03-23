use std::fmt::Debug;
use std::cmp::PartialEq;
use serde::{Serialize, Deserialize};

#[derive(PartialEq,Debug,Serialize,Deserialize,Clone)]
pub enum AST {
    Integer(i32),
    Boolean(bool),
    Null,

    Variable { name: Identifier, value: Box<AST> },
    Array { size: Box<AST>, value: Box<AST> },
    Object { extends: Box<AST>, members: Vec<Box<AST>> },

    AccessVariable { name: Identifier },
    AccessField { object: Box<AST>, field: Identifier },
    AccessArray { array: Box<AST>, index: Box<AST> },

    AssignVariable { name: Identifier, value: Box<AST> },
    AssignField { object: Box<AST>, field: Identifier, value: Box<AST> },
    AssignArray { array: Box<AST>, index: Box<AST>, value: Box<AST> },

    Function { name: Identifier, parameters: Vec<Identifier>, body: Box<AST> },
    //Operator { operator: Operator, parameters: Vec<Identifier>, body: Box<AST> },    // TODO Consider merging with function

    CallFunction { name: Identifier, arguments: Vec<Box<AST>> },
    CallMethod { object: Box<AST>, name: Identifier, arguments: Vec<Box<AST>> },
    //CallOperator { object: Box<AST>, operator: Operator, arguments: Vec<Box<AST>> }, // TODO Consider removing
    //Operation { operator: Operator, left: Box<AST>, right: Box<AST> },               // TODO Consider removing

    Top (Vec<Box<AST>>),
    Block (Vec<Box<AST>>),
    Loop { condition: Box<AST>, body: Box<AST> },
    Conditional { condition: Box<AST>, consequent: Box<AST>, alternative: Box<AST> },

    Print { format: String, arguments: Vec<Box<AST>> },
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
        Self::Variable { name, value: value.into_boxed() }
    }

    pub fn array(size: AST, value: AST) -> Self {
        Self::Array { size: size.into_boxed(), value: value.into_boxed() }
    }

    pub fn object(extends: AST, members: Vec<AST>) -> Self {
        Self::Object { extends: extends.into_boxed(), members: members.into_boxed() }
    }

    pub fn access_variable(name: Identifier) -> Self {
        Self::AccessVariable { name }
    }

    pub fn access_field(object: AST, field: Identifier) -> Self {
        Self::AccessField { object: object.into_boxed(), field }
    }

    pub fn access_array(array: AST, index: AST) -> Self {
        Self::AccessArray { array: array.into_boxed(), index: index.into_boxed() }
    }

    pub fn assign_variable(name: Identifier, value: AST) -> Self {
        Self::AssignVariable { name, value: value.into_boxed() }
    }

    pub fn assign_field(object: AST, field: Identifier, value: AST) -> Self {
        Self::AssignField {
            object: object.into_boxed(),
            field,
            value: value.into_boxed()
        }
    }

    pub fn assign_array(array: AST, index: AST, value: AST) -> Self {
        Self::AssignArray {
            array: array.into_boxed(),
            index: index.into_boxed(),
            value: value.into_boxed()
        }
    }

    pub fn function(name: Identifier, parameters: Vec<Identifier>, body: AST) -> Self {
        Self::Function { name, parameters, body: body.into_boxed() }
    }

    pub fn operator(operator: Operator, parameters: Vec<Identifier>, body: AST) -> Self {
        Self::Function { name: Identifier::from(operator), parameters, body: body.into_boxed() }
    }

    pub fn call_function(name: Identifier, arguments: Vec<AST>) -> Self {
        Self::CallFunction { name, arguments: arguments.into_boxed() }
    }

    pub fn call_method(object: AST, name: Identifier, arguments: Vec<AST>) -> Self {
        Self::CallMethod {
            object: object.into_boxed(),
            name,
            arguments: arguments.into_boxed() }
    }

    pub fn call_operator(object: AST, operator: Operator, arguments: Vec<AST>) -> Self {
        Self::CallMethod {
            object: object.into_boxed(),
            name: Identifier::from(operator),
            arguments: arguments.into_boxed()
        }
    }

    pub fn operation(operator: Operator, left: AST, right: AST) -> Self {
        //Self::Operation { operator, left: left.into_boxed(), right: right.into_boxed() }
        Self::CallMethod {
            object: left.into_boxed(),
            name: Identifier::from(operator),
            arguments: vec![right.into_boxed()]
        }
    }

    pub fn top (statements: Vec<AST>) -> Self {
        Self::Top(statements.into_boxed())
    }

    pub fn block(statements: Vec<AST>) -> Self {
        Self::Block(statements.into_boxed())
    }

    pub fn loop_de_loop(condition: AST, body: AST) -> Self {
        Self::Loop { condition: condition.into_boxed(), body: body.into_boxed() }
    }

    pub fn conditional(condition: AST, consequent: AST, alternative: AST) -> Self {
        Self::Conditional {
            condition: condition.into_boxed(),
            consequent: consequent.into_boxed(),
            alternative: alternative.into_boxed()
        }
    }

    pub fn print(format: String, arguments: Vec<AST>) -> Self {
        Self::Print { format, arguments: arguments.into_boxed() }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum ExtAST {
    Integer(i32),
    Boolean(bool),
    Null,

    Variable { name: Identifier, value: Box<ExtAST> },
    Array { size: Box<ExtAST>, value: Box<ExtAST> },
    Object { extends: Box<ExtAST>, members: Vec<Box<ExtAST>> },

    AccessVariable { name: Identifier },
    AccessField { object: Box<ExtAST>, field: Identifier },
    AccessArray { array: Box<ExtAST>, index: Box<ExtAST> },

    AssignVariable { name: Identifier, value: Box<ExtAST> },
    AssignField { object: Box<ExtAST>, field: Identifier, value: Box<ExtAST> },
    AssignArray { array: Box<ExtAST>, index: Box<ExtAST>, value: Box<ExtAST> },

    Function { name: Identifier, parameters: Vec<Identifier>, body: Box<ExtAST> },
    //Operator { operator: Operator, parameters: Vec<Identifier>, body: Box<AST> },    // TODO Consider merging with function

    CallFunction { name: Identifier, arguments: Vec<Box<ExtAST>> },
    CallMethod { object: Box<ExtAST>, name: Identifier, arguments: Vec<Box<ExtAST>> },
    //CallOperator { object: Box<AST>, operator: Operator, arguments: Vec<Box<AST>> }, // TODO Consider removing
    //Operation { operator: Operator, left: Box<AST>, right: Box<AST> },               // TODO Consider removing

    Top (Vec<Box<ExtAST>>),
    Block (Vec<Box<ExtAST>>),
    Loop { condition: Box<ExtAST>, body: Box<ExtAST> },
    Conditional { condition: Box<ExtAST>, consequent: Box<ExtAST>, alternative: Box<ExtAST> },

    Print { format: String, arguments: Vec<Box<ExtAST>> },

    CompilationUnits(Vec<Box<ExtAST>>),
    DetachedMethod { name: Identifier, parameters: Vec<Identifier>, body: Box<ExtAST> },
    MethodStub { compilation_unit: usize },
}

impl ExtAST {
    pub fn compilation_units(units: Vec<ExtAST>) -> Self {
        Self::CompilationUnits(units.into_iter().map(|unit| unit.into_boxed()).collect())
    }
    pub fn integer(i: i32) -> Self {
        Self::Integer(i)
    }
    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn variable(name: Identifier, value: ExtAST) -> Self {
        Self::Variable { name, value: value.into_boxed() }
    }

    pub fn array(size: ExtAST, value: ExtAST) -> Self {
        Self::Array { size: size.into_boxed(), value: value.into_boxed() }
    }

    pub fn object(extends: ExtAST, members: Vec<ExtAST>) -> Self {
        Self::Object { extends: extends.into_boxed(), members: members.into_boxed() }
    }

    pub fn access_variable(name: Identifier) -> Self {
        Self::AccessVariable { name }
    }

    pub fn access_field(object: ExtAST, field: Identifier) -> Self {
        Self::AccessField { object: object.into_boxed(), field }
    }

    pub fn access_array(array: ExtAST, index: ExtAST) -> Self {
        Self::AccessArray { array: array.into_boxed(), index: index.into_boxed() }
    }

    pub fn assign_variable(name: Identifier, value: ExtAST) -> Self {
        Self::AssignVariable { name, value: value.into_boxed() }
    }

    pub fn assign_field(object: ExtAST, field: Identifier, value: ExtAST) -> Self {
        Self::AssignField {
            object: object.into_boxed(),
            field,
            value: value.into_boxed()
        }
    }

    pub fn assign_array(array: ExtAST, index: ExtAST, value: ExtAST) -> Self {
        Self::AssignArray {
            array: array.into_boxed(),
            index: index.into_boxed(),
            value: value.into_boxed()
        }
    }

    pub fn function(name: Identifier, parameters: Vec<Identifier>, body: ExtAST) -> Self {
        Self::Function { name, parameters, body: body.into_boxed() }
    }

    pub fn operator(operator: Operator, parameters: Vec<Identifier>, body: ExtAST) -> Self {
        Self::Function { name: Identifier::from(operator), parameters, body: body.into_boxed() }
    }

    pub fn call_function(name: Identifier, arguments: Vec<ExtAST>) -> Self {
        Self::CallFunction { name, arguments: arguments.into_boxed() }
    }

    pub fn call_method(object: ExtAST, name: Identifier, arguments: Vec<ExtAST>) -> Self {
        Self::CallMethod {
            object: object.into_boxed(),
            name,
            arguments: arguments.into_boxed() }
    }

    pub fn call_operator(object: ExtAST, operator: Operator, arguments: Vec<ExtAST>) -> Self {
        Self::CallMethod {
            object: object.into_boxed(),
            name: Identifier::from(operator),
            arguments: arguments.into_boxed()
        }
    }

    pub fn operation(operator: Operator, left: ExtAST, right: ExtAST) -> Self {
        //Self::Operation { operator, left: left.into_boxed(), right: right.into_boxed() }
        Self::CallMethod {
            object: left.into_boxed(),
            name: Identifier::from(operator),
            arguments: vec![right.into_boxed()]
        }
    }

    pub fn top (statements: Vec<ExtAST>) -> Self {
        Self::Top(statements.into_boxed())
    }

    pub fn block(statements: Vec<ExtAST>) -> Self {
        Self::Block(statements.into_boxed())
    }

    pub fn loop_de_loop(condition: ExtAST, body: ExtAST) -> Self {
        Self::Loop { condition: condition.into_boxed(), body: body.into_boxed() }
    }

    pub fn conditional(condition: ExtAST, consequent: ExtAST, alternative: ExtAST) -> Self {
        Self::Conditional {
            condition: condition.into_boxed(),
            consequent: consequent.into_boxed(),
            alternative: alternative.into_boxed()
        }
    }

    pub fn print(format: String, arguments: Vec<ExtAST>) -> Self {
        Self::Print { format, arguments: arguments.into_boxed() }
    }

    pub fn method_stub(index: usize) -> Self {
        Self::MethodStub { compilation_unit: index }
    }

    pub fn detached_method(name: Identifier, parameters: Vec<Identifier>, body: ExtAST) -> Self {
        Self::DetachedMethod { name, parameters, body: body.into_boxed() }
    }
}

#[derive(PartialEq,Eq,Hash,Debug,Clone,Serialize,Deserialize)]
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
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(PartialEq,Debug,Copy,Clone,Serialize,Deserialize)]
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
            Operator::Division       => "/",
            Operator::Module         => "%",
            Operator::Addition       => "+",
            Operator::Subtraction    => "-",
            Operator::Inequality     => "!=",
            Operator::Equality       => "==",
            Operator::Less           => "<",
            Operator::LessEqual      => "<=",
            Operator::Greater        => ">",
            Operator::GreaterEqual   => ">=",
            Operator::Disjunction    => "|",
            Operator::Conjunction    => "&",
        }
    }
}

impl From<&str> for Operator {
    fn from(s: &str) -> Self {
        match s {
            "*"  => Operator::Multiplication,
            "/"  => Operator::Division,
            "%"  => Operator::Module,
            "+"  => Operator::Addition,
            "-"  => Operator::Subtraction,
            "!=" => Operator::Inequality,
            "==" => Operator::Equality,
            "<"  => Operator::Less,
            "<=" => Operator::LessEqual,
            ">"  => Operator::Greater,
            ">=" => Operator::GreaterEqual,
            "|"  => Operator::Disjunction,
            "&"  => Operator::Conjunction,

            other => panic!(format!("Cannot parse {} as Operator", other)),
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
            AST::Operation {
                operator: operator,
                left: Box::new(left),
                right: Box::new(value)}
        })
    }
}

impl AST {
    pub fn from_binary_expression(first_operand: AST, other_operators_and_operands: Vec<(Operator, AST)>) -> Self {
        other_operators_and_operands.into_iter()
            .fold(first_operand, |left, (operator, right)| {
                AST::operation(operator, left, right)
            })
    }
}

pub trait IntoBoxed {
    type Into;
    fn into_boxed(self) -> Self::Into;
}

impl IntoBoxed for AST {
    type Into = Box<Self>;
    fn into_boxed(self) -> Self::Into {
        Box::new(self)
    }
}

impl IntoBoxed for Vec<AST> {
    type Into = Vec<Box<AST>>;
    fn into_boxed(self) -> Self::Into {
        self.into_iter().map(|ast| ast.into_boxed()).collect()
    }
}

impl IntoBoxed for Option<AST> {
    type Into = Option<Box<AST>>;
    fn into_boxed(self) -> Self::Into {
        self.map(|ast| ast.into_boxed())
    }
}

impl IntoBoxed for ExtAST {
    type Into = Box<Self>;
    fn into_boxed(self) -> Self::Into {
        Box::new(self)
    }
}

impl IntoBoxed for Vec<ExtAST> {
    type Into = Vec<Box<ExtAST>>;
    fn into_boxed(self) -> Self::Into {
        self.into_iter().map(|ast| ast.into_boxed()).collect()
    }
}

impl IntoBoxed for Option<ExtAST> {
    type Into = Option<Box<ExtAST>>;
    fn into_boxed(self) -> Self::Into {
        self.map(|ast| ast.into_boxed())
    }
}