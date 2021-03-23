use crate::parser::*;

use crate::bytecode::compiler::*;

#[test] fn no_functions_on_top () {
    let ast = AST::top(vec![
        AST::assign_variable(Identifier::from("x"), AST::integer(42)),
        AST::access_variable(Identifier::from("x")),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::top(vec![
            ExtAST::assign_variable(Identifier::from("x"), ExtAST::integer(42)),
            ExtAST::access_variable(Identifier::from("x")),
        ])
    ]);

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

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("identity"), vec![Identifier::from("x")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("x"))
        ])),
        ExtAST::top(vec![
            ExtAST::assign_variable(Identifier::from("x"), ExtAST::integer(42)),
            ExtAST::call_function(Identifier::from("identity"), vec![
                ExtAST::access_variable(Identifier::from("x"))
            ]),
        ])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn just_one_function_on_top () {
    let ast = AST::top(vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("identity"), vec![Identifier::from("x")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("x"))
        ])),
        ExtAST::top(vec![])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn three_functions_on_top () {
    let ast = AST::top(vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::call_function(Identifier::from("identity"), vec![
            AST::integer(42)
        ]),
        AST::function(Identifier::from("left"), vec![Identifier::from("x"), Identifier::from("y")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::call_function(Identifier::from("left"), vec![
            AST::integer(1),
            AST::integer(2),
        ]),
        AST::function(Identifier::from("right"), vec![Identifier::from("x"), Identifier::from("y")], AST::block(vec![
            AST::access_variable(Identifier::from("y"))
        ])),
        AST::call_function(Identifier::from("right"), vec![
            AST::integer(1),
            AST::integer(2),
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("identity"), vec![Identifier::from("x")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("x"))
        ])),
        ExtAST::function(Identifier::from("left"), vec![Identifier::from("x"), Identifier::from("y")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("x"))
        ])),
        ExtAST::function(Identifier::from("right"), vec![Identifier::from("x"), Identifier::from("y")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("y"))
        ])),
        ExtAST::top(vec![
            ExtAST::call_function(Identifier::from("identity"), vec![
                ExtAST::integer(42)
            ]),
            ExtAST::call_function(Identifier::from("left"), vec![
                ExtAST::integer(1),
                ExtAST::integer(2),
            ]),
            ExtAST::call_function(Identifier::from("right"), vec![
                ExtAST::integer(1),
                ExtAST::integer(2),
            ]),
        ])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn method_in_object_in_top () {
    let ast = AST::top(vec![
        AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::block(vec![
                AST::access_variable(Identifier::from("this"))
            ]))
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("self"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::top(vec![ExtAST::object(ExtAST::null(), vec![
            ExtAST::function(Identifier::from("self"), vec![], ExtAST::integer(0))
        ])])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn three_methods_in_object_in_top () {
    let ast = AST::top(vec![
        AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::block(vec![
                AST::access_variable(Identifier::from("this"))
            ])),
            AST::function(Identifier::from("zero"), vec![], AST::integer(0)),
            AST::function(Identifier::from("none"), vec![], AST::Null),
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("self"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::function(Identifier::from("zero"), vec![Identifier::from("this")], ExtAST::integer(0)),
        ExtAST::function(Identifier::from("none"), vec![Identifier::from("this")], ExtAST::Null),
        ExtAST::top(vec![ExtAST::object(ExtAST::null(), vec![
            ExtAST::function(Identifier::from("self"), vec![], ExtAST::integer(0)),
            ExtAST::function(Identifier::from("zero"), vec![], ExtAST::integer(1)),
            ExtAST::function(Identifier::from("none"), vec![], ExtAST::integer(2)),
        ])])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn three_methods_in_object_amidst_fields_in_top () {
    let ast = AST::top(vec![
        AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::block(vec![
                AST::access_variable(Identifier::from("this"))
            ])),
            AST::variable(Identifier::from("x"), AST::Null),
            AST::function(Identifier::from("zero"), vec![], AST::integer(0)),
            AST::variable(Identifier::from("y"), AST::Null),
            AST::function(Identifier::from("none"), vec![], AST::Null),
            AST::variable(Identifier::from("z"), AST::Null),
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("self"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::function(Identifier::from("zero"), vec![Identifier::from("this")], ExtAST::integer(0)),
        ExtAST::function(Identifier::from("none"), vec![Identifier::from("this")], ExtAST::Null),
        ExtAST::top(vec![ExtAST::object(ExtAST::null(), vec![
            ExtAST::function(Identifier::from("self"), vec![], ExtAST::integer(0)),
            ExtAST::variable(Identifier::from("x"), ExtAST::Null),
            ExtAST::function(Identifier::from("zero"), vec![], ExtAST::integer(1)),
            ExtAST::variable(Identifier::from("y"), ExtAST::Null),
            ExtAST::function(Identifier::from("none"), vec![], ExtAST::integer(2)),
            ExtAST::variable(Identifier::from("z"), ExtAST::Null),
        ])])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}

#[test] fn methods_in_objects_in_methods_in_objects_in_top() {
    let ast = AST::top(vec![
        AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::block(vec![
                AST::access_variable(Identifier::from("this"))
            ])),
            AST::variable(Identifier::from("x"), AST::Null),
            AST::function(Identifier::from("new"), vec![], AST::object(AST::null(), vec![
                AST::function(Identifier::from("me"), vec![], AST::block(vec![
                    AST::access_variable(Identifier::from("this"))
                ])),
            ])),
            AST::variable(Identifier::from("y"), AST::object(AST::null(), vec![
                AST::function(Identifier::from("y"), vec![], AST::block(vec![
                    AST::access_variable(Identifier::from("this"))
                ])),
            ])),
            AST::function(Identifier::from("none"), vec![], AST::Null),
            AST::variable(Identifier::from("z"), AST::Null),
        ]),
    ]);

    let compilation_units = ast.split_into_compilation_units();

    let expected_compilation_units = ExtAST::compilation_units(vec![
        ExtAST::function(Identifier::from("self"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::function(Identifier::from("me"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::function(Identifier::from("new"), vec![Identifier::from("this")], ExtAST::object(ExtAST::null(), vec![
            ExtAST::function(Identifier::from("me"), vec![], ExtAST::integer(1)),
        ])),
        ExtAST::function(Identifier::from("y"), vec![Identifier::from("this")], ExtAST::block(vec![
            ExtAST::access_variable(Identifier::from("this"))
        ])),
        ExtAST::function(Identifier::from("none"), vec![Identifier::from("this")], ExtAST::Null),
        ExtAST::top(vec![
            ExtAST::object(ExtAST::null(), vec![
                ExtAST::function(Identifier::from("self"), vec![], ExtAST::Integer(0)),
                ExtAST::variable(Identifier::from("x"), ExtAST::Null),
                ExtAST::function(Identifier::from("new"), vec![], ExtAST::integer(2)),
                ExtAST::variable(Identifier::from("y"), ExtAST::object(ExtAST::null(), vec![
                    ExtAST::function(Identifier::from("y"), vec![], ExtAST::integer(3))
                ])),
                ExtAST::function(Identifier::from("none"), vec![], ExtAST::integer(4)),
                ExtAST::variable(Identifier::from("z"), ExtAST::Null),
            ]),
        ])
    ]);

    assert_eq!(compilation_units, expected_compilation_units);
}