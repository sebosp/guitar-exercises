use clap::App;
use stderrlog;

pub const DEFAULT_VERBOSITY:i8 = 1; // Info statements

pub fn parse_cli_args() -> Result<::GuitarExerciseRequest, String> {
    // Parse command line arguments from cli.yml
    let cli_yaml = load_yaml!("cli.yml");
    let clap_args = App::from_yaml(cli_yaml).get_matches();
    let verbosity = clap_args.value_of("verbosity").map(|v| {
        v.parse::<usize>().unwrap_or_else(|_| {
            clap_invalidvalue(
                "invalid value for 'verbosity', should be a value from 0 to 4".to_string()
            ).exit()
        })
    }).unwrap_or(DEFAULT_VERBOSITY as usize);
    let quiet = clap_args.is_present("quiet");
    // Initialize logger
    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbosity)
        .timestamp(ts)
        .init()
        .unwrap();
    trace!("Starting CLI arg validation");
    let command_mode = clap_args.value_of("MODE").unwrap();
    GuitarExerciseRequest {
        mode: command_mode,
        note: 
    }
}