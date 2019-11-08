use crate::package::Package;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ModulePath {
    package: Package,
    components: Vec<String>,
}

impl ModulePath {
    pub fn new(package: Package, components: Vec<String>) -> Self {
        Self {
            package,
            components,
        }
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn components(&self) -> &[String] {
        &self.components
    }

    pub fn qualify_name(&self, name: &str) -> String {
        [
            self.components
                .last()
                .map(|component| component.as_str())
                .unwrap_or_else(|| self.package.name()),
            name,
        ]
        .join(".")
    }

    pub fn fully_qualify_name(&self, name: &str) -> String {
        vec![&format!(
            "{}({})",
            self.package.name(),
            self.package.version()
        )]
        .into_iter()
        .chain(self.components.iter())
        .map(|component| component.as_str())
        .chain(vec![name].into_iter())
        .collect::<Vec<_>>()
        .join(".")
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.components.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualify_name() {
        assert_eq!(
            ModulePath::new(
                Package::new("foo", (0, 0, 0)),
                vec!["bar".into(), "baz".into()]
            )
            .qualify_name("blah"),
            "baz.blah"
        );
    }

    #[test]
    fn fully_qualify_name() {
        assert_eq!(
            ModulePath::new(
                Package::new("foo", (0, 0, 0)),
                vec!["bar".into(), "baz".into()]
            )
            .fully_qualify_name("blah"),
            "foo(0.0.0).bar.baz.blah"
        );
    }
}