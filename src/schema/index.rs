#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Index {
    pub name: String,
    pub columns: Vec<String>,
}
