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

    let expected_compilation_units = vec![
        AST::function(Identifier::from("identity"), vec![Identifier::from("x")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::function(Identifier::from("left"), vec![Identifier::from("x"), Identifier::from("y")], AST::block(vec![
            AST::access_variable(Identifier::from("x"))
        ])),
        AST::function(Identifier::from("right"), vec![Identifier::from("x"), Identifier::from("y")], AST::block(vec![
            AST::access_variable(Identifier::from("y"))
        ])),
        AST::top(vec![
            AST::call_function(Identifier::from("identity"), vec![
                AST::integer(42)
            ]),
            AST::call_function(Identifier::from("left"), vec![
                AST::integer(1),
                AST::integer(2),
            ]),
            AST::call_function(Identifier::from("right"), vec![
                AST::integer(1),
                AST::integer(2),
            ]),
        ])
    ];

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

    let expected_compilation_units = vec![
        AST::function(Identifier::from("self"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::top(vec![AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::integer(0))
        ])])
    ];

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

    let expected_compilation_units = vec![
        AST::function(Identifier::from("self"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::function(Identifier::from("zero"), vec![Identifier::from("this")], AST::integer(0)),
        AST::function(Identifier::from("none"), vec![Identifier::from("this")], AST::Null),
        AST::top(vec![AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::integer(0)),
            AST::function(Identifier::from("zero"), vec![], AST::integer(1)),
            AST::function(Identifier::from("none"), vec![], AST::integer(2)),
        ])])
    ];

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

    let expected_compilation_units = vec![
        AST::function(Identifier::from("self"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::function(Identifier::from("zero"), vec![Identifier::from("this")], AST::integer(0)),
        AST::function(Identifier::from("none"), vec![Identifier::from("this")], AST::Null),
        AST::top(vec![AST::object(AST::null(), vec![
            AST::function(Identifier::from("self"), vec![], AST::integer(0)),
            AST::variable(Identifier::from("x"), AST::Null),
            AST::function(Identifier::from("zero"), vec![], AST::integer(1)),
            AST::variable(Identifier::from("y"), AST::Null),
            AST::function(Identifier::from("none"), vec![], AST::integer(2)),
            AST::variable(Identifier::from("z"), AST::Null),
        ])])
    ];

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

    let expected_compilation_units = vec![
        AST::function(Identifier::from("self"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::function(Identifier::from("me"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::function(Identifier::from("new"), vec![Identifier::from("this")], AST::object(AST::null(), vec![
            AST::function(Identifier::from("me"), vec![], AST::integer(1)),
        ])),
        AST::function(Identifier::from("y"), vec![Identifier::from("this")], AST::block(vec![
            AST::access_variable(Identifier::from("this"))
        ])),
        AST::function(Identifier::from("none"), vec![Identifier::from("this")], AST::Null),
        AST::top(vec![
            AST::object(AST::null(), vec![
                AST::function(Identifier::from("self"), vec![], AST::Integer(0)),
                AST::variable(Identifier::from("x"), AST::Null),
                AST::function(Identifier::from("new"), vec![], AST::integer(2)),
                AST::variable(Identifier::from("y"), AST::object(AST::null(), vec![
                    AST::function(Identifier::from("y"), vec![], AST::integer(3))
                ])),
                AST::function(Identifier::from("none"), vec![], AST::integer(4)),
                AST::variable(Identifier::from("z"), AST::Null),
            ]),
        ])
    ];

    assert_eq!(compilation_units, expected_compilation_units);
}