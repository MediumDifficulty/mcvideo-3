use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub input: String,
    #[arg(short, long)]
    pub width: u32,
    #[arg(short, long)]
    pub height: u32,
    #[arg(short, long, default_value_t = String::from("localhost"))]
    pub local_url: String,
    #[arg(short, long, default_value_t = 20)]
    pub clip_length: usize,
}