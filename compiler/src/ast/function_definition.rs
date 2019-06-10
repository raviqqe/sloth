use super::expression::Expression;
use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    arguments: Vec<String>,
    body: Expression,
    type_: types::Function,
}

impl FunctionDefinition {
    pub fn new(
        name: String,
        arguments: Vec<String>,
        body: Expression,
        type_: types::Function,
    ) -> Self {
        Self {
            name,
            arguments,
            body,
            type_,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }
}