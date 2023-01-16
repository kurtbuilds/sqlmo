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
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        sql.push_str(&self.name);
        sql.push(' ');
        sql.push_str(&self.typ.to_sql(dialect));
        if !self.nullable {
            sql.push_str(" NOT NULL");
        }
        if self.primary_key {
            sql.push_str(" PRIMARY KEY");
        }
        sql
    }
}