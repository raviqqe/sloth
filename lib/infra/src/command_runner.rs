use super::error::InfrastructureError;
use std::io::Write;

#[derive(Default)]
pub struct CommandRunner {}

impl CommandRunner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(
        &self,
        command: &mut std::process::Command,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let output = command.output()?;

        if output.status.success() {
            return Ok(String::from_utf8(output.stdout)?);
        }

        std::io::stderr().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;

        Err(InfrastructureError::CommandExit {
            status_code: output.status.code(),
        }
        .into())
    }
}
