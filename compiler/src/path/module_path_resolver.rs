use super::module_path::ModulePath;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ModulePathResolver {
    module_interface_directory: String,
}

impl ModulePathResolver {
    pub fn new(module_interface_directory: impl Into<String>) -> Self {
        Self {
            module_interface_directory: module_interface_directory.into(),
        }
    }

    pub fn resolve_module_interface(&self, module_path: &ModulePath) -> PathBuf {
        let mut path = PathBuf::from(&self.module_interface_directory);

        match module_path {
            ModulePath::Absolute(_) => unimplemented!(),
            ModulePath::Relative(components) => {
                for component in components {
                    path.push(component);
                }
            }
        }

        path
    }
}