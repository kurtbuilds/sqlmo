use crate::{Dialect, ToSql, Type};
use crate::query::Expr;
use crate::schema::constraint::Constraint;
use crate::util::SqlExtension;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Column {
    pub name: String,
    pub typ: Type,
    pub nullable: bool,
    pub primary_key: bool,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub default: Option<Expr>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub constraint: Option<Constraint>
}


impl ToSql for Column {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_quoted(&self.name);
        buf.push(' ');
        buf.push_str(&self.typ.to_sql(dialect));
        if !self.nullable {
            buf.push_str(" NOT NULL");
        }
        if self.primary_key {
            buf.push_str(" PRIMARY KEY");
        }
        if let Some(default) = &self.default {
            buf.push_str(" DEFAULT ");
            buf.push_sql(default, dialect);
        }
        if let Some(constraint) = &self.constraint {
            buf.push(' ');
            buf.push_sql(constraint, dialect);
        }
    }
}