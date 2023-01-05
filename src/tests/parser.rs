use crate::fml::TopLevelParser;
use crate::parser::*;

#[allow(dead_code)]
pub fn parse(input: &str) -> Result<crate::parser::AST, String> {
    match crate::fml::TopLevelParser::new().parse(input) {
        Err(e) => Err(format!("{:?}", e)),
        Ok(ast) => Ok(ast),
    }
}

#[allow(dead_code)]
fn parse_ok(input: &str, correct: AST) {
    println!("{}", input);
    for i in 0..input.len() {
        if i % 10 == 0 {
            print!(" ");
        } else {
            print!("{}", i % 10);
        }
    }
    println!();
    assert_eq!(TopLevelParser::new().parse(input), Ok(AST::Top(vec!(Box::new(correct)))));
}

#[allow(dead_code)]
fn parse_err(input: &str) {
    println!("{}", input);
    assert!(TopLevelParser::new().parse(input).is_err());
}

#[test]
fn test_unit() {
    parse_ok("null", AST::null());
}
#[test]
fn test_nothing() {
    parse_ok("", AST::null());
}

#[test]
fn test_0() {
    parse_ok("0", AST::integer(0));
}
#[test]
fn test_negative_0() {
    parse_ok("-0", AST::integer(0));
}
#[test]
fn test_2() {
    parse_ok("2", AST::integer(2));
}
#[test]
fn test_negative_2() {
    parse_ok("-2", AST::integer(-2));
}
#[test]
fn test_42() {
    parse_ok("42", AST::integer(42));
}
#[test]
fn test_042() {
    parse_ok("042", AST::integer(42));
}
#[test]
fn test_00() {
    parse_ok("00", AST::integer(0));
}
#[test]
fn test_negative_042() {
    parse_ok("-042", AST::integer(-42));
}
#[test]
fn test_negative_00() {
    parse_ok("-00", AST::integer(0));
}

#[test]
fn test_underscore() {
    parse_ok("_", AST::access_variable(Identifier::from("_")));
}
#[test]
fn test_underscore_identifier() {
    parse_ok("_x", AST::access_variable(Identifier::from("_x")));
}
#[test]
fn test_identifier() {
    parse_ok("x", AST::access_variable(Identifier::from("x")));
}
#[test]
fn test_identifier_with_number() {
    parse_ok("x1", AST::access_variable(Identifier::from("x1")));
}
#[test]
fn test_multiple_underscores() {
    parse_ok("___", AST::access_variable(Identifier::from("___")));
}
#[test]
fn test_long_identifier() {
    parse_ok("stuff", AST::access_variable(Identifier::from("stuff")));
}

#[test]
fn test_true() {
    parse_ok("true", AST::boolean(true));
}
#[test]
fn test_false() {
    parse_ok("false", AST::boolean(false));
}

#[test]
fn test_number_in_parens() {
    parse_ok("(1)", AST::integer(1));
}
#[test]
fn test_number_in_two_parens() {
    parse_ok("((1))", AST::integer(1));
}
#[test]
fn test_number_parens_with_whitespace() {
    parse_ok("( 1 )", AST::integer(1));
}

#[test]
fn test_local_definition() {
    parse_ok("let x = 1", AST::variable(Identifier::from("x"), AST::integer(1)));
}

#[test]
fn test_mutation() {
    parse_ok("x <- 1", AST::assign_variable(Identifier::from("x"), AST::integer(1)));
}

#[test]
fn test_function_no_args() {
    parse_ok("function f () -> 1", AST::function(Identifier::from("f"), vec![], AST::integer(1)));
}

#[test]
fn test_function_one_arg() {
    parse_ok(
        "function f (x) -> x",
        AST::function(
            Identifier::from("f"),
            vec![Identifier::from("x")],
            AST::access_variable(Identifier::from("x")),
        ),
    );
}

#[test]
fn test_function_many_args() {
    parse_ok(
        "function f (x, y, z) -> x",
        AST::function(
            Identifier::from("f"),
            vec![Identifier::from("x"), Identifier::from("y"), Identifier::from("z")],
            AST::access_variable(Identifier::from("x")),
        ),
    );
}

#[test]
fn test_application_no_args() {
    parse_ok("f ()", AST::call_function(Identifier::from("f"), vec![]));
}

