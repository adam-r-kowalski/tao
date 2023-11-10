extern crate alloc;

use crate::effect::{error, Effect};
use crate::expression::{Call, Environment, Pattern, Result};
use crate::Expression;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use im::Vector;

fn evaluate_symbol(environment: Environment, symbol: String) -> Result {
    if let Some(e) = environment.get(&symbol) {
        Ok((environment.clone(), e.clone()))
    } else {
        Err(error(&format!(
            "Symbol {} not found in environment",
            symbol
        )))
    }
}

pub fn pattern_match(
    env: Environment,
    pattern: Expression,
    value: Expression,
) -> core::result::Result<Environment, Effect> {
    match pattern {
        Expression::Symbol(s) => {
            let mut env = env.clone();
            env.insert(s, value);
            Ok(env)
        }
        Expression::Keyword(k1) => match value {
            Expression::Keyword(k2) if k1 == k2 => Ok(env),
            _ => Err(error(&format!(
                "Cannot pattern match {} with {}",
                Expression::Keyword(k1),
                value
            ))),
        },
        Expression::String(s1) => match value {
            Expression::String(s2) if s1 == s2 => Ok(env),
            _ => Err(error(&format!(
                "Cannot pattern match {} with {}",
                Expression::String(s1),
                value
            ))),
        },
        Expression::Integer(i1) => match value {
            Expression::Integer(i2) if i1 == i2 => Ok(env),
            _ => Err(error(&format!(
                "Cannot pattern match {} with {}",
                Expression::Integer(i1),
                value
            ))),
        },
        Expression::Nil => match value {
            Expression::Nil => Ok(env),
            _ => Err(error(&format!(
                "Cannot pattern match {} with {}",
                Expression::Nil,
                value
            ))),
        },
        Expression::Array(patterns) => {
            if let Expression::Array(values) = value {
                let env = patterns
                    .into_iter()
                    .zip(values.into_iter())
                    .try_fold(env, |env, (pattern, value)| {
                        pattern_match(env, pattern, value)
                    })?;
                Ok(env)
            } else {
                Err(error(&format!(
                    "Cannot pattern match {} with {}",
                    Expression::Array(patterns),
                    value
                )))
            }
        }
        Expression::Map(map) => {
            if let Expression::Map(m) = value {
                let env = map.into_iter().try_fold(env, |env, (pattern, value)| {
                    if let Some(v) = m.get(&pattern) {
                        pattern_match(env, value, v.clone())
                    } else {
                        Err(error(&format!(
                            "Cannot pattern match {} with {}",
                            pattern, value
                        )))
                    }
                })?;
                Ok(env)
            } else {
                Err(error(&format!(
                    "Cannot pattern match {} with {}",
                    Expression::Map(map),
                    value
                )))
            }
        }
        _ => Err(error(&format!(
            "Cannot pattern match {} with {}",
            pattern, value
        ))),
    }
}

fn find_pattern_match(
    env: Environment,
    patterns: Vector<Pattern>,
    arguments: Vector<Expression>,
) -> core::result::Result<(Environment, Vector<Expression>), Effect> {
    let mut failures = vec![];
    for Pattern { parameters, body } in patterns {
        let result = pattern_match(
            env.clone(),
            Expression::Array(parameters.clone()),
            Expression::Array(arguments.clone()),
        );
        match result {
            Ok(env) => return Ok((env, body)),
            Err(e) => failures.push(e),
        };
    }
    let error_message = failures.iter().fold(String::new(), |mut s, e| {
        s.push_str(&format!("{}\n", e));
        s
    });
    Err(error(&error_message))
}

fn evaluate_call(environment: Environment, call: Call) -> Result {
    let Call {
        function,
        arguments,
    } = call;
    let (environment, function) = evaluate(environment.clone(), *function)?;
    match function {
        Expression::Function(patterns) => {
            let original_environment = environment.clone();
            let (environment, arguments) = evaluate_expressions(environment, arguments)?;
            let (environment, body) = find_pattern_match(environment, patterns, arguments)?;
            let (_, value) = body.iter().try_fold(
                (environment, Expression::Nil),
                |(environment, _), expression| evaluate(environment, expression.clone()),
            )?;
            Ok((original_environment, value))
        }
        Expression::NativeFunction(f) => f(environment, arguments),
        Expression::Keyword(k) => {
            let (environment, arguments) = evaluate_expressions(environment, arguments)?;
            match &arguments[0] {
                Expression::Map(m) => {
                    if let Some(v) = m.get(&Expression::Keyword(k)) {
                        Ok((environment, v.clone()))
                    } else if arguments.len() == 2 {
                        Ok((environment, arguments[1].clone()))
                    } else {
                        Ok((environment, Expression::Nil))
                    }
                }
                e => Err(error(&format!("Cannot call keyword {} on {}", k, e))),
            }
        }
        Expression::Map(m) => {
            let (environment, arguments) = evaluate_expressions(environment, arguments)?;
            if let Some(v) = m.get(&arguments[0]) {
                Ok((environment, v.clone()))
            } else if arguments.len() == 2 {
                Ok((environment, arguments[1].clone()))
            } else {
                Ok((environment, Expression::Nil))
            }
        }
        _ => Err(error(&format!("Cannot call {}", function))),
    }
}

pub fn evaluate(environment: Environment, expression: Expression) -> Result {
    match expression {
        Expression::Symbol(s) => evaluate_symbol(environment, s),
        Expression::Call(call) => evaluate_call(environment, call),
        Expression::Array(a) => {
            let (environment, a) = evaluate_expressions(environment, a)?;
            Ok((environment, Expression::Array(a)))
        }
        Expression::Map(m) => {
            let (environment, m) = m.into_iter().try_fold(
                (environment, im::OrdMap::new()),
                |(environment, mut m), (k, v)| {
                    let (environment, k) = evaluate(environment, k)?;
                    let (environment, v) = evaluate(environment, v)?;
                    m.insert(k, v);
                    Ok((environment, m))
                },
            )?;
            Ok((environment, Expression::Map(m)))
        }
        Expression::Quote(e) => Ok((environment, *e)),
        e => Ok((environment, e)),
    }
}

pub fn evaluate_expressions(
    environment: Environment,
    expressions: Vector<Expression>,
) -> core::result::Result<(Environment, Vector<Expression>), Effect> {
    expressions.into_iter().try_fold(
        (environment, Vector::new()),
        |(environment, mut expressions), expression| {
            let (environment, argument) = evaluate(environment, expression)?;
            expressions.push_back(argument);
            Ok((environment, expressions))
        },
    )
}

pub fn evaluate_source(
    env: Environment,
    source: &str,
) -> core::result::Result<(Environment, Expression), Effect> {
    let tokens = crate::Tokens::from_str(source);
    let expression = crate::parse(tokens);
    evaluate(env, expression)
}
