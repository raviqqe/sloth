mod boolean_operation_transformer;
mod elementless_record_transformer;
mod equal_operation_transformer;
mod function_type_argument_transformer;
mod list_literal_transformer;
mod not_equal_operation_transformer;
mod partial_application_transformer;
mod record_element_function_transformer;
mod record_equal_function_transformer;
mod record_update_transformer;
mod type_coercion_transformer;
mod typed_meta_transformer;
mod utilities;

use super::error::CompileError;
use super::expression_type_extractor::ExpressionTypeExtractor;
use super::reference_type_resolver::ReferenceTypeResolver;
use super::type_canonicalizer::TypeCanonicalizer;
use super::type_comparability_checker::TypeComparabilityChecker;
use super::type_equality_checker::TypeEqualityChecker;
use crate::ast::*;
pub use boolean_operation_transformer::BooleanOperationTransformer;
use elementless_record_transformer::ElementlessRecordTransformer;
pub use equal_operation_transformer::EqualOperationTransformer;
use function_type_argument_transformer::FunctionTypeArgumentTransformer;
pub use list_literal_transformer::ListLiteralTransformer;
pub use not_equal_operation_transformer::NotEqualOperationTransformer;
use partial_application_transformer::PartialApplicationTransformer;
use record_element_function_transformer::RecordElementFunctionTransformer;
use record_equal_function_transformer::RecordEqualFunctionTransformer;
use record_update_transformer::RecordUpdateTransformer;
use type_coercion_transformer::TypeCoercionTransformer;
use typed_meta_transformer::TypedMetaTransformer;

pub fn transform_before_name_qualification(module: &Module) -> Result<Module, CompileError> {
    let reference_type_resolver = ReferenceTypeResolver::new(module);
    let type_comparability_checker = TypeComparabilityChecker::new(reference_type_resolver);

    let module = ElementlessRecordTransformer::new().transform(&module);
    let module = RecordElementFunctionTransformer::new().transform(&module);
    let module =
        RecordEqualFunctionTransformer::new(type_comparability_checker).transform(&module)?;

    Ok(module)
}

pub fn transform_without_types(module: &Module) -> Result<Module, CompileError> {
    RecordUpdateTransformer::new().transform(module)
}

