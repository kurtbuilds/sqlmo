use crate::query::Expr;
use crate::util::SqlExtension;
use crate::{Dialect, Select, ToSql};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnConflict {
    Ignore,
    Abort,
    /// Only valid for Sqlite, because we
    Replace,
    /// Only valid for Postgres
    DoUpdate {
        conflict: Conflict,
        updates: Vec<(String, Expr)>,
    },
    /// Only valid for Postgres
    DoUpdateAllRows {
        conflict: Conflict,
        alternate_values: HashMap<String, Expr>,
        ignore_columns: Vec<String>,
    },
}

impl OnConflict {
    pub fn do_update_all_rows(columns: &[&str]) -> Self {
        OnConflict::DoUpdateAllRows {
            conflict: Conflict::Columns(columns.iter().map(|c| c.to_string()).collect()),
            alternate_values: HashMap::new(),
            ignore_columns: Vec::new(),
        }
    }

    pub fn do_update_on_pkey(pkey: &str) -> Self {
        OnConflict::DoUpdateAllRows {
            conflict: Conflict::Columns(vec![pkey.to_string()]),
            alternate_values: HashMap::new(),
            ignore_columns: Vec::new(),
        }
    }

    pub fn alternate_value<V: Into<Expr>>(mut self, column: &str, value: V) -> Self {
        match &mut self {
            OnConflict::DoUpdateAllRows {
                alternate_values, ..
            } => {
                alternate_values.insert(column.to_string(), value.into());
            }
            _ => panic!("alternate_value is only valid for DoUpdate"),
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conflict {
    Columns(Vec<String>),
    ConstraintName(String),
    NoTarget,
}

impl Conflict {
    pub fn columns(t: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Conflict::Columns(t.into_iter().map(|c| c.into()).collect())
    }

    pub fn as_columns(&self) -> Option<&Vec<String>> {
        match self {
            Conflict::Columns(c) => Some(c),
            _ => None,
        }
    }
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict::Abort
    }
}

impl ToSql for Conflict {
    fn write_sql(&self, buf: &mut String, _dialect: Dialect) {
        match self {
            Conflict::Columns(c) => {
                buf.push('(');
                buf.push_quoted_sequence(c, ", ");
                buf.push(')');
            }
            Conflict::ConstraintName(name) => {
                buf.push_str("ON CONSTRAINT ");
                buf.push_quoted(name);
            }
            Conflict::NoTarget => {}
        }
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
                    for v in &value.0 {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value(Vec<String>);

impl Value {
    pub fn with(values: &[&str]) -> Self {
        Self(values.into_iter().map(|v| v.to_string()).collect())
    }

    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn column(mut self, value: &str) -> Self {
        self.0.push(value.to_string());
        self
    }

    pub fn placeholders(mut self, count: usize, dialect: Dialect) -> Self {
        use Dialect::*;
        for i in 1..(count + 1) {
            match dialect {
                Postgres => self.0.push(format!("${}", i)),
                Mysql | Sqlite => self.0.push("?".to_string()),
            }
        }
        self
    }
}

impl From<Vec<String>> for Value {
    fn from(values: Vec<String>) -> Self {
        Self(values)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Values {
    Values(Vec<Value>),
    Select(Select),
    DefaultValues,
}

impl From<&[&[&'static str]]> for Values {
    fn from(values: &[&[&'static str]]) -> Self {
        Self::Values(values.into_iter().map(|v| Value::with(v)).collect())
    }
}

impl From<&[&'static str]> for Values {
    fn from(values: &[&'static str]) -> Self {
        Self::Values(vec![Value::with(values)])
    }
}

impl Values {
    pub fn new_value(value: Value) -> Self {
        Self::Values(vec![value])
    }

    pub fn select(select: Select) -> Self {
        Self::Select(select)
    }

    pub fn default_values() -> Self {
        Self::DefaultValues
    }

    pub fn value(mut self, value: Value) -> Self {
        match &mut self {
            Self::Values(values) => values.push(value),
            _ => panic!("Cannot add value to non-values"),
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn values(mut self, value: Values) -> Self {
        self.values = value;
        self
    }

    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|c| c.to_string()).collect();
        self
    }

    pub fn placeholder_for_each_column(mut self, dialect: Dialect) -> Self {
        self.values = Values::new_value(Value::new().placeholders(self.columns.len(), dialect));
        self
    }

    #[deprecated(note = "Use .values(Values::from(...)) instead")]
    pub fn one_value(mut self, values: &[&str]) -> Self {
        self.values = Values::Values(vec![Value::with(values)]);
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
                DoUpdateAllRows { .. } | DoUpdate { .. } => {
                    panic!("Sqlite does not support ON CONFLICT DO UPDATE")
                }
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
                Abort => {}
                Replace => panic!("Postgres does not support ON CONFLICT REPLACE"),
                DoUpdate { conflict, updates } => {
                    buf.push_str(" ON CONFLICT ");
                    buf.push_sql(conflict, dialect);
                    buf.push_str(" DO UPDATE SET ");
                    let updates: Vec<Expr> = updates
                        .into_iter()
                        .map(|(c, v)| Expr::new_eq(Expr::column(c), v.clone()))
                        .collect();
                    buf.push_sql_sequence(&updates, ", ", dialect);
                }
                DoUpdateAllRows {
                    conflict,
                    alternate_values,
                    ignore_columns,
                } => {
                    buf.push_str(" ON CONFLICT ");
                    buf.push_sql(conflict, dialect);
                    buf.push_str(" DO UPDATE SET ");
                    let conflict_columns = conflict.as_columns();
                    let columns: Vec<Expr> = self
                        .columns
                        .iter()
                        .filter(|&c| !ignore_columns.contains(c))
                        .filter(|&c| conflict_columns.map(|conflict| !conflict.contains(c)).unwrap_or(true))
                        .map(|c| {
                            let r = if let Some(v) = alternate_values.get(c) {
                                v.clone()
                            } else {
                                Expr::excluded(c)
                            };
                            Expr::new_eq(Expr::column(c), r)
                        })
                        .collect();
                    buf.push_sql_sequence(&columns, ", ", dialect);
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
    use pretty_assertions::assert_eq;
    use super::*;
    use crate::query::{Case, Expr};

    #[test]
    fn test_basic() {
        let insert = Insert {
            schema: None,
            table: "foo".to_string(),
            columns: vec!["bar".to_string(), "baz".to_string()],
            values: Values::from(&[&["1", "2"] as &[&str], &["3", "4"]] as &[&[&str]]),
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
            .on_conflict(OnConflict::do_update_all_rows(&["bar"]));
        let expected = r#"INSERT INTO "foo" ("bar", "baz", "qux", "wibble", "wobble", "wubble") VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT ("bar") DO UPDATE SET "baz" = excluded."baz", "qux" = excluded."qux", "wibble" = excluded."wibble", "wobble" = excluded."wobble", "wubble" = excluded."wubble""#;
        assert_eq!(insert.to_sql(Dialect::Postgres), expected);
    }

    #[test]
    fn test_override() {
        let columns = &["id", "name", "email"];

        let update_conditional = columns
            .iter()
            .map(|&c| {
                Expr::not_distinct_from(
                    Expr::table_column("users", c),
                    Expr::excluded(c),
                )
            })
            .collect::<Vec<_>>();
        let on_conflict_update_value = Expr::case(
            Case::new_when(
                Expr::new_and(update_conditional),
                Expr::table_column("users", "updated_at"),
            )
            .els("excluded.updated_at"),
        );

        let insert = Insert::new("users")
            .columns(columns)
            .column("updated_at")
            .values(Values::new_value(Value::with(&[
                "1",
                "Kurt",
                "test@example.com",
                "NOW()",
            ])))
            .on_conflict(
                OnConflict::do_update_on_pkey("id")
                    .alternate_value("updated_at", on_conflict_update_value),
            );
        let sql = insert.to_sql(Dialect::Postgres);
        let expected = r#"
INSERT INTO "users" ("id", "name", "email", "updated_at") VALUES
(1, Kurt, test@example.com, NOW())
ON CONFLICT ("id") DO UPDATE SET
"name" = excluded."name",
"email" = excluded."email",
"updated_at" = CASE WHEN
("users"."id" IS NOT DISTINCT FROM excluded."id" AND
"users"."name" IS NOT DISTINCT FROM excluded."name" AND
"users"."email" IS NOT DISTINCT FROM excluded."email")
THEN "users"."updated_at"
ELSE excluded.updated_at END
"#
        .replace("\n", " ");
        assert_eq!(sql, expected.trim());
    }
}
