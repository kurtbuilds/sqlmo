#![cfg(feature = "openapi")]

use std::fs::File;
use openapiv3::OpenAPI;
use anyhow::Result;
use sqldiff::{Schema, Options};

const OPENAPI_YAML_FILEPATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/spec/openapi.yaml");

#[test]
pub fn test_run_sql_migration() -> Result<()> {
    let yaml = File::open(OPENAPI_YAML_FILEPATH)?;
    let spec: OpenAPI = serde_yaml::from_reader(yaml)?;
    let current = Schema::new();
    let desired = Schema::try_from(spec)?;
    let migration = current.migrate_to(desired, &Options { debug: false })?;
    assert_eq!(migration.statements.len(), 5);
    for statement in migration.statements {
        let statement = statement.prepare("public");
        assert!(statement.starts_with(r#"CREATE TABLE "public".""#));
    }
    Ok(())
}