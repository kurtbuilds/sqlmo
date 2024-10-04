use crate::{Dialect, ToSql};

pub(crate) trait SqlExtension {
    fn push_quoted<T: AsRef<str>>(&mut self, s: T);
    fn push_table_name(&mut self, schema: &Option<String>, table: &str);
    fn push_sql<T: ToSql>(&mut self, sql: &T, dialect: Dialect);
    fn push_sql_sequence<T: ToSql>(&mut self, sql: &[T], separator: &str, dialect: Dialect);
    fn push_quoted_sequence(&mut self, sql: &[String], separator: &str);
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

    fn push_table_name(&mut self, schema: &Option<String>, table: &str) {
        if let Some(schema) = schema {
            self.push_quoted(schema);
            self.push('.');
        }
        self.push_quoted(table);
    }

    fn push_sql<T: ToSql>(&mut self, sql: &T, dialect: Dialect) {
        sql.write_sql(self, dialect);
    }

    fn push_sql_sequence<T: ToSql>(&mut self, sql: &[T], separator: &str, dialect: Dialect) {
        let mut first = true;
        for s in sql.into_iter() {
            if !first {
                self.push_str(separator);
            }
            s.write_sql(self, dialect);
            first = false;
        }
    }

    fn push_quoted_sequence(&mut self, sql: &[String], separator: &str) {
        let mut first = true;
        for s in sql {
            if !first {
                self.push_str(separator);
            }
            self.push_quoted(s);
            first = false;
        }
    }
}

pub fn pkey_column_names(schema: &str) -> Vec<String> {
    vec![
        format!("{}_id", schema),
        format!("{}_uuid", schema),
        "id".to_string(),
        "uuid".to_string(),
    ]
}
