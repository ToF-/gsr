use anyhow::Result;
use clap::Parser;
use std::env;
use crate::order::Order;
use crate::path::{ABSOLUTE_PATH, check_file, check_reading_list_file, check_path, default_extract_list_file};

const DEFAULT_WIDTH: i32   = 1000;
const DEFAULT_HEIGHT: i32  = 1000;
const WIDTH_ENV_VAR :&str  = "GALLSHWIDTH";
const HEIGHT_ENV_VAR :&str = "GALLSHHEIGHT";

#[derive(Parser, Clone, Debug)]
/// Gallery Show
#[command(about("a picture viewer from terminal"), author("ToF"), version, infer_long_args = true, infer_subcommands = true, help_template("\
{before-help}{name} {version} {about} by {author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
"))]
pub struct Args {
     /// Directory to search (default is set with variable GALLSHDIR)
    pub directory: Option<String>,

    /// display only FILE_NAME
    #[arg(short, long, value_name="FILE_NAME")]
    pub file: Option<String>,

    /// display files that only contain STRING in their name
    #[arg(short, long, value_name="STRING")]
    pub pattern: Option<String>,

    /// display files that match the query QUERY
    #[arg(short, long, value_name="QUERY")]
    pub query: Option<String>,

    /// select pictures having tags in the given list
    #[arg(long, value_name="TAG_LIST")]
    pub select: Option<Vec<String>>,

    /// select pictures having all the tags in the given list
    #[arg(long, value_name="TAG_LIST")]
    pub include: Option<Vec<String>>,

    /// list all directories of pictures in the database
    #[arg(long, default_value_t = false)]

    pub directories: bool,
    /// display pictures in order
    #[arg(short, long, value_name="ORDER", ignore_case(true), default_value_t = Order::Random)]
    pub order: Order,

    /// order pictures by Date
    #[arg(short, long, default_value_t = false)]
    pub date: bool,

    /// order pictures by Name
    #[arg(short, long, default_value_t = false)]
    pub name: bool,

    /// order pictures by Value
    #[arg(short, long, default_value_t = false)]
    pub value: bool,

    /// show only cover pictures of each directory
    #[arg(long, default_value_t = false)]
    pub covers: bool,

    /// extract list of selected files to FILE
    #[arg(short, long, value_name="FILE")]
    pub list_extract: Option<String>,

    /// show information about this folder
    #[arg(long)]
    pub info: bool,

    /// list all the tags attached to pictures
    #[arg(long, default_value_t = false)]
    pub tags: bool,

    /// display N x N pictures per page
    #[arg(short, long, value_name="N")]
    pub grid: Option<usize>,

    /// show thumbnails only, in a 10x10 grid
    #[arg(short, long, default_value_t = false)]
    pub thumbnails: bool,

    /// window height (default = GALLSHHEIGHT)
    #[arg(long, value_name="N")]
    pub height: Option<i32>,

    /// window width (defaults = GALLSHWIDTH)
    #[arg(short, long, value_name="N")]
    pub width: Option<i32>,

    /// wait N seconds between each picture
    #[arg(short, long, value_name="N")]
    pub seconds: Option<u64>,

    /// label all unlabeled pictures in the set with LABEL
    #[arg(long, value_name="LABEL")]
    pub label: Option<String>,

    /// add picture files data in DIRECTORY to the database
    #[arg(long, value_name = "DIRECTORY")]
    pub add_files: Option<String>,

    /// import pictures from DIRECTORY in directory specified with add-files, in a 10x10 grid
    #[arg(long, value_name="DIRECTORY")]
    pub from_files: Option<String>,

    /// checks default directory for new pictures
    #[arg(long, default_value_t = false)]
    pub check: bool,

    /// create the schema for the database
    #[arg(long, default_value_t = false)]
    pub create_schema: bool,

    /// move all duplicate files to TARGET_DIR
    #[arg(value_name = "TARGET_DIR")]
    pub deduplicate: Option<String>,

    /// move selected files to TARGET_DIR
    #[arg(long, value_name = "TARGET_DIR")]
    pub move_selection: Option<String>,

    /// remove entries from the database when file no longer exits
    #[arg(long, default_value_t = false)]
    pub purge: bool,

    /// retarget selected pictures with labels to directory TARGET/<LABEL>
    #[arg(long, value_name="TARGET_DIR")]
    pub redirect: Option<String>,

    /// update picture data and thumbnails files
    #[arg(long, default_value_t = false)]
    pub update: bool,

}

impl Args {

    pub fn checked_args(&mut self) -> Result<Args> {
        let result: Args = Args {

            add_files: match &self.add_files {
                None => None,
                Some(dir) => match check_path(dir, ABSOLUTE_PATH) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                }
            },
            check: self.check,

            covers: self.covers,
            create_schema: self.create_schema,

            date: self.date,

            deduplicate: {
                match &self.deduplicate {
                    Some(dir) => match check_path(dir, ! ABSOLUTE_PATH) {
                        Ok(_) => Some(dir.to_string()),
                        Err(err) => return Err(err),
                    },
                    None => None,
                }
            },

            directory: {
                match &self.directory {
                    Some(dir) => match check_path(dir, ! ABSOLUTE_PATH) {
                        Ok(_) => Some(dir.to_string()),
                        Err(err) => return Err(err),
                    }
                    None => None,
                }
            },

            directories: self.directories,

            list_extract: match &self.list_extract {
                None => {
                    match default_extract_list_file() {
                        Ok(file_name) => match check_reading_list_file(&file_name) {
                            Ok(path) => Some(path.display().to_string()),
                            Err(err) => return Err(err),
                        },
                        Err(err) => return Err(err),
                    }
                },
                Some(path) => match check_reading_list_file(path) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            file: match &self.file {
                None => None,
                Some(path) => match check_file(path) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            from_files: match &self.from_files {
                None => None,
                Some(dir) => match check_path(dir, ! ABSOLUTE_PATH) {
                    Ok(_) => Some(dir.to_string()),
                    Err(err) => return Err(err),
                }
            },

            grid: match self.grid {
                None if !self.thumbnails => Some(1),
                None if self.thumbnails => Some(10),
                Some(n) if !self.thumbnails && n <= 10 => Some(n),
                _ => Some(1),
            },

            height: Some(dimension(self.height, HEIGHT_ENV_VAR, "height", DEFAULT_HEIGHT)),

            include: match self.include.clone() {
                Some(list) => if !list.is_empty() {
                    let tags:Vec<String> = list[0].split(' ').map(|s| s.into()).filter(|s:&String| !s.is_empty()).collect();
                    if !tags.is_empty() {
                        Some(tags)
                    } else {
                        None
                    }
                } else {
                    None
                },
                    None => None,
            },


            info: self.info,

            label: self.label.clone(),

            move_selection: match &self.move_selection {
                None => None,
                Some(dir) => match check_path(dir, ABSOLUTE_PATH) {
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
                self.order
            },

            pattern: self.pattern.clone(),

            purge: self.purge,

            query: self.query.clone(),

            redirect: match &self.redirect {
                None => None,
                Some(path) => match check_path(path, true) {
                    Ok(_) => Some(path.to_string()),
                    Err(err) => return Err(err),
                },
            },

            seconds: self.seconds,

            select: match self.select.clone() {
                Some(list) => if !list.is_empty() {
                    let tags:Vec<String> = list[0].split(' ').map(|s| s.into()).filter(|s:&String| !s.is_empty()).collect();
                    if !tags.is_empty() {
                        Some(tags)
                    } else {
                        None
                    }
                } else {
                    None
                },
                    None => None,
            },

            tags: self.tags,

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

    const PGM: &str = "gsr";

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

