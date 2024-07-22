use clap::Parser;
use crate::order::Order;
use crate::path::{directory};

#[derive(Parser, Clone, Debug)]
#[command(infer_long_args = true, infer_subcommands = true)]
/// Gallery Show
pub struct Args {
     /// Directory to search (default is set with variable GALLSHDIR)
    pub directory: Option<String>,

    /// move all labelled pictures to their matching folder on TARGET_DIR
    #[arg(short, long, value_name = "TARGET_DIR")]
    pub all_move: Option<String>,

    /// copy selected files to TARGET_DIR
    #[arg(short, long, value_name = "TARGET_DIR")]
    pub copy_selection: Option<String>,

    /// order pictures by Date
    #[arg(short, long, default_value_t = false)]
    pub date: bool,

    /// display only FILE_NAME
    #[arg(short, long, value_name="FILE_NAME")]
    pub file: Option<String>,

    /// display N x N pictures per page
    #[arg(short, long, value_name="N")]
    pub grid: Option<usize>,

    /// window height (default = GALLSHHEIGHT)
    #[arg(long, value_name="N")]
    pub height: Option<i32>,

    /// show the Nth picture first
    #[arg(short, long, value_name="N")]
    pub index: Option<usize>,

    /// order pictures by Label
    #[arg(short, long, default_value_t = false)]
    pub label: bool,

    /// move selected files to TARGET_DIR
    #[arg(short, long, value_name = "TARGET_DIR")]
    pub move_selection: Option<String>,

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

    /// update picture data and thumbnails files
    #[arg(short, long, default_value_t = false)]
    pub update: bool,

    /// order pictures by Value
    #[arg(short, long, default_value_t = false)]
    pub value: bool,

    /// window width (defaults = GALLSHWIDTH)
    #[arg(short, long, value_name="N")]
    pub width: Option<i32>,
}

impl Args {
    pub fn canonize(&mut self) {
        self.directory = Some(directory(self.directory.clone()));
        if self.name {
            self.order = Order::Name;
        }
        if self.value {
            self.order = Order::Value;
        }
        if self.date {
            self.order = Order::Date;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const pgm: &str = "gsr2";

    #[test]
    fn canonize_args_set_up_the_default_dir_if_none_given() {
        let command_line: Vec<&str> = vec![pgm];
        let result = Args::try_parse_from(command_line.iter());
        assert_eq!(true, result.is_ok());
        let mut args = result.unwrap();
        args.canonize();
        let default = directory(None);
        assert_eq!(default, args.directory.unwrap());
    }

    #[test]
    fn canonize_args_fix_order_to_name_if_option_name_picked() {
        let command_line: Vec<&str> = vec![pgm, "--order", "value","--name"];
        let result = Args::try_parse_from(command_line.iter());
        assert_eq!(true, result.is_ok());
        let mut args = result.unwrap();
        args.canonize();
        assert_eq!(Order::Name, args.order);
    }
    #[test]
    fn canonize_args_fix_order_to_value_if_option_value_picked() {
        let command_line: Vec<&str> = vec![pgm, "--order", "name","--value"];
        let result = Args::try_parse_from(command_line.iter());
        assert_eq!(true, result.is_ok());
        let mut args = result.unwrap();
        args.canonize();
        assert_eq!(Order::Value, args.order);
    }
    #[test]
    fn canonize_args_fix_order_to_date_if_option_date_picked() {
        let command_line: Vec<&str> = vec![pgm, "--order","Name","-d"];
        let result = Args::try_parse_from(command_line.iter());
        assert_eq!(true, result.is_ok());
        let mut args = result.unwrap();
        args.canonize();
        assert_eq!(Order::Date, args.order);
    }
}