#[test]
fn test_application_one_arg() {
    parse_ok("f (0)", AST::call_function(Identifier::from("f"), vec![AST::integer(0)]));
}

#[test]
fn test_application_more_args() {
    parse_ok(
        "f (1, x, true)",
        AST::call_function(
            Identifier::from("f"),
            vec![AST::integer(1), AST::access_variable(Identifier::from("x")), AST::boolean(true)],
        ),
    );
}

#[test]
fn test_application_no_spaces() {
    parse_ok(
        "f(0,-1)",
        AST::call_function(Identifier::from("f"), vec![AST::integer(0), AST::integer(-1)]),
    );
}

#[test]
fn test_application_more_spaces() {
    parse_ok(
        "f    (   0    , -1 )",
        AST::call_function(Identifier::from("f"), vec![AST::integer(0), AST::integer(-1)]),
    );
}

#[test]
fn test_application_extra_comma() {
    parse_ok(
        "f(0,-1,)",
        AST::call_function(Identifier::from("f"), vec![AST::integer(0), AST::integer(-1)]),
    );
}

#[test]
fn test_application_just_a_comma() {
    parse_err("f(,)");
}
#[test]
fn test_application_many_extra_commas() {
    parse_err("f(x,,)");
}

#[test]
fn test_empty_block_is_unit() {
    parse_ok("begin end", AST::null());
}
#[test]
fn test_block_one_expression() {
    parse_ok("begin 1 end", AST::block(vec![AST::integer(1)]))
}

#[test]
fn test_block_one_expression_and_semicolon() {
    parse_ok("begin 1; end", AST::block(vec![AST::integer(1)]))
}
#[test]
fn test_block_many_expressions() {
    parse_ok(
        "begin 1; 2; 3 end",
        AST::block(vec![AST::integer(1), AST::integer(2), AST::integer(3)]),
    )
}

#[test]
fn test_nested_block() {
    parse_ok(
        "begin 0; begin 1; 2; 3 end; 4; 5 end",
        AST::block(vec![
            AST::integer(0),
            AST::block(vec![AST::integer(1), AST::integer(2), AST::integer(3)]),
            AST::integer(4),
            AST::integer(5),
        ]),
    )
}

#[test]
fn test_nested_block_two() {
    parse_ok(
        "begin \n\
                 0; \n\
                 begin \n\
                     1; \n\
                     2; \n\
                     3 \n\
                  end; \n\
                  4; \n\
                  5 \n\
                  end\n",
        AST::block(vec![
            AST::integer(0),
            AST::block(vec![AST::integer(1), AST::integer(2), AST::integer(3)]),
            AST::integer(4),
            AST::integer(5),
        ]),
    )
}

#[test]
fn test_block_trailing_semicolon() {
    parse_ok(
        "begin 1; 2; 3; end",
        AST::block(vec![AST::integer(1), AST::integer(2), AST::integer(3)]),
    )
}

#[test]
fn test_loop() {
    parse_ok("while true do null", AST::loop_de_loop(AST::boolean(true), AST::null()))
}

#[test]
fn test_conditional() {
    parse_ok(
        "if true then false else true",
        AST::conditional(AST::boolean(true), AST::boolean(false), AST::boolean(true)),
    )
}

#[test]
fn test_conditional_no_alternative() {
    parse_ok(
        "if true then false",
        AST::conditional(AST::boolean(true), AST::boolean(false), AST::null()),
    )
}

#[test]
fn test_conditional_so_many() {
    parse_ok(
        "if x then \
                    if y then 3 else 2 \
                else \
                    if y then 1 else 0",
        AST::conditional(
            AST::access_variable(Identifier::from("x")),
            AST::conditional(
                AST::access_variable(Identifier::from("y")),
                AST::integer(3),
                AST::integer(2),
            ),
            AST::conditional(
                AST::access_variable(Identifier::from("y")),
                AST::integer(1),
                AST::integer(0),
            ),
        ),
    )
}

#[test]
fn test_array_definition() {
    parse_ok("array(10,0)", AST::array(AST::integer(10), AST::integer(0)))
}

#[test]
fn test_array_definition_spaces() {
    parse_ok("array ( 10, 0 )", AST::array(AST::integer(10), AST::integer(0)))
}

