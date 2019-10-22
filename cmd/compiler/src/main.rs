extern crate clap;
extern crate infra;
extern crate serde_json;
extern crate sloth;

use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = clap::App::new("Sloth Compiler")
        .version("0.1.0")
        .arg(
            clap::Arg::with_name("module_path")
                .short("m")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("module_interface_directory")
                .short("i")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("output_filename")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("input_filename")
                .index(1)
                .required(true),
        )
        .get_matches_safe()?;

    let input_filename = arguments
        .value_of("input_filename")
        .expect("input filename");

    let module = sloth::parse_module(sloth::Source::new(
        input_filename,
        &std::fs::read_to_string(input_filename)?,
    ))?;

    let module_path_resolver = infra::ModulePathResolver::new(
        arguments
            .value_of("module_interface_directory")
            .expect("module interface directory"),
    );

    let (object_blob, module_interface_blob) = sloth::compile(&sloth::ast::Module::new(
        sloth::parse_module_path(sloth::Source::new(
            "<module path argument>",
            arguments.value_of("module_path").expect("module path"),
        ))?,
        module.export().clone(),
        module
            .imports()
            .iter()
            .map(
                |import| -> Result<sloth::ast::ModuleInterface, Box<dyn std::error::Error>> {
                    Ok(serde_json::from_str(&std::fs::read_to_string(
                        module_path_resolver.resolve_module_interface(import.module_path()),
                    )?)?)
                },
            )
            .collect::<Result<Vec<_>, _>>()?,
        module.definitions().to_vec(),
    ))?;

    let destination = arguments
        .value_of("output_filename")
        .expect("output filename");

    File::create(destination)?.write_all(object_blob.as_bytes())?;

    File::create(Path::new(destination).with_extension("json"))?
        .write_all(module_interface_blob.as_bytes())?;

    Ok(())
}
