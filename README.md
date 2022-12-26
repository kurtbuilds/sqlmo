<div id="top"></div>

<p align="center">
<a href="https://github.com/kurtbuilds/sqldiff/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kurtbuilds/sqldiff.svg?style=flat-square" alt="GitHub Contributors" />
</a>
<a href="https://github.com/kurtbuilds/sqldiff/stargazers">
    <img src="https://img.shields.io/github/stars/kurtbuilds/sqldiff.svg?style=flat-square" alt="Stars" />
</a>
<a href="https://github.com/kurtbuilds/sqldiff/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/kurtbuilds/sqldiff/test.yaml?style=flat-square" alt="Build Status" />
</a>
<a href="https://crates.io/crates/sqldiff">
    <img src="https://img.shields.io/crates/d/sqldiff?style=flat-square" alt="Downloads" />
</a>
<a href="https://crates.io/crates/sqldiff">
    <img src="https://img.shields.io/crates/v/sqldiff?style=flat-square" alt="Crates.io" />
</a>

</p>

# `sqldiff`
`sqldiff` is a set of primitives to load SQL representations in a standardized form (`sqldiff::Schema`), calculate differences between 
schemas (`sqldiff::Migration`), and generate SQL to apply the migration (`sqldiff::Migration::to_sql`).

Currently built-in schema sources:
- Postgres
- OpenAPI 3.0 spec

Tools that want to auto-generate migrations should define a way to load their schema into `sqldiff::Schema` and then use `sqldiff` to
auto-generate migrations.

Current tools that support this:

- (`sqldiff`)[https://github.com/kurtbuilds/sqldiff]

If you use `sqldiff`, submit a PR to be added to this list.

```rust
#[tokio::main]
async fn main() {
    let url = std::env::var("DATABASE_URL").unwrap();
    let mut conn = sqlx::postgres::PgConnection::connect(&url).await?;
    let current = Schema::try_from_database(&mut conn, schema_name).await?;
    let end_state = Schema::new(); // Load your end-state by manually defining it, or building it from another source
    let migration = current.migrate_to(end_state, &sqldiff::Options::default());
    
    for statement in migration.statements {
        let statement = statement.prepare(schema_name);
        println!("{}", statement);
    }
}
```