#[test]
fn test_empty_object() {
    parse_ok("object begin end", AST::object(AST::null(), vec![]))
}

#[test]
fn test_empty_object_with_superobject() {
    parse_ok(
        "object extends y begin end",
        AST::object(AST::access_variable(Identifier::from("y")), vec![]),
    )
}

#[test]
fn test_object_extending_expression() {
    parse_ok(
        "object extends if y then 1 else true begin end",
        AST::object(
            AST::conditional(
                AST::access_variable(Identifier::from("y")),
                AST::integer(1),
                AST::boolean(true),
            ),
            vec![],
        ),
    )
}

#[test]
fn test_object_extending_ad_hoc_object() {
    parse_ok(
        "object extends object begin end begin end",
        AST::object(AST::object(AST::null(), vec![]), vec![]),
    )
}

#[test]
fn test_object_with_one_field() {
    parse_ok(
        "object begin let y = x; end",
        AST::object(
            AST::null(),
            vec![AST::variable(Identifier::from("y"), AST::access_variable(Identifier::from("x")))],
        ),
    )
}

#[test]
fn test_object_with_one_method() {
    parse_ok(
        "object begin function m (x, y, z) -> y; end",
        AST::object(
            AST::null(),
            vec![AST::function(
                Identifier::from("m"),
                vec![Identifier::from("x"), Identifier::from("y"), Identifier::from("z")],
                AST::access_variable(Identifier::from("y")),
            )],
        ),
    )
}

#[test]
fn test_object_with_an_operator() {
    parse_ok(
        "object begin function + (y) -> y; end",
        AST::object(
            AST::null(),
            vec![AST::operator(
                Operator::Addition,
                vec![Identifier::from("y")],
                AST::access_variable(Identifier::from("y")),
            )],
        ),
    )
}

#[test]
fn test_object_with_many_members() {
    parse_ok(
        "object begin \
                let a = x; \
                let b = true; \
                function m (x, y, z) -> y; \
                function id (x) -> x; \
                function me () -> this; \
            end",
        AST::object(
            AST::null(),
            vec![
                AST::variable(Identifier::from("a"), AST::access_variable(Identifier::from("x"))),
                AST::variable(Identifier::from("b"), AST::boolean(true)),
                AST::function(
                    Identifier::from("m"),
                    vec![Identifier::from("x"), Identifier::from("y"), Identifier::from("z")],
                    AST::access_variable(Identifier::from("y")),
                ),
                AST::function(
                    Identifier::from("id"),
                    vec![Identifier::from("x")],
                    AST::access_variable(Identifier::from("x")),
                ),
                AST::function(
                    Identifier::from("me"),
                    vec![],
                    AST::access_variable(Identifier::from("this")),
                ),
            ],
        ),
    );
}

#[test]
fn test_field_access_from_identifier() {
    parse_ok(
        "a.b",
        AST::access_field(AST::access_variable(Identifier::from("a")), Identifier::from("b")),
    );
}

#[test]
fn test_field_access_from_number() {
    parse_ok("1.b", AST::access_field(AST::integer(1), Identifier::from("b")));
}

#[test]
fn test_field_access_from_boolean() {
    parse_ok("true.b", AST::access_field(AST::boolean(true), Identifier::from("b")));
}

#[test]
fn test_field_access_from_parenthesized_expression() {
    parse_ok(
        "(if x then 1 else 2).b",
        AST::access_field(
            AST::conditional(
                AST::access_variable(Identifier::from("x")),
                AST::integer(1),
                AST::integer(2),
            ),
            Identifier::from("b"),
        ),
    );
}

#[test]
fn test_field_chain_access() {
    parse_ok(
        "a.b.c.d",
        AST::access_field(
            AST::access_field(
                AST::access_field(
                    AST::access_variable(Identifier::from("a")),
                    Identifier::from("b"),
                ),
                Identifier::from("c"),
            ),
            Identifier::from("d"),
        ),
    );
}

#[test]
fn test_field_mutation_from_identifier() {
    parse_ok(
        "a.b <- 1",
        AST::assign_field(
            AST::access_variable(Identifier::from("a")),
            Identifier::from("b"),
            AST::integer(1),
        ),
    );
}

