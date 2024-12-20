mod args;
mod server;

use clap::Parser;

use args::Args;
use server::ServerSetup;
use testit_lib::{config::{AppConfiguration, TestConfiguration}, error::ApplicationError};

/**
 * The main function for the testit-daemon application.
 * 
 * This application is used to start a daemon.
 */
#[actix_web::main]
async fn main() -> Result<(), ApplicationError> {
    let args = Args::parse();
    let config = read_input_file(&args)?;
    init(args, config).await?;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        wait_for_terminate().await?
    }
}

/**
 * Read the input file with the specified arguments.
 * 
 * # Arguments
 * @param args: The arguments to read the input file with.
 * 
 * # Returns
 * @return The configuration read from the input file.
 * 
 * # Errors
 * @return An error if the input file could not be read.
 */
fn read_input_file(args: &Args) -> Result<AppConfiguration, ApplicationError> {
    let config = AppConfiguration::load(&args.file)?;
    Ok(config)
}

/**
 * Initialize the application with the specified arguments and configuration.
 * 
 * # Arguments
 * @param args: The arguments to initialize the application with.
 * @param config: The configuration to initialize the application with.
 * 
 * # Returns
 * @return Ok if the application was initialized successfully.
 * 
 * # Errors
 * @return An error if the daemon could not be started.
 * @return An error if the tests could not be listed.
 * @return An error if the id is missing.
 * @return An error if the test is not found.
 */
async fn init(args: Args, config: AppConfiguration) -> Result<(), ApplicationError> {
    if args.list {
        list_tests(&config)?;
    } else {
        start_daemon(&args.id, &config).await?;
    }
    Ok(())
}

/**
 * List the available tests in the specified configuration.
 * 
 * # Arguments
 * @param config: The configuration to list the tests from.
 * 
 * # Returns
 * @return Ok if the tests were listed successfully.
 */
fn list_tests(config: &AppConfiguration) -> Result<(), ApplicationError> {
    println!("Available tests for configuration: {}", config.name);
    println!("ID\tName\tDescription");
    for test in &config.tests {
        println!("{}\t{}\t{}", test.id, test.name, test.description);
    }
    Ok(())
}

/**
 * Start the daemon with the specified id.
 * 
 * # Arguments
 * @param id: The id of the test to start.
 * @param config: The configuration to search for the test.
 * 
 * # Returns
 * @return Ok if the daemon was started successfully.
 * 
 * # Errors
 * @return An error if the test is not found.
 * @return An error if the id is missing.
 */
async fn start_daemon(id: &Option<String>, config: &AppConfiguration) -> Result<(), ApplicationError> {
    let id = match id {
        Some(id) => id,
        None => { return Err(ApplicationError::MissingId("Missing id".to_string())); }
    };
    let test = get_test(id, config)?;
    let mut server_setup = ServerSetup::new();
    server_setup.setup_test(test).await;
    server_setup.start_servers().await.map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    Ok(())
}

/**
 * Get the test with the specified id.
 * 
 * # Arguments
 * @param id: The id of the test.
 * @param config: The configuration to search for the test.
 * 
 * # Returns
 * @return The test with the specified id.
 * 
 * # Errors
 * @return An error if the test is not found.
 */
fn get_test<'a>(id: &str, config: &'a AppConfiguration) -> Result<&'a TestConfiguration, ApplicationError> {
    let test = config.tests.iter().find(|test| test.id == id);
    match test {
        Some(test) => Ok(test),
        None => Err(ApplicationError::CouldNotFindTest(format!("No test with id: {}", id)))
    }
}


/**
 * Wait for the terminate signal.
 */
#[cfg(unix)]
async fn wait_for_terminate() -> Result<(), ApplicationError> {
    use std::process::exit;

    use tokio::signal::unix::{signal, SignalKind};

    // Infos here:
    // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
    let mut signal_terminate = signal(SignalKind::terminate()).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    let mut signal_interrupt = signal(SignalKind::interrupt()).map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;

    tokio::select! {
        _ = signal_terminate.recv() => exit(0),
        _ = signal_interrupt.recv() => exit(0),
    };
}

/**
 * Wait for the terminate signal.
 */
#[cfg(windows)]
async fn wait_for_terminate() -> Result<(), ApplicationError> {
    use std::process::exit;

    use tokio::signal::windows;

    // Infos here:
    // https://learn.microsoft.com/en-us/windows/console/handlerroutine
    let mut signal_c = windows::ctrl_c().map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    let mut signal_break = windows::ctrl_break().map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    let mut signal_close = windows::ctrl_close().map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;
    let mut signal_shutdown = windows::ctrl_shutdown().map_err(|err| ApplicationError::ServerStartUpError(err.to_string()))?;

    tokio::select! {
        _ = signal_c.recv() => exit(0),
        _ = signal_break.recv() => exit(0),
        _ = signal_close.recv() => exit(0),
        _ = signal_shutdown.recv() => exit(0),
    };
}


