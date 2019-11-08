const OUTPUT_DIRECTORY: &str = ".ein";

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    match clap::App::new("ein")
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommand(clap::SubCommand::with_name("build"))
        .get_matches_safe()?
        .subcommand()
    {
        ("build", _) => build(),
        _ => unreachable!(),
    }
}

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let package = infra::parse_package_configuration(&std::fs::read_to_string("ein.json")?)?;
    let object_file_storage = infra::FileStorage::new(OUTPUT_DIRECTORY, "bc");

    app::PackageBuilder::new(
        &app::ModuleCompiler::new(
            &app::ModulePathConverter::new(&package),
            &infra::FileStorage::new("src", "sl"),
            &object_file_storage,
            &infra::FileStorage::new(OUTPUT_DIRECTORY, "json"),
        ),
        &infra::Linker::new(&package, std::env::var("SLOTH_ROOT")?, &object_file_storage),
    )
    .build()
}