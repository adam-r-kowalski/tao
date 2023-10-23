extern crate alloc;

use crate::effect::{error, Effect};
use crate::evaluate_expressions;
use crate::expression::Environment;
use crate::extract;
use crate::Expression;
use alloc::string::String;
use alloc::vec::Vec;
use im::{HashMap, Vector};

type Result<T> = core::result::Result<T, Effect>;

fn self_closing(tag: &str) -> bool {
    match tag {
        "area" => true,
        "base" => true,
        "br" => true,
        "col" => true,
        "embed" => true,
        "hr" => true,
        "img" => true,
        "input" => true,
        "link" => true,
        "meta" => true,
        "param" => true,
        "source" => true,
        "track" => true,
        "wbr" => true,
        _ => false,
    }
}

fn style_tag(style_map: HashMap<Expression, Expression>, string: &mut String) -> Result<()> {
    string.push_str("<style>");
    for (k, v) in style_map {
        let selector = extract::keyword(k.clone())?;
        let rules_map = extract::map(v.clone())?;
        string.push_str(&selector[1..]);
        string.push_str(" { ");
        for (rule_key, rule_value) in rules_map {
            let rule_property = extract::keyword(rule_key.clone())?;
            let rule_val_str = extract::string(rule_value)?;
            string.push_str(&rule_property[1..]);
            string.push_str(": ");
            string.push_str(&rule_val_str);
            string.push_str("; ");
        }
        string.push_str("}");
    }
    string.push_str("</style>");
    Ok(())
}

pub fn build_html_string(expr: Expression, string: &mut String) -> Result<()> {
    match expr {
        Expression::Array(a) => {
            let keyword = extract::keyword(a[0].clone())?;
            let keyword = &keyword[1..];
            if keyword == "style" {
                let style_map = extract::map(a[1].clone())?;
                return style_tag(style_map, string);
            }
            string.push('<');
            string.push_str(keyword);
            if a.len() > 1 {
                if let Expression::Map(m) = &a[1] {
                    let mut entries = Vec::new();
                    for (k, v) in m.iter() {
                        let k = extract::keyword(k.clone())?;
                        entries.push((k, v.clone()));
                    }
                    entries.sort_by_key(|entry| entry.0.clone());
                    for (k, v) in entries {
                        string.push(' ');
                        string.push_str(&k[1..]);
                        string.push_str("=\"");
                        let s = extract::string(v)?;
                        string.push_str(&s);
                        string.push('"');
                    }
                    if self_closing(keyword) {
                        string.push_str(" />");
                        Ok(())
                    } else {
                        string.push('>');
                        for expr in a.iter().skip(2) {
                            build_html_string(expr.clone(), string)?;
                        }
                        string.push_str("</");
                        string.push_str(keyword);
                        string.push('>');
                        Ok(())
                    }
                } else if self_closing(keyword) {
                    string.push_str(" />");
                    Ok(())
                } else {
                    string.push('>');
                    for expr in a.iter().skip(1) {
                        build_html_string(expr.clone(), string)?;
                    }
                    string.push_str("</");
                    string.push_str(keyword);
                    string.push('>');
                    Ok(())
                }
            } else if self_closing(keyword) {
                string.push_str(" />");
                Ok(())
            } else {
                string.push_str("></");
                string.push_str(keyword);
                string.push('>');
                Ok(())
            }
        }
        Expression::String(s) => {
            string.push_str(&s);
            Ok(())
        }
        _ => Err(error("Expected keyword")),
    }
}

pub fn html(env: Environment, args: Vector<Expression>) -> Result<(Environment, Expression)> {
    let (env, args) = evaluate_expressions(env, args)?;
    let mut string = String::new();
    build_html_string(args[0].clone(), &mut string)?;
    Ok((env, Expression::String(string)))
}