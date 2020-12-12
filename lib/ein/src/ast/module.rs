use super::definition::Definition;
use super::export::Export;
use super::expression::Expression;
use super::ffi_package_interface::FfiPackageInterface;
use super::import::Import;
use super::let_::Let;
use super::type_definition::TypeDefinition;
use crate::path::ModulePath;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    path: ModulePath,
    type_definitions: Vec<TypeDefinition>,
    definitions: Vec<Definition>,
    export: Export,
    imports: Vec<Import>,
    ffi_imports: Vec<FfiPackageInterface>,
}

impl Module {
    pub fn new(
        path: ModulePath,
        export: Export,
        imports: Vec<Import>,
        ffi_imports: Vec<FfiPackageInterface>,
        type_definitions: Vec<TypeDefinition>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self {
            path,
            type_definitions,
            definitions,
            export,
            imports,
            ffi_imports,
        }
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        Self::new(
            ModulePath::new(crate::package::Package::new("", ""), vec![]),
            Export::new(Default::default()),
            vec![],
            vec![],
            vec![],
            vec![],
        )
    }

    #[cfg(test)]
    pub fn from_definitions(definitions: Vec<Definition>) -> Self {
        Self::new(
            ModulePath::new(crate::package::Package::new("", ""), vec![]),
            Export::new(Default::default()),
            vec![],
            vec![],
            vec![],
            definitions,
        )
    }

    #[cfg(test)]
    pub fn from_definitions_and_type_definitions(
        type_definitions: Vec<TypeDefinition>,
        definitions: Vec<Definition>,
    ) -> Self {
        Self::new(
            ModulePath::new(crate::package::Package::new("", ""), vec![]),
            Export::new(Default::default()),
            vec![],
            vec![],
            type_definitions,
            definitions,
        )
    }

    pub fn path(&self) -> &ModulePath {
        &self.path
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn definitions(&self) -> &[Definition] {
        &self.definitions
    }

    pub fn export(&self) -> &Export {
        &self.export
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn ffi_imports(&self) -> &[FfiPackageInterface] {
        &self.ffi_imports
    }

    pub fn transform_definitions<E>(
        &self,
        mut transform: &mut impl FnMut(&Definition) -> Result<Definition, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.path.clone(),
            self.export.clone(),
            self.imports.clone(),
            self.ffi_imports.clone(),
            self.type_definitions.clone(),
            self.definitions
                .iter()
                .map(|definition| {
                    let definition = definition.transform_expressions(&mut |expression| {
                        if let Expression::Let(let_) = expression {
                            Ok(Let::new(
                                let_.definitions()
                                    .iter()
                                    .map(&mut transform)
                                    .collect::<Result<_, _>>()?,
                                let_.expression().clone(),
                                let_.source_information().clone(),
                            )
                            .into())
                        } else {
                            Ok(expression.clone())
                        }
                    })?;

                    transform(&definition)
                })
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn transform_expressions<E>(
        &self,
        transform: &mut impl FnMut(&Expression) -> Result<Expression, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.path.clone(),
            self.export.clone(),
            self.imports.clone(),
            self.ffi_imports.clone(),
            self.type_definitions.clone(),
            self.definitions
                .iter()
                .map(|definition| definition.transform_expressions(transform))
                .collect::<Result<_, _>>()?,
        ))
    }

    pub fn transform_types<E>(
        &self,
        transform: &mut impl FnMut(&Type) -> Result<Type, E>,
    ) -> Result<Self, E> {
        Ok(Self::new(
            self.path.clone(),
            self.export.clone(),
            self.imports.clone(),
            self.ffi_imports.clone(),
            self.type_definitions
                .iter()
                .map(|type_definition| type_definition.transform_types(transform))
                .collect::<Result<_, _>>()?,
            self.definitions
                .iter()
                .map(|definition| definition.transform_types(transform))
                .collect::<Result<_, _>>()?,
        ))
    }
}
