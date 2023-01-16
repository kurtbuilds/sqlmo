use crate::{Dialect, Column, ToSql};
use crate::util::table_name;

/// Alter table action
#[derive(Debug)]
pub enum AlterAction {
    AddColumn {
        column: Column,
    },
}

#[derive(Debug)]
pub struct AlterTable {
    pub schema: Option<String>,
    pub name: String,
    pub action: AlterAction,
}

impl ToSql for AlterTable {
    fn to_sql(&self, dialect: Dialect) -> String {
        use AlterAction::*;
        let mut sql = String::new();
        sql.push_str("ALTER TABLE ");
        sql.push_str(&table_name(self.schema.as_ref(), &self.name, None));
        match &self.action {
            AddColumn { column } => {
                sql.push_str(" ADD COLUMN ");
                sql.push_str(&column.to_sql(dialect));
            }
        }
        sql
    }
}