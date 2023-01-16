#![allow(unused)]

//
// Given a OpenAPI spec, a connection to a database, and the schema within that database,
// - build a diff
// - Execute SQL

mod schema;
mod migrate;
mod query;
mod to_sql;
mod util;

pub use migrate::*;
pub use schema::*;
pub use to_sql::*;
pub use query::*;

