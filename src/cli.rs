use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short = 'c', long)]
    pub copy: bool,
}
