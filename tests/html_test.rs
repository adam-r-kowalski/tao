use yeti;

type Result = std::result::Result<(), yeti::effect::Effect>;

#[tokio::test]
async fn evaluate_html_is_module() -> Result {
    let env = yeti::core::environment();
    let (_, actual) = yeti::evaluate_source(env, "html").await?;
    assert!(matches!(actual, yeti::Expression::Module(_)));
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_only_tag() -> Result {
    let env = yeti::core::environment();
    let (_, actual) = yeti::evaluate_source(env, "(html/to-string [:div])").await?;
    let expected = yeti::Expression::String("<div></div>".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_child() -> Result {
    let tokens = yeti::Tokens::from_str(r#"(html/to-string [:ul [:li "hello"]])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected = yeti::Expression::String("<ul><li>hello</li></ul>".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_two_children() -> Result {
    let tokens = yeti::Tokens::from_str(r#"(html/to-string [:ul [:li "first"] [:li "second"]])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected = yeti::Expression::String("<ul><li>first</li><li>second</li></ul>".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_attribute() -> Result {
    let tokens = yeti::Tokens::from_str(r#"(html/to-string [:div {:class "red"}])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected = yeti::Expression::String(r#"<div class="red"></div>"#.to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_attribute_and_doesnt_need_closing_tag() -> Result {
    let tokens =
        yeti::Tokens::from_str(r#"(html/to-string [:input {:type "checkbox" :name "agree"}])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected =
        yeti::Expression::String(r#"<input name="agree" type="checkbox" />"#.to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_css() -> Result {
    let tokens = yeti::Tokens::from_str(
        r#"
        (html/to-string
         [:style
          {:body {:background-color "red"}}])
        "#,
    );
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected_html = "<style>body { background-color: red; }</style>".to_string();
    let expected = yeti::Expression::String(expected_html);
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_array_of_child() -> Result {
    let tokens = yeti::Tokens::from_str(r#"(html/to-string [:ul [[:li "first"] [:li "second"]]])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected = yeti::Expression::String("<ul><li>first</li><li>second</li></ul>".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_with_int() -> Result {
    let tokens = yeti::Tokens::from_str(r#"(html/to-string [:ul [:li 1] [:li 2]])"#);
    let expression = yeti::parse(tokens);
    let environment = yeti::core::environment();
    let (_, actual) = yeti::evaluate(environment.clone(), expression).await?;
    let expected = yeti::Expression::String("<ul><li>1</li><li>2</li></ul>".to_string());
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_from_string_div() -> Result {
    let env = yeti::core::environment();
    let (env, actual) = yeti::evaluate_source(env, "(html/from-string \"<div></div>\")").await?;
    let (_, expected) = yeti::evaluate_source(env, "[:html [:head] [:body [:div]]]").await?;
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_from_string_div_with_attribute() -> Result {
    let env = yeti::core::environment();
    let (env, actual) = yeti::evaluate_source(
        env,
        r#"(html/from-string "<div id=\"foo\" class=\"bar\"></div>")"#,
    )
    .await?;
    let (_, expected) = yeti::evaluate_source(
        env,
        r#"[:html [:head] [:body [:div {:id "foo" :class "bar"}]]]"#,
    )
    .await?;
    assert_eq!(actual, expected);
    Ok(())
}

#[tokio::test]
async fn evaluate_html_from_string_children() -> Result {
    let env = yeti::core::environment();
    let (env, actual) = yeti::evaluate_source(
        env,
        r#"(html/from-string "<ul><li>first</li><li>second</li></ul>")"#,
    )
    .await?;
    let (_, expected) = yeti::evaluate_source(
        env,
        r#"[:html [:head] [:body [:ul [:li "first"] [:li "second"]]]]"#,
    )
    .await?;
    assert_eq!(actual, expected);
    Ok(())
}
