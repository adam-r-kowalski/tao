extern crate alloc;

use crate::effect::{error, Effect};
use crate::evaluate_expressions;
use crate::expression::{Environment, Sqlite};
use crate::extract;
use crate::Expression;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use im::{vector, HashMap, Vector};
use rusqlite::{Connection, ToSql};

type Result<T> = core::result::Result<T, Effect>;

fn create_table(
    map: HashMap<Expression, Expression>,
    table_name: Expression,
) -> Result<Expression> {
    let table_name = &extract::keyword(table_name)?[1..];
    let string = format!("CREATE TABLE {} (", table_name).to_string();
    let columns = extract::array(extract::key(map, ":with-columns")?)?;
    let mut string = columns
        .iter()
        .enumerate()
        .try_fold(string, |mut string, (i, column)| {
            let column = extract::array(column.clone())?;
            let name = extract::keyword(column[0].clone())?;
            let name = &name[1..];
            if i > 0 {
                string.push_str(", ");
            }
            string.push_str(name);
            match column[1].clone() {
                Expression::Keyword(type_name) => {
                    let type_name = &type_name[1..].to_uppercase();
                    string.push(' ');
                    string.push_str(type_name);
                }
                Expression::Array(a) => {
                    let type_name = extract::keyword(a[0].clone())?;
                    let type_name = &type_name[1..].to_uppercase();
                    let argument = extract::integer(a[1].clone())?;
                    string.push(' ');
                    string.push_str(type_name);
                    string.push('(');
                    string.push_str(&argument.to_string());
                    string.push(')');
                }
                _ => return Err(error("Expected keyword")),
            };
            column
                .iter()
                .skip(2)
                .try_fold(string, |mut string, expr| match expr {
                    Expression::Keyword(attribute) => {
                        let attribute = &attribute[1..].to_uppercase();
                        string.push(' ');
                        string.push_str(attribute);
                        Ok(string)
                    }
                    Expression::Array(a) => {
                        let attribute = extract::keyword(a[0].clone())?;
                        let attribute = &attribute[1..].to_uppercase();
                        string.push(' ');
                        string.push_str(attribute);
                        match a[1] {
                            Expression::Nil => {
                                string.push_str(" NULL");
                                Ok(string)
                            }
                            _ => Err(error("Expected nil")),
                        }
                    }
                    _ => Err(error("Expected keyword")),
                })
        })?;
    string.push(')');
    Ok(Expression::Array(vector![Expression::String(string)]))
}

fn insert_into(map: HashMap<Expression, Expression>, table_name: Expression) -> Result<Expression> {
    let table_name = &extract::keyword(table_name)?[1..];
    let string = format!("INSERT INTO {} (", table_name).to_string();
    let columns = extract::array(extract::key(map.clone(), ":columns")?)?;
    let mut string = columns
        .iter()
        .enumerate()
        .try_fold(string, |mut string, (i, column)| {
            if i > 0 {
                string.push_str(", ");
            }
            let column = extract::keyword(column.clone())?;
            let column = &column[1..];
            string.push_str(column);
            Ok(string)
        })?;
    string.push_str(") VALUES");
    let placeholder = columns
        .iter()
        .enumerate()
        .fold(String::new(), |mut string, (i, _)| {
            if i > 0 {
                string.push_str(", ");
            }
            string.push_str("?");
            string
        });
    let values = extract::array(extract::key(map, ":values")?)?;
    let string = (0..values.len()).fold(string, |mut string, i| {
        if i > 0 {
            string.push_str(",");
        }
        string.push_str(" (");
        string.push_str(&placeholder);
        string.push(')');
        string
    });
    let result = vector![Expression::String(string)];
    let result = values.iter().try_fold(result, |result, value| {
        let row = extract::array(value.clone())?;
        let result = row.iter().fold(result, |mut result, column| {
            result.push_back(column.clone());
            result
        });
        Ok(result)
    })?;
    Ok(Expression::Array(result))
}

fn select(map: HashMap<Expression, Expression>, columns: Expression) -> Result<Expression> {
    let columns = extract::array(columns)?;
    let string = "SELECT".to_string();
    let mut string = columns
        .iter()
        .enumerate()
        .try_fold(string, |mut string, (i, column)| {
            if i > 0 {
                string.push_str(",");
            }
            string.push(' ');
            let column = extract::keyword(column.clone())?;
            string.push_str(&column[1..]);
            Ok(string)
        })?;
    let from = extract::keyword(extract::key(map.clone(), ":from")?)?;
    let from = &from[1..];
    string.push_str(" FROM ");
    string.push_str(from);
    if let Some(where_clause) = map.get(&Expression::Keyword(":where".to_string())) {
        let where_clause = extract::array(where_clause.clone())?;
        let op = extract::keyword(where_clause[0].clone())?;
        if op != ":=" {
            return Err(error("Unsupported operator"));
        }
        let lhs = extract::keyword(where_clause[1].clone())?;
        let lhs = &lhs[1..];
        let rhs = where_clause[2].clone();
        string.push_str(" WHERE ");
        string.push_str(lhs);
        string.push_str(" = ?");
        let result = vector![Expression::String(string), rhs];
        Ok(Expression::Array(result))
    } else {
        let result = vector![Expression::String(string)];
        Ok(Expression::Array(result))
    }
}

