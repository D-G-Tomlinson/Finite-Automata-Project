use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Location of the file storing the DSA
    #[arg(short,long)]
    dfa: Option<String>,

    /// The word to be analysed
    #[arg(short, long, value_name = "FILE")]
    word: Option<PathBuf>,
}
fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(dfa) = cli.dfa.as_deref() {
        println!("Value for dfa: {dfa}");
    }
}