#[test]
fn test_method_call_from_identifier() {
    parse_ok(
        "a.b (1)",
        AST::call_method(
            AST::access_variable(Identifier::from("a")),
            Identifier::from("b"),
            vec![AST::integer(1)],
        ),
    );
}

#[test]
fn test_method_call_to_operator() {
    parse_ok(
        "a.+(1)",
        AST::call_operator(
            AST::access_variable(Identifier::from("a")),
            Operator::Addition,
            vec![AST::integer(1)],
        ),
    );
}

#[test]
fn test_array_access() {
    parse_ok(
        "a[1]",
        AST::access_array(AST::access_variable(Identifier::from("a")), AST::integer(1)),
    );
}

#[test]
fn test_array_access_from_object() {
    parse_ok(
        "a.b[1]",
        AST::access_array(
            AST::access_field(AST::access_variable(Identifier::from("a")), Identifier::from("b")),
            AST::integer(1),
        ),
    );
}

#[test]
fn test_array_access_from_array() {
    parse_ok(
        "a[b][1]",
        AST::access_array(
            AST::access_array(
                AST::access_variable(Identifier::from("a")),
                AST::access_variable(Identifier::from("b")),
            ),
            AST::integer(1),
        ),
    );
}

#[test]
fn test_array_call_method_on_member() {
    parse_ok(
        "a[b].c(1)",
        AST::call_method(
            AST::access_array(
                AST::access_variable(Identifier::from("a")),
                AST::access_variable(Identifier::from("b")),
            ),
            Identifier::from("c"),
            vec![AST::integer(1)],
        ),
    );
}

#[test]
fn test_array_access_member_of_member() {
    parse_ok(
        "a[b].a",
        AST::access_field(
            AST::access_array(
                AST::access_variable(Identifier::from("a")),
                AST::access_variable(Identifier::from("b")),
            ),
            Identifier::from("a"),
        ),
    );
}

#[test]
fn test_array_access_with_array_access_as_index() {
    parse_ok(
        "a[b[c]]",
        AST::access_array(
            AST::access_variable(Identifier::from("a")),
            AST::access_array(
                AST::access_variable(Identifier::from("b")),
                AST::access_variable(Identifier::from("c")),
            ),
        ),
    );
}

#[test]
fn test_array_access_from_function_call() {
    parse_ok(
        "f(0,0)[x]",
        AST::access_array(
            AST::call_function(Identifier::from("f"), vec![AST::integer(0), AST::integer(0)]),
            AST::access_variable(Identifier::from("x")),
        ),
    );
}

#[test]
fn test_print_call_with_arguments() {
    parse_ok(
        "print(\"~ ~ ~\", 1, true, x)",
        AST::print(
            "~ ~ ~".to_string(),
            vec![AST::integer(1), AST::boolean(true), AST::access_variable(Identifier::from("x"))],
        ),
    );
}

#[test]
fn test_print_call_without_arguments() {
    parse_ok("print(\"~ ~ ~\")", AST::print("~ ~ ~".to_string(), vec![]));
}

#[test]
fn test_print_call_string() {
    parse_ok("print(\"hello world\")", AST::print("hello world".to_string(), vec![]));
}

#[test]
fn test_print_call_empty_string() {
    parse_ok("print(\"\")", AST::print(String::new(), vec![]));
}

#[test]
fn test_two_prints() {
    parse_ok(
        "begin print(\"\"); print(\"\"); end",
        AST::block(vec![AST::print(String::new(), vec![]), AST::print(String::new(), vec![])]),
    )
}

#[test]
fn test_print_call_escape_newline() {
    parse_ok("print(\"\\n\")", AST::print("\\n".to_string(), vec![]));
}

#[test]
fn test_print_call_escape_tab() {
    parse_ok("print(\"\\t\")", AST::print("\\t".to_string(), vec![]));
}

#[test]
fn test_print_call_escape_return() {
    parse_ok("print(\"\\r\")", AST::print("\\r".to_string(), vec![]));
}

#[test]
fn test_print_call_escape_backslash() {
    parse_ok("print(\"\\\\\")", AST::print("\\\\".to_string(), vec![]));
}

#[test]
fn test_print_call_botched_escape() {
    parse_err("print(\"\\\")");
}
#[test]
fn test_print_call_invalid_escape() {
    parse_err("print(\"\\a\")");
}

