use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::process::exit;
use std::env;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

const KEY_CMD_FILE_VAR: &str = "GALLSHKEY";

#[derive(PartialEq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Command {
    Cancel,
    ChooseOrder,
    CopyAndQuit,
    CopyLabel,
    CopyTemp,
    Delete,
    Down,
    EndPosition,
    ExportCommands,
    FirstPosition,
    GotoIndex,
    Info,
    Label,
    LastPosition,
    Left,
    Next,
    NextPage,
    NoStar,
    OneStar,
    PasteLabel,
    PrevPage,
    Quit,
    Random,
    Right,
    Search,
    SetRange,
    StartPosition,
    ThreeStars,
    ToggleExpand,
    ToggleFullSize,
    TogglePageLimit,
    TogglePalette,
    ToggleSelect,
    ToggleSingleView,
    TwoStars,
    UnSelectPage,
    Unlabel,
    UnselectAll,
    Up,
}

pub type Shortcuts = HashMap<String, Command>;

pub fn default_shortcuts() -> Shortcuts {
    let shortcuts: Shortcuts = HashMap::from([
        (String::from("Escape"), Command::Cancel),
        (String::from("equal"), Command::ChooseOrder),
        (String::from("Q"), Command::CopyAndQuit),
        (String::from("c"), Command::CopyLabel),
        (String::from("C"), Command::CopyTemp),
        (String::from("D"), Command::Delete),
        (String::from("Down"), Command::Down),
        (String::from("s"), Command::Down),
        (String::from("Z"), Command::EndPosition),
        (String::from("a"), Command::FirstPosition),
        (String::from("G"), Command::GotoIndex),
        (String::from("I"), Command::Info),
        (String::from("l"), Command::Label),
        (String::from("z"), Command::LastPosition),
        (String::from("Left"), Command::Left),
        (String::from("t"), Command::Left),
        (String::from("space"), Command::Next),
        (String::from("n"), Command::NextPage),
        (String::from("0"), Command::NoStar),
        (String::from("dollar"), Command::NoStar),
        (String::from("1"), Command::OneStar),
        (String::from("quotedbl"), Command::OneStar),
        (String::from("plus"), Command::PasteLabel),
        (String::from("p"), Command::PrevPage),
        (String::from("q"), Command::Quit),
        (String::from("R"), Command::Random),
        (String::from("Right"), Command::Right),
        (String::from("r"), Command::Right),
        (String::from("S"), Command::Search),
        (String::from("Return"), Command::SetRange),
        (String::from("A"), Command::StartPosition),
        (String::from("3"), Command::ThreeStars),
        (String::from("guillemotright"), Command::ThreeStars),
        (String::from("e"), Command::ToggleExpand),
        (String::from("f"), Command::ToggleFullSize),
        (String::from("o"), Command::TogglePageLimit),
        (String::from("x"), Command::TogglePalette),
        (String::from("comma"), Command::ToggleSelect),
        (String::from("period"), Command::ToggleSingleView),
        (String::from("2"), Command::TwoStars),
        (String::from("guillemotleft"), Command::TwoStars),
        (String::from("u"), Command::UnSelectPage),
        (String::from("minus"), Command::Unlabel),
        (String::from("U"), Command::UnselectAll),
        (String::from("Up"), Command::Up),
        (String::from("d"), Command::Up),
        (String::from("K"), Command::ExportCommands)]);
    shortcuts
}

pub fn load_shortcuts() -> Shortcuts {
    let gallshkey = env::var(KEY_CMD_FILE_VAR);
    if let Ok(key_file_name) = &gallshkey {
        match read_to_string(key_file_name) {
            Ok(content) => match serde_json::from_str(&content) {
                    Err(err) => {
                        eprintln!("{}",&format!("can't deserialize file {} : error: {}", key_file_name, err));
                        exit(1)
                    },
                    Ok(shortcuts) => shortcuts,
            },
            Err(_) => default_shortcuts(),
        }
    } else {
        default_shortcuts()
    }
}

pub fn export_shortcuts(shortcuts: &Shortcuts) {
    let content = serde_json::to_string(shortcuts);
    let path = Path::new("./gallshkey.json");
    match File::create(path) {
        Ok(file) => {
            match serde_json::to_writer(file, &shortcuts) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("{}",format!("error while creating ./gallshkey.json : {}", err));
                },
            }
        },
            Err(err) => {
                eprintln!("{}",format!("error while creating ./gallshkey.json : {}", err));
            },
        }
    }
