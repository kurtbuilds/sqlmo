use crate::{Dialect, Select, ToSql};

pub struct Union {
    pub all: bool,
    pub queries: Vec<Select>,
}

impl ToSql for Union {
    fn write_sql(&self, buf: &mut String, dialect: Dialect) {
        let all = if self.all { "ALL " } else { "" };
        if self.queries.is_empty() {
            return;
        }
        let first = self.queries.iter().next().unwrap();
        first.write_sql(buf, dialect);
        for q in self.queries.iter().skip(1) {
            buf.push_str(" UNION ");
            buf.push_str(all);
            q.write_sql(buf, dialect);
        }
    }
}