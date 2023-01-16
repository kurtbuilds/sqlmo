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
`sqlmo` is a set of primitives to load SQL representations in a standardized form (`sqlmo::Schema`), calculate differences between 
schemas (`sqlmo::Migration`), and generate SQL to apply the migration (`sqlmo::Migration::to_sql`).

Currently built-in schema sources:
- Postgres
- OpenAPI 3.0 spec

Tools that want to auto-generate migrations should define a way to load their schema into `sqlmo::Schema` and then use `sqlmo` to
auto-generate migrations.

Current tools that support this:

- [`ormlite`](https://github.com/kurtbuilds/ormlite)

If you use this library, submit a PR to be added to this list.

```rust
#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut conn = sqlx::postgres::PgConnection::connect(&url).await?;
    let current = Schema::try_from_database(&mut conn, schema_name).await?;
    let end_state = Schema::new(); // Load your end-state by manually defining it, or building it from another source
    let migration = current.migrate_to(end_state, &sqlmo::Options::default());
    
    for statement in migration.statements {
        let statement = statement.prepare(schema_name);
        println!("{}", statement);
    }
}
```