use clap::Parser;
use crate::order::Order;

#[derive(Parser, Clone, Debug)]
#[command(infer_long_args = true, infer_subcommands = true)]
/// Gallery Show
pub struct Args {
    /// Pattern that the file names must match to be selected
    #[arg(long, value_name="REGEXP")]
    pub pattern: Option<String>,

    /// List of pictures to be viewed
    #[arg(short, long, value_name="FILE_LIST")]
    pub reading: Option<String>,

    /// File to be viewed
    #[arg(short, long, value_name="FILE_NAME")]
    pub file: Option<String>,

    /// Ordered display
    #[arg(short, long, value_name="ORDER", ignore_case(true), default_value_t = Order::Random)]
    pub order: Order,

    /// Order by Name
    #[arg(short, long, default_value_t = false)]
    pub name: bool,


}
