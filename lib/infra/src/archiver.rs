use super::command_runner::CommandRunner;
use super::utilities;
use std::collections::HashMap;

#[derive(Default)]
pub struct Archiver;

impl Archiver {
    pub fn new() -> Self {
        Self
    }
}

impl app::Archiver for Archiver {
    fn archive(
        &self,
        object_file_paths: &[app::FilePath],
        interface_file_paths: &HashMap<app::FilePath, app::FilePath>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        CommandRunner::run(
            std::process::Command::new("llvm-link")
                .arg("-o")
                .arg("library.bc")
                .args(object_file_paths.iter().map(utilities::convert_to_os_path)),
        )?;

        let mut builder = tar::Builder::new(std::fs::File::create("library.tar")?);
        builder.append_path("library.bc")?;
        std::fs::remove_file("library.bc")?;

        for (relative_interface_file_path, interface_file_path) in interface_file_paths {
            builder.append_file(
                &utilities::convert_to_os_path(relative_interface_file_path),
                &mut std::fs::File::open(&utilities::convert_to_os_path(interface_file_path))?,
            )?;
        }

        Ok(())
    }
}