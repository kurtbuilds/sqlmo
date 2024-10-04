use crate::util::SqlExtension;
use crate::{Column, Dialect, Table, ToSql};

/// Create table action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTable {
    pub schema: Option<String>,
    pub name: String,
    pub columns: Vec<Column>,
}

impl CreateTable {
    pub fn from_table(table: &Table) -> CreateTable {
        CreateTable {
            schema: table.schema.clone(),
            name: table.name.clone(),
            columns: table.columns.clone(),
        }
    }
}

impl ToSql for CreateTable {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        buf.push_str("CREATE TABLE ");
        buf.push_table_name(&self.schema, &self.name);
        buf.push_str(" (\n");
        buf.push_sql_sequence(&self.columns, ",\n", dialect);
        buf.push_str("\n)");
    }
}
