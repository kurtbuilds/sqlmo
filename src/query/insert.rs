use crate::{Dialect, ToSql};
use crate::util::{quote, table_name};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OnConflict {
    Ignore,
    Abort,
    // Replace,
}

impl Default for OnConflict {
    fn default() -> Self {
        OnConflict::Abort
    }
}

pub struct Insert {
    pub schema: Option<String>,
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<String>>,
    pub on_conflict: OnConflict,
    pub returning: Vec<String>,
}

fn query_start(dialect: Dialect, on_conflict: OnConflict) -> String {
    use Dialect::*;
    use OnConflict::*;
    if dialect == Sqlite {
        match on_conflict {
            Ignore => "INSERT OR IGNORE INTO ".to_string(),
            Abort => "INSERT OR ABORT INTO ".to_string(),
        }
    } else {
        "INSERT INTO ".to_string()
    }
}

impl ToSql for Insert {
    fn to_sql(&self, dialect: Dialect) -> String {
        let mut q = query_start(dialect, self.on_conflict);
        q.push_str(&table_name(self.schema.as_ref(), &self.table, None));
        q.push_str(" (");
        let mut first = true;
        for column in &self.columns {
            if !first {
                q.push_str(", ");
            }
            q.push_str(quote(column).as_str());
            first = false;
        }
        q.push_str(") VALUES ");
        let mut first = true;
        for row in &self.values {
            if !first {
                q.push_str(", ");
            }
            q.push_str("(");
            let mut first = true;
            for value in row {
                if !first {
                    q.push_str(", ");
                }
                q.push_str(value.as_str());
                first = false;
            }
            q.push_str(")");
            first = false;
        }
        if !self.returning.is_empty() {
            q.push_str(" RETURNING ");
            let mut first = true;
            for column in &self.returning {
                if !first {
                    q.push_str(", ");
                }
                q.push_str(quote(column).as_str());
                first = false;
            }
        }
        q
    }
}