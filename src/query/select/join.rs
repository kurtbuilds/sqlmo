use crate::query::Where;
use crate::util::SqlExtension;
use crate::{Dialect, Select, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinTable {
    Select(Select),
    Table {
        schema: Option<String>,
        table: String,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl Default for JoinType {
    fn default() -> Self {
        JoinType::Inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Criteria {
    On(Where),
    Using(Vec<String>),
}

impl From<Where> for Criteria {
    fn from(where_: Where) -> Self {
        Criteria::On(where_)
    }
}

impl ToSql for Criteria {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Criteria::On(where_) => {
                buf.push_str(" ON ");
                buf.push_sql(where_, dialect);
            }
            Criteria::Using(columns) => {
                buf.push_str(" USING (");
                buf.push_quoted_sequence(columns, ", ");
                buf.push(')');
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join {
    pub typ: JoinType,
    pub table: JoinTable,
    pub alias: Option<String>,
    pub criteria: Criteria,
}

impl Join {
    pub fn new(table: &str) -> Self {
        Self {
            typ: JoinType::Inner,
            table: JoinTable::Table {
                schema: None,
                table: table.to_string(),
            },
            alias: None,
            criteria: Criteria::On(Where::And(vec![])),
        }
    }

    pub fn left(table: &str) -> Self {
        Self {
            typ: JoinType::Left,
            table: JoinTable::Table {
                schema: None,
                table: table.to_string(),
            },
            alias: None,
            criteria: Criteria::On(Where::And(vec![])),
        }
    }

    pub fn alias(mut self, alias: &str) -> Self {
        self.alias = Some(alias.to_string());
        self
    }

    pub fn on_raw(mut self, on: impl Into<String>) -> Self {
        self.criteria = Criteria::On(Where::raw(on));
        self
    }
}

impl ToSql for Join {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use JoinTable::*;
        use JoinType::*;
        match self.typ {
            Inner => buf.push_str("JOIN "),
            Left => buf.push_str("LEFT JOIN "),
            Right => buf.push_str("RIGHT JOIN "),
            Full => buf.push_str("FULL JOIN "),
        }
        match &self.table {
            Select(s) => {
                buf.push('(');
                buf.push_str(&s.to_sql(dialect));
                buf.push(')');
            }
            Table { schema, table } => {
                buf.push_table_name(schema, table);
            }
        }
        if let Some(alias) = &self.alias {
            buf.push_str(" AS ");
            buf.push_quoted(alias);
        }
        buf.push_sql(&self.criteria, dialect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let j = Join {
            typ: JoinType::Inner,
            table: JoinTable::Table {
                schema: None,
                table: "foo".to_string(),
            },
            alias: Some("bar".to_string()),
            criteria: Criteria::On(Where::raw("bar.id = parent.bar_id".to_string())),
        };
        assert_eq!(
            j.to_sql(Dialect::Postgres),
            r#"JOIN "foo" AS "bar" ON bar.id = parent.bar_id"#
        );
    }

    #[test]
    fn test_builder() {
        let j = Join::new("table")
            .alias("bar")
            .on_raw("bar.id = parent.bar_id");
        assert_eq!(
            j.to_sql(Dialect::Postgres),
            r#"JOIN "table" AS "bar" ON bar.id = parent.bar_id"#
        );
    }
}
