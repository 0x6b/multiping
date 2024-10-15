use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, version)]
pub struct Args {
    /// Space seperated ping targets
    #[arg(
        value_delimiter = ' ',
        default_value = "192.168.0.10 turingpi.local 192.168.0.31 192.168.0.32 192.168.0.33 192.168.0.34"
    )]
    pub targets: Vec<String>,

    /// Specify ping interval in seconds
    #[arg(short, long, default_value = "1")]
    pub interval: f64,

    /// Specify ping timeout in seconds
    #[arg(short, long, default_value = "1")]
    pub timeout: f64,
}
