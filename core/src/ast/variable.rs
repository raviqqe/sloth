use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    name: String,
}

impl Variable {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rename_variables(&self, names: &HashMap<String, String>) -> Self {
        match names.get(self.name.as_str()) {
            Some(name) => Self::new(name),
            None => self.clone(),
        }
    }
}