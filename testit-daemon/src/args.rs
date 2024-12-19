use clap::Parser;

/// Command line application for starting daemon and reading test configurations.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author="Kjetil Fjellheim")]
pub struct Args {
    /// Input file.
    #[arg(long)]
    pub file: String,

    /// This starts the daemon with the specified if from the file.
    #[arg(long)]
    pub id: Option<String>,

    /// Lists the available tests in the specified file.
    #[arg(long)]
    pub list: bool,
}