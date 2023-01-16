<div id="top"></div>

<p align="center">
<a href="https://github.com/kurtbuilds/sqlmo/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/sqlmo.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/sqlmo/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/sqlmo.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/sqlmo/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/kurtbuilds/sqlmo/test.yaml?style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/sqlmo">
    <img src="https://img.shields.io/crates/d/sqlmo?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/sqlmo">
    <img src="https://img.shields.io/crates/v/sqlmo?style=flat-square" alt="Crates.io" />
</a>

</p>

# `sqlmo`
`sqlmo` is a set of primitives to represent SQL tables and queries. Use these primitives to:
- **Auto-generate migrations**: Load SQL representations in a standardized form (`sqlmo::Schema`), calculate differences between 
schemas (`sqlmo::Migration`), and generate SQL to apply the migration (`sqlmo::Migration::to_sql`).
- **Build SQL queries**: Represent SQL queries in a data model, to create APIs for query generation. Then, generate the
SQL query. *Note: this library does not support parsing SQL queries (yet).*

For auto-generating migrations, there are a few built-in schema sources:
- Postgres
- OpenAPI 3.0 spec

If you need another source, you should define a way to build a `sqlmo::Schema` from your data source, then use `sqlmo` 
to auto-generate migrations.

Current tools that support this:

- [`ormlite`](https://github.com/kurtbuilds/ormlite)

If you use this library, submit a PR to be added to this list.

## Usage

This example reads the schema from a postgres database, defines an empty schema (which should be filled in),
and then computes the migration to apply to the database.

```rust
#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut conn = sqlx::postgres::PgConnection::connect(&url).await?;
    let current = Schema::try_from_database(&mut conn, schema_name).await?;
    let end_state = Schema::new(); // Load your end-state by manually defining it, or building it from another source
    let migration = current.migrate_to(end_state, &sqlmo::Options::default());
    
    for statement in migration.statements {
        let statement = statement.to_sql(Dialect::Postgres);
        println!("{}", statement);
    }
}
```