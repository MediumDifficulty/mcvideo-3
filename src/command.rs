use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the input video file
    #[arg(short, long)]
    pub input: String,

    /// Width (in pixels) of the input video
    #[arg(short, long)]
    pub width: u32,

    /// Height (in pixels) of the input video
    #[arg(short, long)]
    pub height: u32,

    /// The local URL for the server. Change this if your hosting from another machine. (Then use it's address)
    #[arg(short, long, default_value_t = String::from("localhost"))]
    pub local_url: String,

    /// The audio segment length in seconds.
    #[arg(short, long, default_value_t = 20)]
    pub clip_length: usize,
}