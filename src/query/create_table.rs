use crate::{Dialect, Table, TableColumn, ToSql};
use crate::util::table_name;

/// Create table action
#[derive(Debug)]
pub struct CreateTable {
    pub schema: Option<String>,
    pub name: String,
    pub columns: Vec<TableColumn>,
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
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(&table_name(self.schema.as_ref(), &self.name, None));
        sql.push_str(" (\n");
        let mut first = true;
        for column in &self.columns {
            if !first {
                sql.push_str("\n, ");
            }
            sql.push_str(&column.to_sql(dialect));
            first = false;
        }
        sql.push_str("\n)");
        sql
    }
}