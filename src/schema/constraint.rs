use crate::{Dialect, ToSql};
use crate::util::SqlExtension;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForeignKey {
    pub table: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Constraint {
    ForeignKey(ForeignKey),
}

impl Constraint {
    pub fn foreign_key(table: String, columns: Vec<String>) -> Self {
        Constraint::ForeignKey(ForeignKey { table, columns })
    }

    pub fn name(&self) -> &str {
        match self {
            Constraint::ForeignKey(fk) => &fk.table,
        }
    }
}

impl ToSql for ForeignKey {
    fn write_sql(&self, buf: &mut String, _dialect: Dialect) {
        buf.push_str("REFERENCES ");
        buf.push_quoted(&self.table);
        if !self.columns.is_empty() {
            buf.push('(');
            buf.push_quoted_sequence(&self.columns, ", ");
            buf.push(')');
        }
    }
}

impl ToSql for Constraint {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        match self {
            Constraint::ForeignKey(fk) => fk.write_sql(buf, dialect),
        }
    }
}