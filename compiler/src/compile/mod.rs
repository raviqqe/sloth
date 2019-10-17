mod desugar;
mod error;
mod expression_compiler;
mod free_variable_finder;
mod module_compiler;
mod name_generator;
mod type_compiler;
mod type_inference;

use crate::ast;
use desugar::{desugar_with_types, desugar_without_types};
use error::CompileError;
use module_compiler::ModuleCompiler;
use std::collections::HashMap;
use std::io::Write;
use type_inference::infer_types;

pub fn compile(
    module_name: &str,
    module: &ast::Module,
    destination: &str,
) -> Result<(), CompileError> {
    std::fs::File::create(destination)?.write_all(core::compile::compile(
        &rename_top_level_variables(
            &ModuleCompiler::new().compile(&desugar_with_types(&infer_types(
                &desugar_without_types(module),
            )?))?,
            module_name,
        ),
    )?)?;

    Ok(())
}

fn rename_top_level_variables(module: &core::ast::Module, module_name: &str) -> core::ast::Module {
    let mut names = HashMap::new();

    for definition in module.definitions() {
        names.insert(
            definition.name(),
            format!("{}.{}", module_name, definition.name()),
        );
    }

    names.insert("main", "sloth_main".into());

    core::ast::Module::new(
        module
            .definitions()
            .iter()
            .map(|definition| match definition {
                core::ast::Definition::FunctionDefinition(function_definition) => {
                    core::ast::FunctionDefinition::new(
                        names
                            .get(function_definition.name())
                            .cloned()
                            .unwrap_or_else(|| function_definition.name().into()),
                        function_definition.environment().iter().cloned().collect(),
                        function_definition.arguments().iter().cloned().collect(),
                        function_definition.body().rename_variables(&names),
                        function_definition.result_type().clone(),
                    )
                    .into()
                }
                core::ast::Definition::ValueDefinition(value_definition) => {
                    core::ast::ValueDefinition::new(
                        names
                            .get(value_definition.name())
                            .cloned()
                            .unwrap_or_else(|| value_definition.name().into()),
                        value_definition.body().rename_variables(&names),
                        value_definition.type_().clone(),
                    )
                    .into()
                }
            })
            .collect(),
    )
}
