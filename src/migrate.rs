use std::collections::HashMap;

use crate::query::{AlterTable, Update};
use anyhow::Result;

use crate::query::AlterAction;
use crate::query::CreateIndex;
use crate::query::CreateTable;
use crate::query::DropTable;
use crate::schema::Schema;
use crate::{Dialect, ToSql};

#[derive(Debug, Clone, Default)]
pub struct MigrationOptions {
    pub debug: bool,
    pub allow_destructive: bool,
}

pub fn migrate(current: Schema, desired: Schema, options: &MigrationOptions) -> Result<Migration> {
    let current_tables = current
        .tables
        .iter()
        .map(|t| (&t.name, t))
        .collect::<HashMap<_, _>>();
    let desired_tables = desired
        .tables
        .iter()
        .map(|t| (&t.name, t))
        .collect::<HashMap<_, _>>();

    let mut debug_results = vec![];
    let mut statements = Vec::new();
    // new tables
    for (_name, table) in desired_tables
        .iter()
        .filter(|(name, _)| !current_tables.contains_key(*name))
    {
        let statement = Statement::CreateTable(CreateTable::from_table(table));
        statements.push(statement);
    }

    // alter existing tables
    for (name, desired_table) in desired_tables
        .iter()
        .filter(|(name, _)| current_tables.contains_key(*name))
    {
        let current_table = current_tables[name];
        let current_columns = current_table
            .columns
            .iter()
            .map(|c| (&c.name, c))
            .collect::<HashMap<_, _>>();
        // add columns
        let mut actions = vec![];
        for desired_column in desired_table.columns.iter() {
            if let Some(current) = current_columns.get(&desired_column.name) {
                if current.nullable != desired_column.nullable {
                    actions.push(AlterAction::set_nullable(
                        desired_column.name.clone(),
                        desired_column.nullable,
                    ));
                }
                if !desired_column.typ.lossy_eq(&current.typ) {
                    actions.push(AlterAction::set_type(
                        desired_column.name.clone(),
                        desired_column.typ.clone(),
                    ));
                };
                if desired_column.constraint != current.constraint {
                    if let Some(c) = &desired_column.constraint {
                        let name = desired_column.name.clone();
                        actions.push(AlterAction::add_constraint(&desired_table.name, name, c.clone()));
                    }
                }
            } else {
                // add the column can be in 1 step if the column is nullable
                if desired_column.nullable {
                    actions.push(AlterAction::AddColumn {
                        column: desired_column.clone(),
                    });
                } else {
                    let mut nullable = desired_column.clone();
                    nullable.nullable = true;
                    statements.push(Statement::AlterTable(AlterTable {
                        schema: desired_table.schema.clone(),
                        name: desired_table.name.clone(),
                        actions: vec![AlterAction::AddColumn { column: nullable }],
                    }));
                    statements.push(Statement::Update(
                        Update::new(name)
                            .set(
                                &desired_column.name,
                                "/* TODO set a value before setting the column to null */",
                            )
                            .where_(crate::query::Where::raw("true")),
                    ));
                    statements.push(Statement::AlterTable(AlterTable {
                        schema: desired_table.schema.clone(),
                        name: desired_table.name.clone(),
                        actions: vec![AlterAction::AlterColumn {
                            name: desired_column.name.clone(),
                            action: crate::query::AlterColumnAction::SetNullable(false),
                        }],
                    }));
                }
            }
        }
        if actions.is_empty() {
            debug_results.push(DebugResults::TablesIdentical(name.to_string()));
        } else {
            statements.push(Statement::AlterTable(AlterTable {
                schema: desired_table.schema.clone(),
                name: desired_table.name.clone(),
                actions,
            }));
        }
    }

    for (_name, current_table) in current_tables
        .iter()
        .filter(|(name, _)| !desired_tables.contains_key(*name))
    {
        if options.allow_destructive {
            statements.push(Statement::DropTable(DropTable {
                schema: current_table.schema.clone(),
                name: current_table.name.clone(),
            }));
        } else {
            debug_results.push(DebugResults::SkippedDropTable(current_table.name.clone()));
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

impl Migration {
    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    pub fn set_schema(&mut self, schema_name: &str) {
        for statement in &mut self.statements {
            statement.set_schema(schema_name);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    CreateTable(CreateTable),
    CreateIndex(CreateIndex),
    AlterTable(AlterTable),
    DropTable(DropTable),
    Update(Update),
}

impl Statement {
    pub fn set_schema(&mut self, schema_name: &str) {
        match self {
            Statement::CreateTable(s) => {
                s.schema = Some(schema_name.to_string());
            }
            Statement::AlterTable(s) => {
                s.schema = Some(schema_name.to_string());
            }
            Statement::DropTable(s) => {
                s.schema = Some(schema_name.to_string());
            }
            Statement::CreateIndex(s) => {
                s.schema = Some(schema_name.to_string());
            }
            Statement::Update(s) => {
                s.schema = Some(schema_name.to_string());
            }
        }
    }

    pub fn table_name(&self) -> &str {
        match self {
            Statement::CreateTable(s) => &s.name,
            Statement::AlterTable(s) => &s.name,
            Statement::DropTable(s) => &s.name,
            Statement::CreateIndex(s) => &s.table,
            Statement::Update(s) => &s.table,
        }
    }
}

impl ToSql for Statement {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        use Statement::*;
        match self {
            CreateTable(c) => c.write_sql(buf, dialect),
            CreateIndex(c) => c.write_sql(buf, dialect),
            AlterTable(a) => a.write_sql(buf, dialect),
            DropTable(d) => d.write_sql(buf, dialect),
            Update(u) => u.write_sql(buf, dialect),
        }
    }
}

#[derive(Debug)]
pub enum DebugResults {
    TablesIdentical(String),
    SkippedDropTable(String),
}

impl DebugResults {
    pub fn table_name(&self) -> &str {
        match self {
            DebugResults::TablesIdentical(name) => name,
            DebugResults::SkippedDropTable(name) => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Table;

    #[test]
    fn test_drop_table() {
        let empty_schema = Schema::default();
        let mut single_table_schema = Schema::default();
        let t = Table::new("new_table");
        single_table_schema.tables.push(t.clone());
        let mut allow_destructive_options = MigrationOptions::default();
        allow_destructive_options.allow_destructive = true;

        let mut migrations = migrate(
            single_table_schema,
            empty_schema,
            &allow_destructive_options,
        )
            .unwrap();

        let statement = migrations.statements.pop().unwrap();
        let expected_statement = Statement::DropTable(DropTable {
            schema: t.schema,
            name: t.name,
        });

        assert_eq!(statement, expected_statement);
    }

    #[test]
    fn test_drop_table_without_destructive_operations() {
        let empty_schema = Schema::default();
        let mut single_table_schema = Schema::default();
        let t = Table::new("new_table");
        single_table_schema.tables.push(t.clone());
        let options = MigrationOptions::default();

        let migrations = migrate(single_table_schema, empty_schema, &options).unwrap();
        assert!(migrations.statements.is_empty());
    }
}
