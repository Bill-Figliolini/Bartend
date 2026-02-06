pub(super) struct Schema {
    name: String,
    columns: Vec<String>,
}

pub(super) struct SchemaBuilder {
    name: String,
    columns: Vec<String>,
}

impl SchemaBuilder {
    pub fn column(mut self, column: String) -> Self {
        self.columns.push(column);
        Self {
            name: self.name,
            columns: self.columns,
        }
    }

    pub fn build(self) -> Schema {
        Schema {
            name: self.name,
            columns: self.columns,
        }
    }
}

impl Schema {
    pub fn new(name: &str) -> SchemaBuilder {
        SchemaBuilder {
            name: name.to_string(),
            columns: vec![],
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn columns(&self) -> &Vec<String> {
        &self.columns
    }
}

#[cfg(test)]
mod test {}