#[test]
fn test_simple_disjunction() {
    parse_ok(
        "true | false",
        AST::operation(Operator::Disjunction, AST::boolean(true), AST::boolean(false)),
    );
}

#[test]
fn test_double_disjunction() {
    parse_ok(
        "true | false | true",
        AST::operation(
            Operator::Disjunction,
            AST::operation(Operator::Disjunction, AST::boolean(true), AST::boolean(false)),
            AST::boolean(true),
        ),
    );
}

#[test]
fn test_simple_conjunction() {
    parse_ok(
        "true & false",
        AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
    );
}

#[test]
fn test_double_conjunction() {
    parse_ok(
        "true & false & true",
        AST::operation(
            Operator::Conjunction,
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
            AST::boolean(true),
        ),
    );
}

#[test]
fn test_simple_equality() {
    parse_ok(
        "true == false",
        AST::operation(Operator::Equality, AST::boolean(true), AST::boolean(false)),
    );
}

#[test]
fn test_simple_inequality() {
    parse_ok(
        "true != false",
        AST::operation(Operator::Inequality, AST::boolean(true), AST::boolean(false)),
    );
}

#[test]
fn test_disjunction_and_conjunction() {
    //or (true, (true & false & false)))
    parse_ok(
        "true | true & false",
        AST::operation(
            Operator::Disjunction,
            AST::boolean(true),
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
        ),
    );
}

#[test]
fn test_disjunction_and_conjunctions() {
    //or (true, (true & false & false)))
    parse_ok(
        "true & false | true & false",
        AST::operation(
            Operator::Disjunction,
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
        ),
    );
}

#[test]
fn test_disjunctions_and_conjunctions() {
    //or (true, (true & false & false)))
    parse_ok(
        "true & false | true & false | true & false",
        AST::operation(
            Operator::Disjunction,
            AST::operation(
                Operator::Disjunction,
                AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
                AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
            ),
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
        ),
    );
}

#[test]
fn test_more_disjunctions_and_more_conjunctions() {
    //or (true, (true & false & false)))
    parse_ok(
        "true & false & true | true & true & false & true | true & false",
        AST::operation(
            Operator::Disjunction,
            AST::operation(
                Operator::Disjunction,
                AST::operation(
                    Operator::Conjunction,
                    AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
                    AST::boolean(true),
                ),
                AST::operation(
                    Operator::Conjunction,
                    AST::operation(
                        Operator::Conjunction,
                        AST::operation(
                            Operator::Conjunction,
                            AST::boolean(true),
                            AST::boolean(true),
                        ),
                        AST::boolean(false),
                    ),
                    AST::boolean(true),
                ),
            ),
            AST::operation(Operator::Conjunction, AST::boolean(true), AST::boolean(false)),
        ),
    );
}

#[test]
fn test_simple_addition() {
    parse_ok("1 + 2", AST::operation(Operator::Addition, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_addition_to_field_object() {
    parse_ok(
        "a.x + 2",
        AST::operation(
            Operator::Addition,
            AST::access_field(AST::access_variable(Identifier::from("a")), Identifier::from("x")),
            AST::integer(2),
        ),
    );
}

#[test]
fn test_simple_subtraction() {
    parse_ok("1 - 2", AST::operation(Operator::Subtraction, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_multiplication() {
    parse_ok("1 * 2", AST::operation(Operator::Multiplication, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_module() {
    parse_ok("1 % 2", AST::operation(Operator::Module, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_division() {
    parse_ok("1 / 2", AST::operation(Operator::Division, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_less_than() {
    parse_ok("1 < 2", AST::operation(Operator::Less, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_less_or_equal() {
    parse_ok("1 <= 2", AST::operation(Operator::LessEqual, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_greater_than() {
    parse_ok("1 > 2", AST::operation(Operator::Greater, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_simple_greater_or_equal() {
    parse_ok("1 >= 2", AST::operation(Operator::GreaterEqual, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_comment() {
    parse_ok("/* a */", AST::null());
}
#[test]
fn test_comment_in_expression() {
    parse_ok("1 + /* a */ 2", AST::operation(Operator::Addition, AST::integer(1), AST::integer(2)));
}

#[test]
fn test_multiline_comment() {
    parse_ok("/* \n\n\n */", AST::null());
}
