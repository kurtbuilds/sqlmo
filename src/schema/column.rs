use crate::{Type, ToSql, Dialect};

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub typ: Type,
    pub nullable: bool,
    pub primary_key: bool,
    pub default: Option<String>,
}


impl ToSql for Column {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str(&self.name);
        buf.push(' ');
        buf.push_str(&self.typ.to_sql(dialect));
        if !self.nullable {
            buf.push_str(" NOT NULL");
        }
        if self.primary_key {
            buf.push_str(" PRIMARY KEY");
        }
    }
}