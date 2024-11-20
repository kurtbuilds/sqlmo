use crate::{Dialect, Insert, Select, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CteQuery {
    Select(Select),
    Insert(Insert),
    Raw(String),
}

impl ToSql for CteQuery {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            CteQuery::Select(s) => s.write_sql(buf, dialect),
            CteQuery::Insert(i) => i.write_sql(buf, dialect),
            CteQuery::Raw(s) => buf.push_str(s),
        }
    }
}

impl Into<CteQuery> for Select {
    fn into(self) -> CteQuery {
        CteQuery::Select(self)
    }
}

impl Into<CteQuery> for Insert {
    fn into(self) -> CteQuery {
        CteQuery::Insert(self)
    }
}

impl Into<CteQuery> for String {
    fn into(self) -> CteQuery {
        CteQuery::Raw(self)
    }
}

/// Common table expression
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cte {
    pub name: String,
    pub query: CteQuery,
}

impl Cte {
    pub fn new(name: impl Into<String>, query: impl Into<CteQuery>) -> Self {
        Self {
            name: name.into(),
            query: query.into(),
        }
    }
}

impl ToSql for Cte {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str(&self.name);
        buf.push_str(" AS (");
        buf.push_str(&self.query.to_sql(dialect));
        buf.push(')');
    }
}
