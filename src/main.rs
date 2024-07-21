mod args;
mod catalog;
mod direction;
mod image_data;
mod order;
mod palette;
mod path;
mod picture_io;
mod rank;
mod picture_entry;

use clap::Parser;
use crate::args::Args;

fn main() {
    let args = Args::parse();
    println!("{:?}", args);
}
