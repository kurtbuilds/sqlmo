#![cfg(feature = "openapi")]

use std::fs::File;
use openapiv3::OpenAPI;
use anyhow::Result;
use sqlmo::{MigrationOptions, Schema, Dialect, ToSql};

const OPENAPI_YAML_FILEPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/spec/openapi.yaml");

#[test]
pub fn test_run_sql_migration() -> Result<()> {
    let yaml = File::open(OPENAPI_YAML_FILEPATH)?;
    let spec: OpenAPI = serde_yaml::from_reader(yaml)?;
    let current = Schema::default();
    let mut desired = Schema::try_from(spec)?;
    desired.name_schema("public");
    let migration = current.migrate_to(desired, &MigrationOptions { debug: false })?;

    assert_eq!(migration.statements.len(), 5);
    for statement in migration.statements {
        let statement = statement.to_sql(Dialect::Postgres);
        assert!(statement.starts_with(r#"CREATE TABLE "public".""#));
    }
    Ok(())
}