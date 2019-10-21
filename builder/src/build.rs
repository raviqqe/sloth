use std::io::Write;
use std::path::Path;

const BC_PATH: &str = "main.bc";

pub fn build(root_directory: String) -> Result<(), std::io::Error> {
    run_command(
        std::process::Command::new(Path::new(&root_directory).join("target/release/compiler"))
            .arg("-i")
            .arg(".")
            .arg("-m")
            .arg("main")
            .arg("-o")
            .arg(BC_PATH)
            .arg("main.sl"),
    )?;

    run_command(
        std::process::Command::new("clang")
            .arg("-O3")
            .arg("-flto")
            .arg("-ldl")
            .arg("-lpthread")
            .arg(BC_PATH)
            .arg(Path::new(&root_directory).join("target/release/libruntime.a")),
    )?;

    Ok(())
}

fn run_command(command: &mut std::process::Command) -> Result<(), std::io::Error> {
    let output = command.output()?;

    if output.status.success() {
        return Ok(());
    }

    std::io::stderr().write_all(&output.stdout)?;
    std::io::stderr().write_all(&output.stderr)?;

    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        output
            .status
            .code()
            .map(|code| format!("a command exited with status code {}", code))
            .unwrap_or_else(|| "a command exited with no status code".into()),
    ))
}