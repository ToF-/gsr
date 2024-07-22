use clap::Parser;
use crate::order::Order;

#[derive(Parser, Clone, Debug)]
#[command(infer_long_args = true, infer_subcommands = true)]
/// Gallery Show
pub struct Args {
    /// order pictures by Date
    #[arg(short, long, default_value_t = false)]
    pub date: bool,

    /// display only FILE_NAME
    #[arg(short, long, value_name="FILE_NAME")]
    pub file: Option<String>,

    /// display N x N pictures per page
    #[arg(short, long, value_name="N")]
    pub grid: Option<usize>,

    /// show the Nth picture first
    #[arg(short, long, value_name="N")]
    pub index: Option<usize>,

    /// order pictures by Label
    #[arg(short, long, default_value_t = false)]
    pub label: bool,

    /// order pictures by Name
    #[arg(short, long, default_value_t = false)]
    pub name: bool,

    /// display pictures in order
    #[arg(short, long, value_name="ORDER", ignore_case(true), default_value_t = Order::Random)]
    pub order: Order,

    /// display files that only match REGEXP
    #[arg(short, long, value_name="REGEXP")]
    pub pattern: Option<String>,

    /// display the files in FILE_LIST
    #[arg(short, long, value_name="FILE_LIST")]
    pub reading: Option<String>,

    /// wait N seconds between each picture
    #[arg(short, long, value_name="N")]
    pub seconds: Option<u64>,

    /// show thumbnails only
    #[arg(short, long, default_value_t = false)]
    pub thumbnails: bool,

    /// order pictures by Value
    #[arg(short, long, default_value_t = false)]
    pub value: bool,
}
