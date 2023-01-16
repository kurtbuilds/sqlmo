use std::collections::{HashMap};


use anyhow::Result;
use crate::query::AlterTable;

use crate::query::AlterAction;
use crate::query::CreateIndex;
use crate::query::CreateTable;
use crate::schema::{Schema};
use crate::{Dialect, ToSql};

#[derive(Debug, Clone, Default)]
pub struct MigrationOptions {
    pub debug: bool,
}


pub fn migrate(current: Schema, desired: Schema, _options: &MigrationOptions) -> Result<Migration> {
    let current_tables = current.tables.iter().map(|t| (&t.name, t)).collect::<HashMap<_, _>>();
    let desired_tables = desired.tables.iter().map(|t| (&t.name, t)).collect::<HashMap<_, _>>();

    let mut debug_results = vec![];
    let mut statements = Vec::new();
    // new tables
    for (_name, table) in desired_tables.iter().filter(|(name, _)| !current_tables.contains_key(*name)) {
        let statement = Statement::CreateTable(CreateTable::from_table(table));
        statements.push(statement);
    }

    // alter existing tables
    for (name, desired_table) in desired_tables.iter().filter(|(name, _)| current_tables.contains_key(*name)) {
        let current_table = current_tables[name];
        let current_columns = current_table.columns.iter().map(|c| (&c.name, c)).collect::<HashMap<_, _>>();
        // add columns
        let mut matches = true;
        for desired_column in desired_table.columns.iter().filter(|c| !current_columns.contains_key(&c.name)) {
            let statement = Statement::AlterTable(AlterTable {
                schema: desired_table.schema.clone(),
                name: desired_table.name.clone(),
                action: AlterAction::AddColumn {
                    column: desired_column.clone(),
                },
            });
            statements.push(statement);
            matches = false;
        }
        if matches {
            debug_results.push(DebugResults::TablesIdentical(name.to_string()));
        }
    }

    Ok(Migration {
        statements,
        debug_results,
    })
}

#[derive(Debug)]
pub struct Migration {
    pub statements: Vec<Statement>,
    pub debug_results: Vec<DebugResults>,
}


#[derive(Debug)]
pub enum Statement {
    CreateTable(CreateTable),
    CreateIndex(CreateIndex),
    AlterTable(AlterTable),
}

impl ToSql for Statement {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use Statement::*;
        match self {
            CreateTable(c) => c.write_sql(buf, dialect),
            CreateIndex(c) => c.write_sql(buf, dialect),
            AlterTable(a) => a.write_sql(buf, dialect),
        }
    }
}

#[derive(Debug)]
pub enum DebugResults {
    TablesIdentical(String)
}
