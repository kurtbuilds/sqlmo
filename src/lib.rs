#![allow(unused)]

//
// Given a OpenAPI spec, a connection to a database, and the schema within that database,
// - build a diff
// - Execute SQL

pub mod schema;
pub mod migrate;
pub mod query;
mod to_sql;
mod util;

#[doc(inline)]
pub use migrate::{Migration, MigrationOptions, migrate};
#[doc(inline)]
pub use schema::{Schema, Table, Column, Type};
#[doc(inline)]
pub use to_sql::{ToSql, Dialect};
#[doc(inline)]
pub use query::{Select, Insert, CreateIndex, CreateTable, AlterTable};

