use crate::ast::*;
use crate::types::Type;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModuleEnvironmentCreator {}

impl ModuleEnvironmentCreator {
    pub fn new() -> Arc<Self> {
        Self {}.into()
    }

    pub fn create(&self, module: &Module) -> HashMap<String, Type> {
        let mut variables = HashMap::<String, Type>::new();

        for import in module.imports() {
            for (name, type_) in import.module_interface().variables() {
                variables.insert(name.into(), type_.clone());
            }
        }

        for declaration in module.import_foreigns() {
            variables.insert(declaration.name().into(), declaration.type_().clone());
        }

        for definition in module.definitions() {
            match definition {
                Definition::FunctionDefinition(function_definition) => {
                    variables.insert(
                        function_definition.name().into(),
                        function_definition.type_().clone(),
                    );
                }
                Definition::VariableDefinition(variable_definition) => {
                    variables.insert(
                        variable_definition.name().into(),
                        variable_definition.type_().clone(),
                    );
                }
            }
        }

        variables
    }
}
