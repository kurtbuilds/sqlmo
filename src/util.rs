use crate::{Dialect, ToSql};

pub trait SqlExtension {
    fn push_quoted<T: AsRef<str>>(&mut self, s: T);
    fn push_table_name(&mut self, schema: &Option<String>, table: &str, alias: Option<&String>);
    fn push_sql<T: ToSql>(&mut self, sql: &T, dialect: Dialect);
    fn push_sql_sequence<T: ToSql>(&mut self, sql: &Vec<T>, separator: &str, dialect: Dialect);
}

impl SqlExtension for String {
    fn push_quoted<T: AsRef<str>>(&mut self, s: T) {
        if s.as_ref().contains('"') {
            panic!("Cannot quote string with double quotes");
        }
        self.push('"');
        self.push_str(s.as_ref());
        self.push('"');
    }

    fn push_table_name(&mut self, schema: &Option<String>, table: &str, alias: Option<&String>) {
        if let Some(schema) = schema {
            self.push_quoted(schema);
            self.push('.');
        }
        self.push_quoted(table);
        if let Some(alias) = alias {
            self.push_str(" AS ");
            self.push_quoted(alias);
        }
    }

    fn push_sql<T: ToSql>(&mut self, sql: &T, dialect: Dialect) {
        sql.write_sql(self, dialect);
    }

    fn push_sql_sequence<T: ToSql>(&mut self, sql: &Vec<T>, separator: &str, dialect: Dialect) {
        let mut first = true;
        for s in sql {
            if !first {
                self.push_str(separator);
            }
            s.write_sql(self, dialect);
            first = false;
        }
    }
}