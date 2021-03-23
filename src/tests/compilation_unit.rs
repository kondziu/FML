use crate::parser::*;

use crate::bytecode::compiler::*;

#[test] fn no_functions_on_top () {
    let ast = AST::top(vec![
        AST::assign_variable(Identifier::from("x"), AST::integer(42)),
        AST::access_variable(Identifier::from("x")),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = vec![
        AST::top(vec![
            AST::assign_variable(Identifier::from("x"), AST::integer(42)),
            AST::access_variable(Identifier::from("x")),
        ])
    ];

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn one_function_on_top () {
    let ast = AST::top(vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::assign_variable(Identifier::from("x"), AST::integer(42)),
        AST::call_function(Identifier::from("identity"), vec![
            AST::access_variable(Identifier::from("x"))
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::top(vec![
            AST::assign_variable(Identifier::from("x"), AST::integer(42)),
            AST::call_function(Identifier::from("identity"), vec![
                AST::access_variable(Identifier::from("x"))
            ]),
        ])
    ];

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn just_one_function_on_top () {
    let ast = AST::top(vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::top(vec![])
    ];

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn just_one_function_on_top () {
    let ast = AST::top(vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::top(vec![])
    ];

    assert_eq!(compilation_units, expected_compilation_units);
}