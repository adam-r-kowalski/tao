use im::{hashmap, vector, HashMap};
use rug::{Integer, Rational};
use tao;

type Result = std::result::Result<(), tao::RaisedEffect>;

#[test]
fn evaluate_keyword() -> Result {
    let tokens = tao::tokenize(":x");
    let expression = tao::parse(tokens);
    let environment = HashMap::new();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Keyword(":x".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_string() -> Result {
    let tokens = tao::tokenize(r#""hello""#);
    let expression = tao::parse(tokens);
    let environment = HashMap::new();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::String("hello".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_integer() -> Result {
    let tokens = tao::tokenize("5");
    let expression = tao::parse(tokens);
    let environment = HashMap::new();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(5));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_float() -> Result {
    let tokens = tao::tokenize("3.14");
    let expression = tao::parse(tokens);
    let environment = HashMap::new();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Float(tao::Float::from_str("3.14"));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_symbol_bound_to_integer() -> Result {
    let tokens = tao::tokenize("x");
    let expression = tao::parse(tokens);
    let environment = hashmap! {
        "x".to_string() => tao::Expression::Integer(Integer::from(5)),
    };
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(5));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_symbol_bound_to_function() -> Result {
    let tokens = tao::tokenize("(double 5)");
    let expression = tao::parse(tokens);
    let environment = hashmap! {
        "double".to_string() => tao::Expression::IntrinsicFunction(
          |env, args| {
            let (env, args) = tao::evaluate_expressions(env, args)?;
            match &args[0] {
              tao::Expression::Integer(i) => Ok((env, tao::Expression::Integer(i * Integer::from(2)))),
              _ => panic!("Expected integer argument"),
            }
          }
        ),
    };
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(10));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_add() -> Result {
    let tokens = tao::tokenize("(+ 5 3)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(8));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_if_then_branch() -> Result {
    let tokens = tao::tokenize("(if true 1 2)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(1));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_if_else_branch() -> Result {
    let tokens = tao::tokenize("(if false 1 2)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment, expression)?;
    let expected = tao::Expression::Integer(Integer::from(2));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_def() -> Result {
    let tokens = tao::tokenize("(def x 5)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (actual_environment, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Nil;
    assert_eq!(actual, expected);
    let mut expected_environment = environment;
    expected_environment.insert("x".to_string(), tao::Expression::Integer(Integer::from(5)));
    assert_eq!(actual_environment, expected_environment);
    Ok(())
}

#[test]
fn evaluate_array() -> Result {
    let tokens = tao::tokenize("[(+ 1 2) (/ 4 3)]");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Array(vector![
        tao::Expression::Integer(Integer::from(3)),
        tao::Expression::Ratio(Rational::from((Integer::from(4), Integer::from(3)))),
    ]);
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_map() -> Result {
    let tokens = tao::tokenize("{:a (+ 1 2) :b (/ 4 3)}");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Map(hashmap! {
        tao::Expression::Keyword(":a".to_string()) => tao::Expression::Integer(Integer::from(3)),
        tao::Expression::Keyword(":b".to_string()) => tao::Expression::Ratio(Rational::from((Integer::from(4), Integer::from(3)))),
    });
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_key_on_map() -> Result {
    let tokens = tao::tokenize("(:a {:a 1})");
    let expression = tao::parse(tokens);
    let environment = hashmap! {};
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Integer(Integer::from(1));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_map_on_key() -> Result {
    let tokens = tao::tokenize("({:a 1} :a)");
    let expression = tao::parse(tokens);
    let environment = hashmap! {};
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Integer(Integer::from(1));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_assoc() -> Result {
    let tokens = tao::tokenize("(assoc {} :a 1)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Map(hashmap! {
        tao::Expression::Keyword(":a".to_string()) => tao::Expression::Integer(Integer::from(1)),
    });
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_dissoc() -> Result {
    let tokens = tao::tokenize("(dissoc {:a 1} :a)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Map(hashmap! {});
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_merge() -> Result {
    let tokens = tao::tokenize("(merge {:a 1} {:b 2})");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Map(hashmap! {
        tao::Expression::Keyword(":a".to_string()) => tao::Expression::Integer(Integer::from(1)),
        tao::Expression::Keyword(":b".to_string()) => tao::Expression::Integer(Integer::from(2)),
    });
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_quote() -> Result {
    let tokens = tao::tokenize("'(1 2)");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Call {
        function: Box::new(tao::Expression::Integer(Integer::from(1))),
        arguments: vector![tao::Expression::Integer(Integer::from(2)),],
    };
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_eval() -> Result {
    let tokens = tao::tokenize("(eval '(+ 1 2))");
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Integer(Integer::from(3));
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn evaluate_read_string() -> Result {
    let tokens = tao::tokenize(r#"(read-string "(+ 1 2)")"#);
    let expression = tao::parse(tokens);
    let environment = tao::core::environment();
    let (_, actual) = tao::evaluate(environment.clone(), expression)?;
    let expected = tao::Expression::Call {
        function: Box::new(tao::Expression::Symbol("+".to_string())),
        arguments: vector![
            tao::Expression::Integer(Integer::from(1)),
            tao::Expression::Integer(Integer::from(2)),
        ],
    };
    assert_eq!(actual, expected);
    Ok(())
}
