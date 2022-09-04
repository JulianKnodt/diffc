use clap::{Parser, ArgEnum};
use progdiff::{frontend::ProgramParser};

#[derive(Debug, Clone, Copy, ArgEnum)]
pub enum Target {
  Rust,
}

/// A compiler for differentiating expressions.
#[derive(Debug, Clone, Parser)]
pub struct Args {
  #[clap(short, long)]
  input: String,

  #[clap(long, arg_enum)]
  target: Target,

  #[clap(long)]
  stdout: bool
}

fn main() {
  let args = Args::parse();

  // TODO parse input file using standard frontend

}
