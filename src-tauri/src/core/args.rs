/// Simple program to greet a person
#[derive(Debug, clap::Parser)]
pub struct Args {
    #[arg(long)]
    pub settings_path: Option<String>,
}
