#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dialect {
    Postgres,
    Mysql,
    Sqlite,
}

pub trait ToSql {
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut buf = String::new();
        self.write_sql(&mut buf, dialect);
        buf
    }

    fn write_sql(&self, buf: &mut String, dialect: Dialect);
}