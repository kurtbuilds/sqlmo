#![cfg(feature = "openapi")]

use std::fs::File;
use anyhow::Result;
use openapiv3::OpenAPI;
use sqlx::{Connection, Executor};
use sqlmo::Schema;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[clap(long)]
    dry: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 1. First, read the current schema from the database
    let schema_name = "public";
    let mut conn = sqlx::postgres::PgConnection::connect(&url).await?;

    let current = Schema::try_from_database(&mut conn, schema_name).await?;

    // 2. Next, define the desired schema somehow. This can come from reading another database,
    // analyzing an openapi spec (as here), manually constructing the schema, or using a library for
    // your ORM to analyze your project structure. See `https://github.com/kurtbuilds/ormlite` for
    // an example of the latter.
    let openapi_file = std::env::var("OPENAPI_FILE").expect("OPENAPI_FILE must be set");
    let spec: OpenAPI = serde_yaml::from_reader(File::open(openapi_file)?)?;

    let desired = Schema::try_from(spec)?;

    // 3. Finally, generate the SQL to transform the current schema into the desired schema.
    let migration = current.migrate_to(desired, Options {})?;
    if migration.statements.is_empty() {
        println!("No migration needed");
        return Ok(())
    }
    let dry = !std::env::var("RUN").map(|s| s == "1").unwrap_or(false);
    if dry {
        println!("Dry run. Set RUN=1 to execute.");
    }
    for statement in migration.statements {
        let statement = statement.prepare(schema_name);
        println!("{}", statement);
        if !dry {
            conn.execute(&*statement).await?;
        }
    }
    Ok(())
}