pub(super) struct Schema {
    name: String,
    columns: Vec<String>,
}

pub(super) struct SchemaBuilder {
    name: String,
    columns: Vec<String>,
}
impl Schema {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: vec![],
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        self
    }
    pub const fn columns(&self) -> &Vec<String> {
        &self.columns
    }
}

#[cfg(test)]
mod test {}
