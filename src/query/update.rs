use crate::query::Cte;
use crate::query::Where;
use crate::util::SqlExtension;
use crate::{Dialect, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub ctes: Vec<Cte>,
    pub schema: Option<String>,
    pub table: String,
    pub values: Vec<(String, String)>,
    pub where_: Where,
}

impl Update {
    pub fn new(table: &str) -> Self {
        Self {
            ctes: vec![],
            schema: None,
            table: table.to_string(),
            values: vec![],
            where_: Where::And(vec![]),
        }
    }

    pub fn set(mut self, column: &str, value: &str) -> Self {
        self.values.push((column.to_string(), value.to_string()));
        self
    }

    pub fn where_(mut self, where_: Where) -> Self {
        match self.where_ {
            Where::And(ref mut v) => v.push(where_),
            _ => self.where_ = Where::And(vec![self.where_, where_]),
        }
        self
    }
}

impl ToSql for Update {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        if !self.ctes.is_empty() {
            buf.push_str("WITH ");
            buf.push_sql_sequence(&self.ctes, ", ", dialect);
            buf.push(' ');
        }
        buf.push_str("UPDATE ");
        buf.push_table_name(&self.schema, &self.table);
        buf.push_str(" SET ");
        for (i, (column, value)) in self.values.iter().enumerate() {
            if i > 0 {
                buf.push_str(", ");
            }
            buf.push_quoted(column);
            buf.push_str(" = ");
            buf.push_str(value);
        }
        if !self.where_.is_empty() {
            buf.push_str(" WHERE ");
            self.where_.write_sql(buf, dialect);
        }
    }
}
