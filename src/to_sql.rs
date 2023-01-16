#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dialect {
    Postgres,
    Mysql,
    Sqlite,
}

pub trait ToSql {
    fn to_sql(&self, dialect: Dialect) -> String;
}