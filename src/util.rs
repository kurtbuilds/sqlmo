use crate::{Dialect, ToSql};

pub(crate) fn quote(s: &str) -> String {
    if s.contains('"') {
        panic!("Cannot quote string with double quotes");
    }
    let mut r = String::new();
    r.push('"');
    r.push_str(s);
    r.push('"');
    r
}

pub(crate) fn table_name(schema: Option<&String>, table: &str, alias: Option<&String>) -> String {
    let mut s = String::new();
    if let Some(schema) = schema {
        s.push_str(&quote(schema));
        s.push('.');
    }
    s.push_str(&quote(table));
    if let Some(alias) = alias {
        s.push_str(" AS ");
        s.push_str(&quote(alias));
    }
    s
}

pub(crate) fn column_name(schema: Option<&String>, table: Option<&String>, column: &str, alias: Option<&String>) -> String {
    let mut s = String::new();
    if let Some(schema) = schema {
        s.push_str(&quote(schema));
        s.push('.');
    }
    if let Some(table) = table {
        s.push_str(&quote(table));
        s.push('.');
    }
    s.push_str(&quote(column));
    if let Some(alias) = alias {
        s.push_str(" AS ");
        s.push_str(&quote(alias));
    }
    s
}

pub(crate) fn push_sql_sequence<S: ToSql>(s: &mut String, seq: &Vec<S>, sep: &'static str, dialect: Dialect) {
    let mut first = true;
    for item in seq {
        if !first {
            s.push_str(sep);
        }
        s.push_str(item.to_sql(dialect).as_str());
        first = false;
    }
}