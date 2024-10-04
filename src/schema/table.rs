use crate::schema::index::Index;
use crate::schema::column::Column;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Table {
    pub schema: Option<String>,
    pub name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
}

impl Table {
    pub fn primary_key(&self) -> Option<&Column> {
        self.columns.iter().find(|c| c.primary_key)
    }

    pub fn new(name: &str) -> Table {
        Table {
            schema: None,
            name: name.to_string(),
            columns: vec![],
            indexes: vec![],
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn index(mut self, index: Index) -> Self {
        self.indexes.push(index);
        self
    }

    pub fn schema(mut self, schema: &str) -> Self {
        self.schema = Some(schema.to_string());
        self
    }
}
