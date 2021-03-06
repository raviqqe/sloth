use super::error::CompileError;
use super::expression_compiler::ExpressionCompiler;
use super::type_compiler::TypeCompiler;
use crate::ast::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModuleCompiler {
    expression_compiler: Arc<ExpressionCompiler>,
    type_compiler: Arc<TypeCompiler>,
    global_names: Arc<HashMap<String, String>>,
}

impl ModuleCompiler {
    pub fn new(
        expression_compiler: Arc<ExpressionCompiler>,
        type_compiler: Arc<TypeCompiler>,
        global_names: Arc<HashMap<String, String>>,
    ) -> Self {
        Self {
            expression_compiler,
            type_compiler,
            global_names,
        }
    }

    pub fn compile(&self, module: &Module) -> Result<ssf::ir::Module, CompileError> {
        Ok(ssf::ir::Module::new(
            module
                .import_foreigns()
                .iter()
                .map(|import| -> Result<_, CompileError> {
                    Ok(ssf::ir::ForeignDeclaration::new(
                        import.name(),
                        import.foreign_name(),
                        self.type_compiler
                            .compile(import.type_())?
                            .into_function()
                            .ok_or_else(|| {
                                CompileError::FunctionExpected(import.source_information().clone())
                            })?,
                        match import.calling_convention() {
                            CallingConvention::Native => ssf::ir::CallingConvention::Source,
                            CallingConvention::C => ssf::ir::CallingConvention::Target,
                        },
                    ))
                })
                .collect::<Result<_, _>>()?,
            module
                .export_foreign()
                .names()
                .iter()
                .map(|name| {
                    Ok(ssf::ir::ForeignDefinition::new(
                        self.global_names.get(name).ok_or_else(|| {
                            CompileError::ExportedNameNotFound { name: name.clone() }
                        })?,
                        name,
                    ))
                })
                .collect::<Result<_, _>>()?,
            module
                .imports()
                .iter()
                .flat_map(|import| {
                    import
                        .module_interface()
                        .variables()
                        .iter()
                        .map(|(name, type_)| {
                            let type_ = self.type_compiler.compile(type_)?;

                            Ok(ssf::ir::Declaration::new(
                                name,
                                if let ssf::types::Type::Function(function_type) = type_ {
                                    function_type
                                } else {
                                    ssf::types::Function::new(
                                        self.type_compiler.compile_thunk_argument(),
                                        type_,
                                    )
                                },
                            ))
                        })
                })
                .collect::<Result<Vec<_>, CompileError>>()?,
            module
                .definitions()
                .iter()
                .map(|definition| {
                    Ok(match definition {
                        Definition::FunctionDefinition(function_definition) => {
                            vec![self.compile_function_definition(function_definition)?]
                        }
                        Definition::VariableDefinition(variable_definition) => {
                            self.compile_variable_definition(variable_definition)?
                        }
                    })
                })
                .collect::<Result<Vec<_>, CompileError>>()?
                .into_iter()
                .flatten()
                .collect(),
        ))
    }

    fn compile_function_definition(
        &self,
        function_definition: &FunctionDefinition,
    ) -> Result<ssf::ir::Definition, CompileError> {
        let core_type = self
            .type_compiler
            .compile_function(function_definition.type_())?;

        Ok(ssf::ir::Definition::new(
            function_definition.name(),
            function_definition
                .arguments()
                .iter()
                .zip(core_type.arguments())
                .map(|(name, type_)| ssf::ir::Argument::new(name.clone(), type_.clone()))
                .collect::<Vec<_>>(),
            self.expression_compiler
                .compile(function_definition.body())?,
            (0..function_definition.arguments().len())
                .fold(core_type.into(), |type_: ssf::types::Type, _| {
                    type_.into_function().unwrap().result().clone()
                }),
        ))
    }

    fn compile_variable_definition(
        &self,
        variable_definition: &VariableDefinition,
    ) -> Result<Vec<ssf::ir::Definition>, CompileError> {
        let core_type = self.type_compiler.compile(variable_definition.type_())?;

        Ok(
            if let ssf::types::Type::Function(function_type) = core_type {
                self.compile_function_variable_definition(
                    variable_definition.name(),
                    variable_definition.body(),
                    &function_type,
                )?
            } else {
                vec![self.compile_value_variable_definition(
                    variable_definition.name(),
                    variable_definition.body(),
                    &core_type,
                )?]
            },
        )
    }

    fn compile_function_variable_definition(
        &self,
        name: &str,
        body: &Expression,
        function_type: &ssf::types::Function,
    ) -> Result<Vec<ssf::ir::Definition>, CompileError> {
        let thunk_name = format!("{}.thunk", name);
        const ARGUMENT_NAME: &str = "$arg";

        Ok(vec![
            ssf::ir::Definition::thunk(
                &thunk_name,
                vec![ssf::ir::Argument::new(
                    "",
                    self.type_compiler.compile_thunk_argument(),
                )],
                self.expression_compiler.compile(body)?,
                function_type.clone(),
            ),
            ssf::ir::Definition::new(
                name,
                vec![ssf::ir::Argument::new(
                    ARGUMENT_NAME,
                    function_type.argument().clone(),
                )],
                ssf::ir::FunctionApplication::new(
                    ssf::ir::FunctionApplication::new(
                        ssf::ir::Variable::new(&thunk_name),
                        ssf::ir::ConstructorApplication::new(
                            ssf::ir::Constructor::new(
                                self.type_compiler.compile_thunk_argument(),
                                0,
                            ),
                            vec![],
                        ),
                    ),
                    ssf::ir::Variable::new(ARGUMENT_NAME),
                ),
                function_type.result().clone(),
            ),
        ])
    }

    fn compile_value_variable_definition(
        &self,
        name: &str,
        body: &Expression,
        type_: &ssf::types::Type,
    ) -> Result<ssf::ir::Definition, CompileError> {
        Ok(ssf::ir::Definition::thunk(
            name,
            vec![ssf::ir::Argument::new(
                "",
                self.type_compiler.compile_thunk_argument(),
            )],
            self.expression_compiler.compile(body)?,
            type_.clone(),
        ))
    }
}