fn sql_string(expr: Expression) -> Result<Expression> {
    let map = extract::map(expr)?;
    if let Some(table_name) = map.get(&Expression::Keyword(":create-table".to_string())) {
        create_table(map.clone(), table_name.clone())
    } else if let Some(table_name) = map.get(&Expression::Keyword(":insert-into".to_string())) {
        insert_into(map.clone(), table_name.clone())
    } else if let Some(columns) = map.get(&Expression::Keyword(":select".to_string())) {
        select(map.clone(), columns.clone())
    } else {
        Err(error("Unsupported SQL operation"))
    }
}

pub fn connect(env: Environment, _args: Vector<Expression>) -> Result<(Environment, Expression)> {
    match Connection::open_in_memory() {
        Ok(db) => Ok((env, Expression::Sqlite(Sqlite::new(db)))),
        Err(_) => Err(error("Failed to open SQLite database")),
    }
}

pub fn string(env: Environment, args: Vector<Expression>) -> Result<(Environment, Expression)> {
    let (env, args) = evaluate_expressions(env, args)?;
    let expr = sql_string(args[0].clone())?;
    Ok((env, expr))
}

impl ToSql for Expression {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        match self {
            Expression::Integer(i) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(i.to_i64().unwrap()),
            )),
            Expression::String(s) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(s.clone()),
            )),
            Expression::Nil => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Null,
            )),
            e => panic!("Unsupported data type: {:?}", e),
        }
    }
}

pub fn query(env: Environment, args: Vector<Expression>) -> Result<(Environment, Expression)> {
    let (env, args) = evaluate_expressions(env, args)?;
    let db = extract::sqlite(args[0].clone())?;
    let array = extract::array(sql_string(args[1].clone())?)?;
    let string = extract::string(array[0].clone())?;
    let parameters = array
        .iter()
        .skip(1)
        .map(|p| p as &dyn ToSql)
        .collect::<Vec<_>>();
    let result = db.connection.prepare(&string);
    match result {
        Ok(mut stmt) => {
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|c| c.to_string()).collect();
            let rows: Vector<Expression> = stmt
                .query_map(&parameters[..], |row| {
                    let map = column_names.iter().enumerate().fold(
                        HashMap::new(),
                        |mut map, (i, name)| {
                            match row.get_ref(i).unwrap().data_type() {
                                rusqlite::types::Type::Text => {
                                    map.insert(
                                        Expression::Keyword(format!(":{}", name)),
                                        Expression::String(row.get(i).unwrap()),
                                    );
                                }
                                _ => panic!("Unsupported data type"),
                            }
                            map
                        },
                    );
                    Ok(Expression::Map(map))
                })
                .unwrap()
                .map(|row| row.unwrap())
                .collect();
            Ok((env, Expression::Array(rows)))
        }
        Err(e) => {
            return Err(error(&format!("Failed to execute query: {}", e)));
        }
    }
}

pub fn execute(env: Environment, args: Vector<Expression>) -> Result<(Environment, Expression)> {
    let (env, args) = evaluate_expressions(env, args)?;
    let db = extract::sqlite(args[0].clone())?;
    let array = extract::array(sql_string(args[1].clone())?)?;
    let string = extract::string(array[0].clone())?;
    let parameters = array
        .iter()
        .skip(1)
        .map(|p| p as &dyn ToSql)
        .collect::<Vec<_>>();
    match db.connection.execute(&string, &parameters[..]) {
        Ok(_) => Ok((env, Expression::Nil)),
        Err(e) => Err(error(&format!("Failed to execute query: {}", e))),
    }
}

pub fn tables(env: Environment, args: Vector<Expression>) -> Result<(Environment, Expression)> {
    let (env, args) = evaluate_expressions(env, args)?;
    let db = extract::sqlite(args[0].clone())?;
    let result = db
        .connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table';");
    match result {
        Ok(mut stmt) => {
            let column_names: Vec<String> =
                stmt.column_names().iter().map(|c| c.to_string()).collect();
            let rows: Vector<Expression> = stmt
                .query_map([], |row| {
                    let map = column_names.iter().enumerate().fold(
                        HashMap::new(),
                        |mut map, (i, name)| {
                            match row.get_ref(i).unwrap().data_type() {
                                rusqlite::types::Type::Text => {
                                    map.insert(
                                        Expression::Keyword(format!(":{}", name)),
                                        Expression::String(row.get(i).unwrap()),
                                    );
                                }
                                _ => panic!("Unsupported data type"),
                            }
                            map
                        },
                    );
                    Ok(Expression::Map(map))
                })
                .unwrap()
                .map(|row| row.unwrap())
                .collect();
            Ok((env, Expression::Array(rows)))
        }
        Err(e) => {
            return Err(error(&format!("Failed to execute query: {}", e)));
        }
    }
}
