use im::{hashmap, vector};
use rug::{Integer, Rational};
use tao;

#[test]
fn parse_symbol() {
    let tokens = tao::tokenize("x");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Symbol("x".to_string());
    assert_eq!(actual, expected);
}

#[test]
fn parse_keyword() {
    let tokens = tao::tokenize(":x");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Keyword(":x".to_string());
    assert_eq!(actual, expected);
}

#[test]
fn parse_string() {
    let tokens = tao::tokenize(r#""hello""#);
    let actual = tao::parse(tokens);
    let expected = tao::Expression::String("hello".to_string());
    assert_eq!(actual, expected);
}

#[test]
fn parse_integer() {
    let tokens = tao::tokenize("123");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Integer(Integer::from(123));
    assert_eq!(actual, expected);
}

#[test]
fn parse_float() {
    let tokens = tao::tokenize("3.14");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Float(tao::Float::from_str("3.14"));
    assert_eq!(actual, expected);
}

#[test]
fn parse_homogenous_array() {
    let tokens = tao::tokenize("[1 2 3]");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Array(vector![
        tao::Expression::Integer(Integer::from(1)),
        tao::Expression::Integer(Integer::from(2)),
        tao::Expression::Integer(Integer::from(3)),
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn parse_heterogenous_array() {
    let tokens = tao::tokenize("[3.14 2 3]");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Array(vector![
        tao::Expression::Float(tao::Float::from_str("3.14")),
        tao::Expression::Integer(Integer::from(2)),
        tao::Expression::Integer(Integer::from(3)),
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn parse_call() {
    let tokens = tao::tokenize("(+ 1 2)");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Call {
        function: Box::new(tao::Expression::Symbol("+".to_string())),
        arguments: vector![
            tao::Expression::Integer(Integer::from(1)),
            tao::Expression::Integer(Integer::from(2)),
        ],
    };
    assert_eq!(actual, expected);
}

#[test]
fn parse_nested_array() {
    let tokens = tao::tokenize("[3.14 [2 3]]");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Array(vector![
        tao::Expression::Float(tao::Float::from_str("3.14")),
        tao::Expression::Array(vector![
            tao::Expression::Integer(Integer::from(2)),
            tao::Expression::Integer(Integer::from(3)),
        ])
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn parse_nested_call() {
    let tokens = tao::tokenize("(+ 3.14 (- 2 3))");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Call {
        function: Box::new(tao::Expression::Symbol("+".to_string())),
        arguments: vector![
            tao::Expression::Float(tao::Float::from_str("3.14")),
            tao::Expression::Call {
                function: Box::new(tao::Expression::Symbol("-".to_string())),
                arguments: vector![
                    tao::Expression::Integer(Integer::from(2)),
                    tao::Expression::Integer(Integer::from(3)),
                ]
            }
        ],
    };
    assert_eq!(actual, expected);
}

#[test]
fn parse_call_inside_array() {
    let tokens = tao::tokenize("[3.14 (+ 2 3)]");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Array(vector![
        tao::Expression::Float(tao::Float::from_str("3.14")),
        tao::Expression::Call {
            function: Box::new(tao::Expression::Symbol("+".to_string())),
            arguments: vector![
                tao::Expression::Integer(Integer::from(2)),
                tao::Expression::Integer(Integer::from(3)),
            ]
        }
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn parse_array_inside_call() {
    let tokens = tao::tokenize("(+ 3.14 [2 3])");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Call {
        function: Box::new(tao::Expression::Symbol("+".to_string())),
        arguments: vector![
            tao::Expression::Float(tao::Float::from_str("3.14")),
            tao::Expression::Array(vector![
                tao::Expression::Integer(Integer::from(2)),
                tao::Expression::Integer(Integer::from(3)),
            ])
        ],
    };
    assert_eq!(actual, expected);
}

#[test]
fn parse_rational() {
    let tokens = tao::tokenize("1/2");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Ratio(Rational::from((Integer::from(1), Integer::from(2))));
    assert_eq!(actual, expected);
}

#[test]
fn parse_map() {
    let tokens = tao::tokenize("{:a 1 :b 2}");
    let actual = tao::parse(tokens);
    let expected = tao::Expression::Map(hashmap![
        tao::Expression::Keyword(":a".to_string()) => tao::Expression::Integer(Integer::from(1)),
        tao::Expression::Keyword(":b".to_string()) => tao::Expression::Integer(Integer::from(2)),
    ]);
    assert_eq!(actual, expected);
}
