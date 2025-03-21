use anyhow::Result;
use clap::Parser;
use std::env;
use crate::order::Order;
use crate::path::{directory, check_file, check_reading_list_file, check_path, default_extract_list_file};

const DEFAULT_WIDTH: i32   = 1000;
const DEFAULT_HEIGHT: i32  = 1000;
const WIDTH_ENV_VAR :&str  = "GALLSHWIDTH";
const HEIGHT_ENV_VAR :&str = "GALLSHHEIGHT";

#[derive(Parser, Clone, Debug)]
#[command(infer_long_args = true, infer_subcommands = true)]
/// Gallery Show
pub struct Args {
    
    /// copy representative pictures of each subfolders in CATALOG_DIR
    #[arg(long, value_name = "CATALOG_DIR")]
    pub catalog: Option<String>,

     /// Directory to search (default is set with variable GALLSHDIR)
    pub directory: Option<String>,

    /// move all duplicate files to TARGET_DIR
    #[arg(long, value_name = "TARGET_DIR")]
    pub deduplicate: Option<String>,

    /// move all labelled pictures to their matching folder on TARGET_DIR
    #[arg(short, long, value_name = "TARGET_DIR")]
    pub all_move: Option<String>,

    /// copy selected files to TARGET_DIR
    #[arg(short, long, value_name = "TARGET_DIR")]
    pub copy_selection: Option<String>,

    /// order pictures by Date
    #[arg(short, long, default_value_t = false)]
    pub date: bool,

    /// extract list
    #[arg(short, long, value_name="FILE_NAME")]
    pub extract: Option<String>,

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

    /// show information about this folder
    #[arg(long)]
    pub info: bool,

    /// label all pictures in the set
    #[arg(short, long)]
    pub label: Option<String>,

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

    /// sample pictures from each directory within of size N between 2 and 8
    #[arg(long, value_name="N")]
    pub sample: Option<usize>,

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

    pub fn checked_args(&mut self) -> Result<Args> {
        let result: Args = Args {
            catalog: match &self.catalog {
                None => None,
                Some(dir) => match check_path(&dir) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                },
            },

            all_move: match &self.all_move {
                None => None,
                Some(dir) => match check_path(&dir) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                },
            },

            copy_selection: match &self.copy_selection {
                None => None,
                Some(dir) => match check_path(&dir) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                },
            },

            date: self.date,

            deduplicate: {
                match self.deduplicate.clone() {
                    Some(dir) => match check_path(&dir) {
                        Ok(_) => Some(dir.to_string()),
                        Err(err) => return Err(err),
                    },
                    None => None,
                }
            },

            directory: {
                let dir = directory(self.directory.clone());
                match check_path(&dir) {
                    Ok(_) => Some(dir),
                    Err(err) => return Err(err),
                }
            },

            extract: match &self.extract {
                None => {
                    match default_extract_list_file() {
                        Ok(file_name) => match check_reading_list_file(&file_name) {
                            Ok(path) => Some(path.display().to_string()),
                            Err(err) => return Err(err),
                        },
                        Err(err) => return Err(err),
                    }
                },
                Some(path) => match check_reading_list_file(&path) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            file: match &self.file {
                None => None,
                Some(path) => match check_file(&path) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            grid: match self.grid {
                None if !self.thumbnails => Some(1),
                None if self.thumbnails => Some(10),
                Some(n) if !self.thumbnails && n <= 10 => Some(n),
                _ => Some(1),
            },

            height: Some(dimension(self.height, HEIGHT_ENV_VAR, "height", DEFAULT_HEIGHT)),

            index: self.index,

            info: self.info,

            label: self.label.clone(),

            move_selection: match &self.move_selection {
                None => None,
                Some(dir) => match check_path(&dir) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                },
            },

            name: self.name,

            order: if self.name {
                Order::Name
            } else if self.value {
                Order::Value
            } else if self.date {
                Order::Date
            } else {
                self.order.clone()
            },

            pattern: self.pattern.clone(),

            reading: match &self.reading {
                None => None,
                Some(path) => match check_reading_list_file(&path) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            sample: match self.sample {
                None => None,
                Some(n) if n >= 2 && n <= 10 => Some(n),
                Some(_) => Some(2),
            },

            seconds: self.seconds,

            thumbnails: self.thumbnails,

            update: self.update,

            value: self.value,

            width: Some(dimension(self.width, WIDTH_ENV_VAR, "width", DEFAULT_WIDTH)),
        };
        Ok(result)
    }
}

fn dimension(source: Option<i32>, var_name: &str, dimension_name: &str, default: i32) -> i32 {
    let candidate = match source {
        Some(n) => n,
        None => match env::var(var_name) {
            Ok(s) => match s.parse::<i32>() {
                Ok(n) => n,
                _ => {
                    println!("illegal {} value: {}, setting to default", dimension_name, s);
                    default
                }
            },
            _ => {
                default
            }
        }
    };
    if candidate < 3000 && candidate > 100 {
        candidate
    } else {
        println!("illegal {} value: {}, setting to default", dimension_name, candidate);
        default
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const PGM: &str = "gsr2";

    fn my_checked_args(command_line: Vec<&str>) -> Result<Args> {
        let result = Args::try_parse_from(command_line.iter());
        let mut args = result.unwrap();
        args.checked_args()
    }

    #[test]
    fn checked_args_set_order_to_name_if_option_name_picked() {
        let args = my_checked_args(vec![PGM, "--order", "value","--name"]);
        println!("{:?}", args);
        assert_eq!(Order::Name, args.unwrap().order);
    }
    #[test]
    fn checked_args_set_order_to_value_if_option_value_picked() {
        let args = my_checked_args(vec![PGM, "--order", "name","--value"]);
        assert_eq!(Order::Value, args.unwrap().order);
    }
    #[test]
    fn checked_args_set_order_to_date_if_option_date_picked() {
        let args = my_checked_args(vec![PGM, "--order","Name","-d"]);
        assert_eq!(Order::Date, args.unwrap().order);
    }
    #[test]
    fn checked_args_wont_accept_a_wrong_directory() {
        let args = my_checked_args(vec![PGM,"/foo"]);
        println!("{:?}", args);
        assert_eq!(false, args.is_ok());
    }
    #[test]
    fn checked_args_wont_accept_a_wrong_all_move_target() {
        let args = my_checked_args(vec![PGM,"-a","/foo"]);
        println!("{:?}", args);
        assert_eq!(false, args.is_ok());
    }
    #[test]
    fn checked_args_wont_accept_a_wrong_copy_selection_target() {
        let args = my_checked_args(vec![PGM,"-c","/foo"]);
        println!("{:?}", args);
        assert_eq!(false, args.is_ok());
    }
    #[test]
    fn checked_args_wont_accept_a_wrong_move_selection_target() {
        let args = my_checked_args(vec![PGM,"-m","/foo"]);
        println!("{:?}", args);
        assert_eq!(false, args.is_ok());
    }
    #[test]
    fn checked_args_default_grid_size_is_one() {
        let args = my_checked_args(vec![PGM]).unwrap();
        assert_eq!(1, args.grid.unwrap());
    }
    #[test]
    fn checked_args_thumbnails_equals_grid_10() {
        let args = my_checked_args(vec![PGM, "-t"]).unwrap();
        println!("{:?}", args);
        assert_eq!(10, args.grid.unwrap());
    }
}

