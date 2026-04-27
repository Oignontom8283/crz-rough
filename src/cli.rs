
use clap::Parser;
use crate::common::{APP_NAME};


#[derive(Parser, Debug)]
#[command(author, version, about = APP_NAME)]
pub struct Args {
    /// Character used to separate fields in the value
    #[arg(short, long, default_value = ",")]
    pub delim: String,

    /// Default position of the cursor 
    #[arg(short, long, default_value_t = 0)]
    pub index: usize,

    #[arg(short, long, default_value_t = 10)]
    lines: u32
}