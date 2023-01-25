use crate::{Dialect, Select, ToSql};
use crate::util::SqlExtension;

#[derive(Debug, Clone, PartialEq)]
pub enum OnConflict {
    Ignore,
    Abort,
    /// Only valid for Sqlite, because we
    Replace,

    /// Only valid for Postgres
    DoUpdate(ConflictTarget)
}

impl OnConflict {
    pub fn do_update_on_pkey(pkey: &str) -> Self {
        OnConflict::DoUpdate(ConflictTarget::Columns(vec![pkey.to_string()]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictTarget {
    Columns(Vec<String>),
    ConstraintName(String)
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict::Abort
    }
}

impl ToSql for Values {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Values::Values(values) => {
                let mut first_value = true;
                for value in values {
                    if !first_value {
                        buf.push_str(", ");
                    }
                    let mut first = true;
                    buf.push('(');
                    for v in value {
                        if !first {
                            buf.push_str(", ");
                        }
                        buf.push_str(v);
                        first = false;
                    }
                    buf.push(')');
                    first_value = false;
                }
            }
            Values::Select(select) => {
                buf.push_sql(select, dialect);
            }
            Values::DefaultValues => {
                buf.push_str("DEFAULT VALUES");
            }
        }
    }
}

pub enum Values {
    Values(Vec<Vec<String>>),
    Select(Select),
    DefaultValues,
}

pub struct Insert {
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
    pub values: Values,
    pub on_conflict: OnConflict,
    pub returning: Vec<String>,
}

impl Insert {
    pub fn new(table: &str) -> Self {
        Self {
            schema: None,
            table: table.to_string(),
            columns: Vec::new(),
            values: Values::DefaultValues,
            on_conflict: OnConflict::default(),
            returning: Vec::new(),
        }
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_string());
        self
    }

    pub fn column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        self
    }

    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    pub fn placeholder_for_each_column(mut self, dialect: Dialect) -> Self {
        use Dialect::*;
        let mut placeholders = Vec::new();
        for i in 0..self.columns.len() {
            match dialect {
                Postgres => placeholders.push(format!("${}", i + 1)),
                Mysql => placeholders.push("?".to_string()),
                Sqlite => placeholders.push("?".to_string()),
            }
        }
        self.values = Values::Values(vec![placeholders]);
        self
    }

    pub fn one_value(mut self, values: &[&str]) -> Self {
        self.values = Values::Values(vec![values.iter().map(|v| v.to_string()).collect()]);
        self
    }

    pub fn on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = on_conflict;
        self
    }

    pub fn returning(mut self, returning: &[&str]) -> Self {
        self.returning = returning.iter().map(|r| r.to_string()).collect();
        self
    }
}

impl ToSql for Insert {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use Dialect::*;
        use OnConflict::*;
        if dialect == Sqlite {
            match self.on_conflict {
                Ignore => buf.push_str("INSERT OR IGNORE INTO "),
                Abort => buf.push_str("INSERT OR ABORT INTO "),
                Replace => buf.push_str("INSERT OR REPLACE INTO "),
                DoUpdate(_) => panic!("Sqlite does not support ON CONFLICT DO UPDATE"),
            }
        } else {
            buf.push_str("INSERT INTO ");
        }
        buf.push_table_name(&self.schema, &self.table);
        buf.push_str(" (");
        buf.push_quoted_sequence(&self.columns, ", ");
        buf.push_str(") VALUES ");
        self.values.write_sql(buf, dialect);

        if dialect == Postgres {
            match &self.on_conflict {
                Ignore => buf.push_str(" ON CONFLICT DO NOTHING"),
                Abort => {},
                Replace => panic!("Postgres does not support ON CONFLICT REPLACE"),
                DoUpdate(conflict_target) => {
                    let mut column_filter = Vec::new();
                    buf.push_str(" ON CONFLICT ");
                    match conflict_target {
                        ConflictTarget::Columns(c) => {
                            column_filter = c.clone();
                            buf.push('(');
                            buf.push_quoted_sequence(c, ", ");
                            buf.push(')');
                        }
                        ConflictTarget::ConstraintName(name) => {
                            buf.push_str("ON CONSTRAINT ");
                            buf.push_quoted(name);
                        }
                    }
                    buf.push_str(" DO UPDATE SET ");
                    let mut first = true;
                    for column in self.columns.iter().filter(|c| !column_filter.contains(c)) {
                        if !first {
                            buf.push_str(", ");
                        }
                        buf.push_quoted(column);
                        buf.push_str(" = EXCLUDED.");
                        buf.push_quoted(column);
                        first = false;
                    }
                }
            }
        }

        if !self.returning.is_empty() {
            buf.push_str(" RETURNING ");
            buf.push_quoted_sequence(&self.returning, ", ");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let insert = Insert {
            schema: None,
            table: "foo".to_string(),
            columns: vec!["bar".to_string(), "baz".to_string()],
            values: Values::Values(vec![
                vec!["1".to_string(), "2".to_string()],
                vec!["3".to_string(), "4".to_string()],
            ]),
            on_conflict: OnConflict::Abort,
            returning: vec!["id".to_string()],
        };
        assert_eq!(
            insert.to_sql(Dialect::Postgres),
            r#"INSERT INTO "foo" ("bar", "baz") VALUES (1, 2), (3, 4) RETURNING "id""#
        );
    }

    #[test]
    fn test_placeholders() {
        let insert = Insert::new("foo")
            .columns(&["bar", "baz", "qux", "wibble", "wobble", "wubble"])
            .placeholder_for_each_column(Dialect::Postgres)
            .on_conflict(OnConflict::DoUpdate(ConflictTarget::Columns(vec!["bar".to_string()])));
        let expected = r#"INSERT INTO "foo" ("bar", "baz", "qux", "wibble", "wobble", "wubble") VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT ("bar") DO UPDATE SET "baz" = EXCLUDED."baz", "qux" = EXCLUDED."qux", "wibble" = EXCLUDED."wibble", "wobble" = EXCLUDED."wobble", "wubble" = EXCLUDED."wubble""#;
        assert_eq!(insert.to_sql(Dialect::Postgres), expected);
    }
}
