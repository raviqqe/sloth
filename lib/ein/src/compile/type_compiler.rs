use super::reference_type_resolver::ReferenceTypeResolver;
use crate::ast;
use crate::types::{self, Type};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct TypeCompiler {
    reference_indices: HashMap<String, usize>,
    reference_type_resolver: Rc<ReferenceTypeResolver>,
}

impl TypeCompiler {
    pub fn new(module: &ast::Module) -> Self {
        Self {
            reference_indices: HashMap::new(),
            reference_type_resolver: ReferenceTypeResolver::new(module).into(),
        }
    }

    pub fn compile(&self, type_: &Type) -> ssf::types::Type {
        match type_ {
            Type::Function(_) => self.compile_function(type_).into(),
            Type::Number(_) => ssf::types::Value::Float64.into(),
            Type::Reference(_) => unimplemented!(),
            Type::Unknown(_) | Type::Variable(_) => unreachable!(),
        }
    }

    pub fn compile_function(&self, type_: &types::Type) -> ssf::types::Function {
        match type_ {
            Type::Function(function) => ssf::types::Function::new(
                function
                    .arguments()
                    .iter()
                    .map(|type_| self.compile(type_))
                    .collect::<Vec<_>>(),
                self.compile_value(function.last_result()),
            ),
            Type::Number(_) => unreachable!(),
            Type::Reference(_) => {
                self.compile_function(&self.reference_type_resolver.resolve(type_))
            }
            Type::Unknown(_) | Type::Variable(_) => unreachable!(),
        }
    }

    pub fn compile_value(&self, type_: &Type) -> ssf::types::Value {
        match type_ {
            Type::Function(_) => unreachable!(),
            Type::Number(_) => ssf::types::Value::Float64,
            Type::Reference(_) => self.compile_value(&self.reference_type_resolver.resolve(type_)),
            Type::Unknown(_) | Type::Variable(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::debug::SourceInformation;

    #[test]
    fn compile_number_type() {
        assert_eq!(
            TypeCompiler::new(&Module::dummy())
                .compile(&types::Number::new(SourceInformation::dummy()).into()),
            ssf::types::Value::Float64.into()
        );
    }

    #[test]
    fn compile_function_type() {
        assert_eq!(
            TypeCompiler::new(&Module::dummy()).compile(
                &types::Function::new(
                    types::Number::new(SourceInformation::dummy()),
                    types::Number::new(SourceInformation::dummy()),
                    SourceInformation::dummy(),
                )
                .into()
            ),
            ssf::types::Function::new(
                vec![ssf::types::Value::Float64.into()],
                ssf::types::Value::Float64
            )
            .into()
        );
    }
}
