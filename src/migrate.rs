use std::collections::HashMap;

use crate::query::{AlterTable, Update};
use anyhow::Result;

use crate::query::AlterAction;
use crate::query::CreateIndex;
use crate::query::CreateTable;
use crate::query::DropTable;
use crate::schema::{Constraint, Schema};
use crate::{Dialect, ToSql};
use topo_sort::{SortResults, TopoSort};

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
                if desired_column.constraint.is_some() && current.constraint.is_none() {
                    if let Some(c) = &desired_column.constraint {
                        let name = desired_column.name.clone();
                        actions.push(AlterAction::add_constraint(
                            &desired_table.name,
                            name,
                            c.clone(),
                        ));
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

    // Sort statements topologically based on foreign key dependencies
    let sorted_statements = topologically_sort_statements(&statements, &desired_tables);

    Ok(Migration {
        statements: sorted_statements,
        debug_results,
    })
}

/// Topologically sorts the migration statements based on foreign key dependencies
fn topologically_sort_statements(
    statements: &[Statement],
    tables: &HashMap<&String, &crate::schema::Table>,
) -> Vec<Statement> {
    // First, extract create table statements
    let create_statements: Vec<_> = statements
        .iter()
        .filter(|s| matches!(s, Statement::CreateTable(_)))
        .collect();

    if create_statements.is_empty() {
        // If there are no create statements, just return the original
        return statements.to_vec();
    }

    // Build a map of table name to index in the statements array
    let mut table_to_index = HashMap::new();
    for (i, stmt) in create_statements.iter().enumerate() {
        if let Statement::CreateTable(create) = stmt {
            table_to_index.insert(create.name.clone(), i);
        }
    }

    // Set up topological sort
    let mut topo_sort = TopoSort::new();

    // Find table dependencies and add them to topo_sort
    for stmt in &create_statements {
        if let Statement::CreateTable(create) = stmt {
            let table_name = &create.name;
            let mut dependencies = Vec::new();

            // Get the actual table from the tables map
            if let Some(table) = tables.values().find(|t| &t.name == table_name) {
                // Check all columns for foreign key constraints
                for column in &table.columns {
                    if let Some(Constraint::ForeignKey(fk)) = &column.constraint {
                        dependencies.push(fk.table.clone());
                    }
                }
            }

            // Add this table and its dependencies to the topo_sort
            dbg!(table_name, &dependencies);
            topo_sort.insert(table_name.clone(), dependencies);
        }
    }

    // Perform the sort
    let table_order = match topo_sort.into_vec_nodes() {
        SortResults::Full(nodes) => nodes,
        SortResults::Partial(nodes) => {
            // Return partial results even if there's a cycle
            nodes
        }
    };

    // First create a sorted list of CREATE TABLE statements
    let mut sorted_statements = Vec::new();
    for table_name in &table_order {
        if let Some(&idx) = table_to_index.get(table_name) {
            sorted_statements.push(create_statements[idx].clone());
        }
    }

    // Add remaining statements (non-create-table) in their original order
    for stmt in statements {
        if !matches!(stmt, Statement::CreateTable(_)) {
            sorted_statements.push(stmt.clone());
        }
    }

    sorted_statements
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

    use crate::schema::{Column, Constraint, ForeignKey};
    use crate::Table;
    use crate::Type;

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

    #[test]
    fn test_topological_sort_statements() {
        let empty_schema = Schema::default();
        let mut schema_with_tables = Schema::default();

        // Create dependent tables: User depends on Team
        let team_table = Table::new("team").column(Column {
            name: "id".to_string(),
            typ: Type::I32,
            nullable: false,
            primary_key: true,
            default: None,
            constraint: None,
        });

        let user_table = Table::new("user")
            .column(Column {
                name: "id".to_string(),
                typ: Type::I32,
                nullable: false,
                primary_key: true,
                default: None,
                constraint: None,
            })
            .column(Column {
                name: "team_id".to_string(),
                typ: Type::I32,
                nullable: false,
                primary_key: false,
                default: None,
                constraint: Some(Constraint::ForeignKey(ForeignKey {
                    table: "team".to_string(),
                    columns: vec!["id".to_string()],
                })),
            });

        schema_with_tables.tables.push(user_table);
        schema_with_tables.tables.push(team_table);

        let options = MigrationOptions::default();

        // Generate migration
        let migration = migrate(empty_schema, schema_with_tables, &options).unwrap();

        // Check that team table is created before user table
        let team_index = migration
            .statements
            .iter()
            .position(|s| {
                if let Statement::CreateTable(create) = s {
                    create.name == "team"
                } else {
                    false
                }
            })
            .unwrap();

        let user_index = migration
            .statements
            .iter()
            .position(|s| {
                if let Statement::CreateTable(create) = s {
                    create.name == "user"
                } else {
                    false
                }
            })
            .unwrap();

        assert!(
            team_index < user_index,
            "Team table should be created before User table"
        );
    }
}
