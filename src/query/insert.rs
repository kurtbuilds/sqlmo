use std::collections::HashMap;
use crate::{Dialect, Select, ToSql};
use crate::query::Expr;
use crate::util::SqlExtension;

#[derive(Debug, Clone, PartialEq)]
pub enum OnConflict {
    Ignore,
    Abort,
    /// Only valid for Sqlite, because we
    Replace,

    /// Only valid for Postgres
    DoUpdate {
        target: ConflictTarget,
        alternate_values: HashMap<String, Expr>,
        ignore_columns: Vec<String>,
    },
}

impl OnConflict {
    pub fn do_update(columns: &[&str]) -> Self {
        OnConflict::DoUpdate {
            target: ConflictTarget::Columns(columns.iter().map(|c| c.to_string()).collect()),
            alternate_values: HashMap::new(),
            ignore_columns: Vec::new(),
        }
    }

    pub fn do_update_on_pkey(pkey: &str) -> Self {
        OnConflict::DoUpdate {
            target: ConflictTarget::Columns(vec![pkey.to_string()]),
            alternate_values: HashMap::new(),
            ignore_columns: Vec::new(),
        }
    }

    pub fn alternate_value<V: Into<Expr>>(mut self, column: &str, value: V) -> Self {
        match &mut self {
            OnConflict::DoUpdate { alternate_values, .. } => {
                alternate_values.insert(column.to_string(), value.into());
            }
            _ => panic!("alternate_value is only valid for DoUpdate"),
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictTarget {
    Columns(Vec<String>),
    ConstraintName(String),
    NoTarget,
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
                DoUpdate { .. } => panic!("Sqlite does not support ON CONFLICT DO UPDATE"),
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
                DoUpdate { target, alternate_values, ignore_columns } => {
                    let mut column_filter = ignore_columns.clone();
                    buf.push_str(" ON CONFLICT ");
                    match target {
                        ConflictTarget::Columns(c) => {
                            column_filter.extend_from_slice(c);
                            buf.push('(');
                            buf.push_quoted_sequence(c, ", ");
                            buf.push(')');
                        }
                        ConflictTarget::ConstraintName(name) => {
                            buf.push_str("ON CONSTRAINT ");
                            buf.push_quoted(name);
                        }
                        ConflictTarget::NoTarget => {}
                    }
                    buf.push_str(" DO UPDATE SET ");
                    let mut first = true;
                    for column in self.columns.iter().filter(|c| !column_filter.contains(c)) {
                        if !first {
                            buf.push_str(", ");
                        }
                        buf.push_quoted(column);
                        buf.push_str(" = ");
                        if let Some(alternate_value) = alternate_values.get(column) {
                            buf.push_sql(alternate_value, dialect);
                        } else {
                            buf.push_str("EXCLUDED.");
                            buf.push_quoted(column);
                        }
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
    use crate::query::{Case, Expr};
    use super::*;

    #[test]
    fn test_basic() {
        let insert = Insert {
            schema: None,
            table: "foo".to_string(),
            columns: vec!["bar".to_string(), "baz".to_string()],
            values: Values::from(&[
                &["1", "2"] as &[&str],
                &["3", "4"],
            ] as &[&[&str]]),
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
            .on_conflict(OnConflict::do_update(&["bar"]));
        let expected = r#"INSERT INTO "foo" ("bar", "baz", "qux", "wibble", "wobble", "wubble") VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT ("bar") DO UPDATE SET "baz" = EXCLUDED."baz", "qux" = EXCLUDED."qux", "wibble" = EXCLUDED."wibble", "wobble" = EXCLUDED."wobble", "wubble" = EXCLUDED."wubble""#;
        assert_eq!(insert.to_sql(Dialect::Postgres), expected);
    }

    #[test]
    fn test_override() {
        let columns = &["id", "name", "email"];

        let update_conditional = columns.iter().map(|&c| {
            Expr::not_distinct_from(Expr::table_column("users", c), Expr::table_column("excluded", c))
        }).collect::<Vec<_>>();
        let on_conflict_update_value = Expr::case(
            Case::new_when(Expr::new_and(update_conditional), Expr::table_column("users", "updated_at"))
                .els("excluded.updated_at")
        );

        let insert = Insert::new("users")
            .columns(columns)
            .column("updated_at")
            .values(Values::new_value(Value::with(&[
                "1", "Kurt", "test@example.com", "NOW()",
            ])))
            .on_conflict(OnConflict::do_update_on_pkey("id")
                .alternate_value("updated_at", on_conflict_update_value));
        let sql = insert.to_sql(Dialect::Postgres);
        let expected = r#"
INSERT INTO "users" ("id", "name", "email", "updated_at") VALUES
(1, Kurt, test@example.com, NOW())
ON CONFLICT ("id") DO UPDATE SET
"name" = EXCLUDED."name",
"email" = EXCLUDED."email",
"updated_at" = CASE WHEN
("users"."id" IS NOT DISTINCT FROM "excluded"."id" AND
"users"."name" IS NOT DISTINCT FROM "excluded"."name" AND
"users"."email" IS NOT DISTINCT FROM "excluded"."email")
THEN "users"."updated_at"
ELSE excluded.updated_at END
"#.replace("\n", " ");
        assert_eq!(sql, expected.trim());
    }
}