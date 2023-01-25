use crate::{Dialect, ToSql};
use crate::util::SqlExtension;

#[derive(Debug)]
pub struct CreateSchema {
    pub name: String,
    pub if_not_exists: bool,
}

impl CreateSchema {
    pub fn new(name: &str) -> Self {
        CreateSchema {
            name: name.to_string(),
            if_not_exists: false,
        }
    }

    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }
}

impl ToSql for CreateSchema {
    fn write_sql(&self, buf: &mut String, _: Dialect) {
        buf.push_str("CREATE SCHEMA ");
        if self.if_not_exists {
            buf.push_str(" IF NOT EXISTS ");
        }
        buf.push_quoted(&self.name);
    }
}
