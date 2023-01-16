use crate::{Dialect, Select, ToSql};
use crate::query::Where;
use crate::util::SqlExtension;

#[derive(Debug, Clone)]
pub enum JoinTable {
    Select(Select),
    Table { schema: Option<String>, table: String },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone)]
pub struct Join {
    pub typ: JoinType,
    pub table: JoinTable,
    pub alias: Option<String>,
    pub on: Where,
}


impl ToSql for Join {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use JoinType::*;
        use JoinTable::*;
        match self.typ {
            Inner => buf.push_str("INNER JOIN "),
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
            buf.push_str(alias);
        }
        buf.push_str(" ON ");
        buf.push_str(&self.on.to_sql(dialect));
    }
}
