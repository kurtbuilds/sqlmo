/// Defines structs and functions for representing SQL database schemas.
pub mod schema;
/// Defines structs and functions for auto-generating migrations.
pub mod migrate;
/// Defines structs and functions for representing SQL queries.
pub mod query;

mod to_sql;
pub mod util;

#[doc(inline)]
pub use migrate::{Migration, MigrationOptions, migrate};
#[doc(inline)]
pub use schema::{Schema, Table, Column, Type, Constraint};
#[doc(inline)]
pub use to_sql::{ToSql, Dialect};
#[doc(inline)]
pub use query::{Select, Insert, CreateIndex, CreateTable, AlterTable, Operation, Expr};

