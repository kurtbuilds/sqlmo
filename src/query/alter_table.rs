use crate::{Dialect, Column, ToSql};
use crate::util::SqlExtension;

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
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use AlterAction::*;
        buf.push_str("ALTER TABLE ");
        buf.push_table_name(&self.schema, &self.name);
        match &self.action {
            AddColumn { column } => {
                buf.push_str(" ADD COLUMN ");
                buf.push_str(&column.to_sql(dialect));
            }
        }
    }
}