use crate::{Dialect, Select, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CteQuery {
    Select(Select),
    Raw(String),
}

impl ToSql for CteQuery {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            CteQuery::Select(s) => s.write_sql(buf, dialect),
            CteQuery::Raw(s) => buf.push_str(s),
        }
    }
}

/// Common table expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cte {
    pub name: String,
    pub query: CteQuery,
}

impl ToSql for Cte {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str(&self.name);
        buf.push_str(" AS (");
        buf.push_str(&self.query.to_sql(dialect));
        buf.push(')');
    }
}
