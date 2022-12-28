use crate::schema::{Table, Type, Column};
use crate::schema::ToSql;


/// Create table action
#[derive(Debug)]
pub struct CreateTable {
    pub name: String,
    pub columns: Vec<Column>,
}

/// Create index action for a table
#[derive(Debug)]
pub struct CreateIndex {
    pub name: String,
    pub table_name: String,
}


/// Alter table action
#[derive(Debug)]
pub enum AlterAction {
    AddColumn {
        column: Column,
    },
}

#[derive(Debug)]
pub struct AlterTable {
    pub name: String,
    pub alter_action: AlterAction,
}


#[derive(Debug)]
pub enum Statement {
    CreateTable(CreateTable),
    CreateIndex(CreateIndex),
    AlterTable(AlterTable),
}

#[derive(Debug)]
pub struct Migration {
    pub statements: Vec<Statement>
}

impl ToSql for Column {
    fn to_sql(&self) -> String {
        let mut sql = String::new();
        sql.push_str(&self.name);
        sql.push(' ');
        sql.push_str(&self.typ.to_sql());
        if !self.nullable {
            sql.push_str(" NOT NULL");
        }
        if self.primary_key {
            sql.push_str(" PRIMARY KEY");
        }
        sql
    }
}

impl CreateTable {
    pub fn from_table(table: &Table) -> CreateTable {
        CreateTable {
            name: table.name.clone(),
            columns: table.columns.clone(),
        }
    }
}

fn quote(s: &str) -> String {
    if s.contains('"') {
        panic!("Cannot quote string with double quotes");
    }
    format!("\"{}\"", s)
}

fn table_name(schema: &str, table: &str) -> String {
    format!("{}.{}", quote(schema), quote(table))
}

impl Statement {
    pub fn prepare(&self, schema_name: &str) -> String {
        use Statement::*;
        match self {
            CreateTable(create_table) => {
                let mut sql = String::new();
                sql.push_str("CREATE TABLE ");
                sql.push_str(&table_name(schema_name, &create_table.name));
                sql.push_str(" (\n");
                let mut first = true;
                for column in &create_table.columns {
                    if !first {
                        sql.push_str("\n, ");
                    }
                    sql.push_str(&column.to_sql());
                    first = false;
                }
                sql.push_str("\n)");
                sql
            }
            CreateIndex(create_index) => unimplemented!(),
            AlterTable(alter_table) => {
                let mut sql = String::new();
                sql.push_str("ALTER TABLE ");
                sql.push_str(&table_name(schema_name, &alter_table.name));
                match &alter_table.alter_action {
                    AlterAction::AddColumn { column } => {
                        sql.push_str(" ADD COLUMN ");
                        sql.push_str(&column.to_sql());
                    }
                }
                sql
            }
        }
    }
}