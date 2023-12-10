use compiler;

type Result = std::result::Result<(), compiler::effect::Effect>;

#[tokio::test]
async fn yaml_to_string_for_map() -> Result {
    let env = compiler::core::environment();
    let (_, actual) =
        compiler::evaluate_source(env, r#"(yaml/to-string {:first "John" :last "Smith"})"#).await?;
    let expected = compiler::Expression::String("first: John\nlast: Smith\n".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn yaml_to_string_for_map_with_int() -> Result {
    let env = compiler::core::environment();
    let (_, actual) =
        compiler::evaluate_source(env, r#"(yaml/to-string {:first "John" :age 20})"#).await?;
    let expected = compiler::Expression::String("age: 20\nfirst: John\n".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn yaml_from_string_for_map() -> Result {
    let env = compiler::core::environment();
    let (env, actual) =
        compiler::evaluate_source(env, r#"(yaml/from-string "first: John\nlast: Smith\n")"#)
            .await?;
    let (_, expected) = compiler::evaluate_source(env, r#"{:first "John" :last "Smith"}"#).await?;
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn yaml_from_string_for_map_with_int() -> Result {
    let env = compiler::core::environment();
    let (env, actual) =
        compiler::evaluate_source(env, r#"(yaml/from-string "first: John\nage: 20\n")"#).await?;
    let (_, expected) = compiler::evaluate_source(env, r#"{:first "John" :age 20}"#).await?;
    assert_eq!(actual, expected);
    Ok(())
}
