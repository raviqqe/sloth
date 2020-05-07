use super::super::error::CompileError;
use super::super::expression_type_extractor::ExpressionTypeExtractor;
use super::super::name_generator::NameGenerator;
use super::super::reference_type_resolver::ReferenceTypeResolver;
use super::super::type_equality_checker::TypeEqualityChecker;
use super::typed_meta_desugarer::TypedDesugarer;
use crate::ast::*;
use crate::debug::SourceInformation;
use crate::types::Type;
use std::collections::HashMap;
use std::rc::Rc;

pub struct FunctionTypeArgumentDesugarer {
    name_generator: NameGenerator,
    reference_type_resolver: Rc<ReferenceTypeResolver>,
    type_equality_checker: Rc<TypeEqualityChecker>,
    expression_type_extractor: Rc<ExpressionTypeExtractor>,
}

impl FunctionTypeArgumentDesugarer {
    pub fn new(
        reference_type_resolver: Rc<ReferenceTypeResolver>,
        type_equality_checker: Rc<TypeEqualityChecker>,
        expression_type_extractor: Rc<ExpressionTypeExtractor>,
    ) -> Self {
        FunctionTypeArgumentDesugarer {
            name_generator: NameGenerator::new("fta_function_"),
            reference_type_resolver,
            type_equality_checker,
            expression_type_extractor,
        }
    }

    fn desugar_function_type_argument(
        &mut self,
        expression: &Expression,
        to_type: &Type,
        source_information: Rc<SourceInformation>,
        variables: &HashMap<String, Type>,
    ) -> Result<Expression, CompileError> {
        let from_type = self.reference_type_resolver.resolve(
            &self
                .expression_type_extractor
                .extract(&expression, variables)?,
        )?;
        let to_type = self.reference_type_resolver.resolve(to_type)?;

        Ok(
            if to_type.is_function()
                && (!self.type_equality_checker.equal(&from_type, &to_type)?
                    || !expression.is_variable())
            {
                let name = self.name_generator.generate();

                Let::new(
                    vec![ValueDefinition::new(
                        &name,
                        expression.clone(),
                        to_type,
                        source_information.clone(),
                    )
                    .into()],
                    Variable::new(name, source_information),
                )
                .into()
            } else {
                expression.clone()
            },
        )
    }
}

impl TypedDesugarer for FunctionTypeArgumentDesugarer {
    fn desugar_function_definition(
        &mut self,
        function_definition: &FunctionDefinition,
        _: &HashMap<String, Type>,
    ) -> Result<FunctionDefinition, CompileError> {
        Ok(function_definition.clone())
    }

    fn desugar_value_definition(
        &mut self,
        value_definition: &ValueDefinition,
        _: &HashMap<String, Type>,
    ) -> Result<ValueDefinition, CompileError> {
        Ok(value_definition.clone())
    }

    fn desugar_expression(
        &mut self,
        expression: &Expression,
        variables: &HashMap<String, Type>,
    ) -> Result<Expression, CompileError> {
        match expression {
            Expression::Application(application) => {
                let source_information = application.source_information();
                let function_type = self
                    .expression_type_extractor
                    .extract(application.function(), variables)?;

                Ok(Application::new(
                    application.function().clone(),
                    self.desugar_function_type_argument(
                        application.argument(),
                        function_type.to_function().unwrap().argument(),
                        source_information.clone(),
                        variables,
                    )?,
                    source_information.clone(),
                )
                .into())
            }
            Expression::RecordConstruction(record_construction) => {
                let type_ = self
                    .reference_type_resolver
                    .resolve_reference(record_construction.type_())?;
                let record_type = type_.to_record().unwrap();

                Ok(RecordConstruction::new(
                    record_construction.type_().clone(),
                    record_construction
                        .elements()
                        .iter()
                        .map(|(key, expression)| {
                            Ok((
                                key.clone(),
                                self.desugar_function_type_argument(
                                    expression,
                                    &record_type.elements()[key],
                                    record_construction.source_information().clone(),
                                    variables,
                                )?,
                            ))
                        })
                        .collect::<Result<_, CompileError>>()?,
                    record_construction.source_information().clone(),
                )
                .into())
            }
            Expression::Boolean(_)
            | Expression::Case(_) // TODO Desugar case expression arguments.
            | Expression::If(_)
            | Expression::Let(_)
            | Expression::None(_)
            | Expression::Number(_)
            | Expression::Operation(_)
            | Expression::RecordElementOperation(_)
            | Expression::Variable(_) => Ok(expression.clone()),
            Expression::RecordUpdate(_) | Expression::TypeCoercion(_) => unreachable!(),
        }
    }
}
