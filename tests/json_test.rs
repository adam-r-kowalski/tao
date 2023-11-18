use yeti;

type Result = std::result::Result<(), yeti::effect::Effect>;

#[tokio::test]
async fn json_to_string_for_map() -> Result {
    let env = yeti::core::environment();
    let (_, actual) = yeti::evaluate_source(
        env,
        r#"
        (json/to-string
         {:first "John"
          :last "Smith"})
    "#,
    )
    .await?;
    let expected =
        yeti::Expression::String("{\n  \"first\": \"John\",\n  \"last\": \"Smith\"\n}".to_string());
    assert_eq!(actual, expected);
    Ok(())
}