pub fn transform_with_types(module: &Module) -> Result<Module, CompileError> {
    let reference_type_resolver = ReferenceTypeResolver::new(module);
    let type_equality_checker = TypeEqualityChecker::new(reference_type_resolver.clone());
    let type_canonicalizer = TypeCanonicalizer::new(
        reference_type_resolver.clone(),
        type_equality_checker.clone(),
    );
    let expression_type_extractor =
        ExpressionTypeExtractor::new(reference_type_resolver.clone(), type_canonicalizer.clone());

    let mut type_coercion_transformer = TypedMetaTransformer::new(
        TypeCoercionTransformer::new(
            reference_type_resolver.clone(),
            type_equality_checker.clone(),
            expression_type_extractor.clone(),
            type_canonicalizer,
        ),
        reference_type_resolver.clone(),
    );
    let mut function_type_argument_transformer = TypedMetaTransformer::new(
        FunctionTypeArgumentTransformer::new(
            reference_type_resolver.clone(),
            type_equality_checker,
            expression_type_extractor,
        ),
        reference_type_resolver,
    );
    let partial_application_transformer = PartialApplicationTransformer::new();

    let module = function_type_argument_transformer.transform(&module)?;
    let module = partial_application_transformer.transform(&module)?;
    let module = type_coercion_transformer.transform(&module)?;

    Ok(module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::SourceInformation;
    use crate::types;
    use insta::assert_debug_snapshot;

    fn transform_with_types(module: &Module) -> Result<Module, CompileError> {
        super::transform_with_types(module)
    }

    mod type_coercion {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_function_definition() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |body: Expression| {
                Module::from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Function::new(
                            union_type.clone(),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    FunctionDefinition::new(
                        "g",
                        vec!["x".into()],
                        body,
                        types::Function::new(
                            types::Number::new(SourceInformation::dummy()),
                            union_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Application::new(
                        Variable::new("f", SourceInformation::dummy()),
                        Number::new(42.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Application::new(
                            Variable::new("f", SourceInformation::dummy()),
                            TypeCoercion::new(
                                Number::new(42.0, SourceInformation::dummy()),
                                types::Number::new(SourceInformation::dummy()),
                                union_type.clone(),
                                SourceInformation::dummy(),
                            ),
                            SourceInformation::dummy(),
                        ),
                        types::Number::new(SourceInformation::dummy()),
                        union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_variable_definition() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |expression: Expression| {
                Module::from_definitions(vec![VariableDefinition::new(
                    "x",
                    expression,
                    union_type.clone(),
                    SourceInformation::dummy(),
                )
                .into()])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Number::new(42.0, SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_application() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |argument: Expression| {
                Module::from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Function::new(
                            union_type.clone(),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    VariableDefinition::new(
                        "x",
                        Application::new(
                            Variable::new("f", SourceInformation::dummy()),
                            argument,
                            SourceInformation::dummy(),
                        ),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Number::new(42.0, SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_let_value_expression() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |expression: Expression| {
                Module::from_definitions(vec![VariableDefinition::new(
                    "x",
                    Let::new(
                        vec![VariableDefinition::new(
                            "y",
                            expression,
                            union_type.clone(),
                            SourceInformation::dummy(),
                        )
                        .into()],
                        Number::new(42.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    ),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
                .into()])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Number::new(42.0, SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_let_function_expression() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |body: Expression| {
                Module::from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Function::new(
                            union_type.clone(),
                            types::Number::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    VariableDefinition::new(
                        "x",
                        Let::new(
                            vec![FunctionDefinition::new(
                                "g",
                                vec!["x".into()],
                                body,
                                types::Function::new(
                                    types::Number::new(SourceInformation::dummy()),
                                    union_type.clone(),
                                    SourceInformation::dummy(),
                                ),
                                SourceInformation::dummy(),
                            )
                            .into()],
                            Number::new(42.0, SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        types::Number::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Application::new(
                        Variable::new("f", SourceInformation::dummy()),
                        Number::new(42.0, SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Application::new(
                            Variable::new("f", SourceInformation::dummy()),
                            TypeCoercion::new(
                                Number::new(42.0, SourceInformation::dummy()),
                                types::Number::new(SourceInformation::dummy()),
                                union_type.clone(),
                                SourceInformation::dummy(),
                            ),
                            SourceInformation::dummy(),
                        ),
                        types::Number::new(SourceInformation::dummy()),
                        union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_record_construction() {
            let union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );
            let reference_type = types::Reference::new("Foo", SourceInformation::dummy());

            assert_debug_snapshot!(transform_with_types(
                &Module::from_definitions_and_type_definitions(
                    vec![TypeDefinition::new(
                        "Foo",
                        types::Record::new(
                            "Foo",
                            vec![("foo".into(), union_type.into())]
                                .into_iter()
                                .collect(),
                            SourceInformation::dummy(),
                        )
                    )],
                    vec![VariableDefinition::new(
                        "x",
                        RecordConstruction::new(
                            reference_type.clone(),
                            vec![(
                                "foo".into(),
                                Number::new(42.0, SourceInformation::dummy()).into()
                            )]
                            .into_iter()
                            .collect(),
                            SourceInformation::dummy(),
                        ),
                        reference_type,
                        SourceInformation::dummy(),
                    )
                    .into()],
                )
            ));
        }

        #[test]
        fn transform_union() {
            let lower_union_type = types::Union::new(
                vec![
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );
            let upper_union_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::Number::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |expression1: Expression, expression2: Expression| {
                Module::from_definitions(vec![
                    VariableDefinition::new(
                        "x",
                        expression1,
                        lower_union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    VariableDefinition::new(
                        "y",
                        expression2,
                        upper_union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Number::new(42.0, SourceInformation::dummy()).into(),
                    Variable::new("x", SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        lower_union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    TypeCoercion::new(
                        Variable::new("x", SourceInformation::dummy()),
                        lower_union_type.clone(),
                        upper_union_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_function() {
            let lower_type = types::None::new(SourceInformation::dummy());
            let upper_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |definition: Definition| {
                Module::from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        None::new(SourceInformation::dummy()),
                        types::Function::new(
                            upper_type.clone(),
                            lower_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    VariableDefinition::new(
                        "x",
                        Let::new(
                            vec![definition],
                            None::new(SourceInformation::dummy()),
                            SourceInformation::dummy(),
                        ),
                        types::None::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    VariableDefinition::new(
                        "g",
                        Variable::new("f", SourceInformation::dummy()),
                        types::Function::new(
                            lower_type.clone(),
                            upper_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into()
                )),
                Ok(create_module(
                    FunctionDefinition::new(
                        "g",
                        vec!["pa_argument_0".into()],
                        TypeCoercion::new(
                            Application::new(
                                Variable::new("f", SourceInformation::dummy()),
                                TypeCoercion::new(
                                    Variable::new("pa_argument_0", SourceInformation::dummy()),
                                    lower_type.clone(),
                                    upper_type.clone(),
                                    SourceInformation::dummy(),
                                ),
                                SourceInformation::dummy()
                            ),
                            lower_type.clone(),
                            upper_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        types::Function::new(
                            lower_type.clone(),
                            upper_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_function_as_argument() {
            let lower_type = types::None::new(SourceInformation::dummy());
            let upper_type = types::Union::new(
                vec![
                    types::Boolean::new(SourceInformation::dummy()).into(),
                    types::None::new(SourceInformation::dummy()).into(),
                ],
                SourceInformation::dummy(),
            );

            let create_module = |expression: Expression| {
                Module::from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec!["x".into()],
                        None::new(SourceInformation::dummy()),
                        types::Function::new(
                            upper_type.clone(),
                            lower_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    FunctionDefinition::new(
                        "g",
                        vec!["x".into()],
                        None::new(SourceInformation::dummy()),
                        types::Function::new(
                            types::Function::new(
                                lower_type.clone(),
                                upper_type.clone(),
                                SourceInformation::dummy(),
                            ),
                            lower_type.clone(),
                            SourceInformation::dummy(),
                        ),
                        SourceInformation::dummy(),
                    )
                    .into(),
                    VariableDefinition::new(
                        "x",
                        Application::new(
                            Variable::new("g", SourceInformation::dummy()),
                            expression,
                            SourceInformation::dummy(),
                        ),
                        lower_type.clone(),
                        SourceInformation::dummy(),
                    )
                    .into(),
                ])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Variable::new("f", SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    Let::new(
                        vec![FunctionDefinition::new(
                            "fta_function_0",
                            vec!["pa_argument_0".into()],
                            TypeCoercion::new(
                                Application::new(
                                    Variable::new("f", SourceInformation::dummy()),
                                    TypeCoercion::new(
                                        Variable::new("pa_argument_0", SourceInformation::dummy()),
                                        lower_type.clone(),
                                        upper_type.clone(),
                                        SourceInformation::dummy(),
                                    ),
                                    SourceInformation::dummy()
                                ),
                                lower_type.clone(),
                                upper_type.clone(),
                                SourceInformation::dummy(),
                            ),
                            types::Function::new(
                                lower_type.clone(),
                                upper_type.clone(),
                                SourceInformation::dummy(),
                            ),
                            SourceInformation::dummy(),
                        )
                        .into()],
                        Variable::new("fta_function_0", SourceInformation::dummy()),
                        SourceInformation::dummy()
                    )
                    .into()
                ))
            );
        }

        #[test]
        fn transform_any() {
            let create_module = |expression: Expression| {
                Module::from_definitions(vec![VariableDefinition::new(
                    "x",
                    expression,
                    types::Any::new(SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
                .into()])
            };

            assert_eq!(
                transform_with_types(&create_module(
                    Number::new(42.0, SourceInformation::dummy()).into()
                )),
                Ok(create_module(
                    TypeCoercion::new(
                        Number::new(42.0, SourceInformation::dummy()),
                        types::Number::new(SourceInformation::dummy()),
                        types::Any::new(SourceInformation::dummy()),
                        SourceInformation::dummy(),
                    )
                    .into()
                ))
            );
        }
    }

    #[test]
    fn transform_case_expression() {
        let argument_union_type = types::Union::new(
            vec![
                types::Boolean::new(SourceInformation::dummy()).into(),
                types::None::new(SourceInformation::dummy()).into(),
            ],
            SourceInformation::dummy(),
        );
        let result_union_type = types::Union::new(
            vec![
                types::Number::new(SourceInformation::dummy()).into(),
                types::None::new(SourceInformation::dummy()).into(),
            ],
            SourceInformation::dummy(),
        );

        assert_debug_snapshot!(transform_with_types(&Module::from_definitions(vec![
            VariableDefinition::new(
                "x",
                Case::with_type(
                    argument_union_type,
                    "foo",
                    Boolean::new(false, SourceInformation::dummy()),
                    vec![
                        Alternative::new(
                            types::Boolean::new(SourceInformation::dummy()),
                            Number::new(42.0, SourceInformation::dummy()),
                        ),
                        Alternative::new(
                            types::None::new(SourceInformation::dummy()),
                            None::new(SourceInformation::dummy()),
                        ),
                    ],
                    SourceInformation::dummy(),
                ),
                result_union_type,
                SourceInformation::dummy(),
            )
            .into()
        ])));
    }
}