use std::collections::{HashMap, HashSet};
use std::ops::Sub;

use anyhow::Result;

pub use model::*;
use crate::schema::{Column, Schema, Table, Type};

mod model;

/// This is currently empty.
#[derive(Debug, Clone, Default)]
pub struct Options {
    pub debug: bool,
}

impl Schema {
    pub fn migrate_to(self, desired: Schema, options: &Options) -> Result<Migration> {
        migrate(self, desired, options)
    }
}

fn migrate(current: Schema, desired: Schema, options: &Options) -> Result<Migration> {
    let current_tables = current.tables.iter().map(|t| (&t.name, t)).collect::<HashMap<_, _>>();
    let desired_tables = desired.tables.iter().map(|t| (&t.name, t)).collect::<HashMap<_, _>>();

    let mut debug_results = vec![];
    let mut statements = Vec::new();
    // new tables
    for (name, table) in desired_tables.iter().filter(|(name, _)| !current_tables.contains_key(*name)) {
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
                name: desired_table.name.clone(),
                alter_action: AlterAction::AddColumn {
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
