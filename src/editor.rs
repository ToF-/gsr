use crate::catalog::{Catalog};
use crate::completion::candidates;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
    AddTagInput, DeleteTagInput, SearchInput, SearchLabelInput, LabelInput, RelabelInput, IndexInput, }

pub struct Editor {
    input: Option<String>,
    input_kind: Option<InputKind>,
    pub tags: HashSet<String>,
    current_candidates: Vec<String>,
}

impl Editor {

    pub fn new() -> Self {
        Editor {
            input: None,
            input_kind: None,
            tags: HashSet::new(),
            current_candidates: vec![],
        }
    }

    pub fn input(&self) -> String {
        self.input.clone().expect("input() not called properly")
    }

    pub fn input_kind(&self) -> Option<InputKind> {
        self.input_kind.clone()
    }

    pub fn begin_input(&mut self, kind: InputKind, tags: HashSet<String>) {
        self.tags = tags;
        self.input_kind = Some(kind);
        self.input = Some(String::from(""));
    }

    pub fn editing(&self) -> bool {
        self.input_kind.is_some()
    }

    pub fn cancel(&mut self) {
        self.input_kind = None;
    }

    pub fn confirm(&mut self, catalog: &mut Catalog) {
        let input = &self.input.clone().unwrap();
        if let Some(kind) = self.input_kind.clone() {
            match kind {
                InputKind::AddTagInput => {
                    let _ = catalog.tag_current_entry(input);
                },
                InputKind::DeleteTagInput => {
                    let _ = catalog.delete_tag(input);
                },
                InputKind::SearchInput => {
                    catalog.move_to_input_pattern(input);
                },
                InputKind::SearchLabelInput => {
                    catalog.move_to_label_pattern(input);
                },
                InputKind::IndexInput => {
                    if let Ok(index) = input.parse::<usize>() {
                        if index < catalog.length() && catalog.can_move_to_index(index) {
                            catalog.move_to_index(index)
                        }
                    }
                },
                InputKind::LabelInput => {
                    let _ = catalog.label_current_entry(input);
                },
                InputKind::RelabelInput => {
                    let _ = catalog.set_selected_labels_with_input(input);
                },
            }
        }
        self.input_kind = None;
        self.input = None
    }

    pub fn delete(&mut self) {
        self.input = self.input.clone().map (|s| {
            let mut t = s.clone();
            t.pop();
            t });
    }

    pub fn append(&mut self, ch: char) {
        if let Some(kind) = self.input_kind.clone() {
            let ch_is_ok: bool = match kind {
                InputKind::IndexInput => {
                    match ch {
                        '0'..='9' => true,
                        _ => false,
                    }
                },
                InputKind::AddTagInput|InputKind::DeleteTagInput|InputKind::LabelInput|InputKind::RelabelInput|InputKind::SearchLabelInput => {
                    match ch {
                        'a'..='z' => true,
                        '0'..='9' => true,
                        '-'|'_' => true,
                        _ => false,
                    }
                },
                InputKind::SearchInput => true,
            };
            if ch_is_ok {
                self.input = self.input.clone().map( |s| {
                    let mut t = s.clone();
                    t.push(ch);
                    t
                });
            }
        }
    }

    pub fn complete(&mut self) {
        if let Some(kind) = self.input_kind.clone(){
            if [InputKind::AddTagInput,InputKind::DeleteTagInput,InputKind::LabelInput,InputKind::RelabelInput,InputKind::SearchLabelInput].contains(&kind) {
                match &self.input {
                    Some(prefix) => {
                        let candidates = candidates(prefix, &self.tags);
                        match candidates.len() {
                            0 => { self.current_candidates = vec![] } ,
                            1 => {
                                self.input = Some(candidates[0].clone());
                                self.current_candidates = vec![];
                            },
                            _ => { self.current_candidates = candidates.clone() },
                        }
                    },
                    None => {},
                }
            }
        }
    }
}

#[cfg(test)]

    #[test]
    fn editing_input() {
        let mut editor = Editor::new();
        assert_eq!(false, editor.editing());
        editor.begin_input(InputKind::LabelInput, HashSet::new());
        assert_eq!(true, editor.editing());
        editor.append('f');
        editor.append('o');
        editor.append('o');
        editor.append('-');
        assert_eq!(String::from("foo-"), editor.input.clone().unwrap());
        editor.delete();
        assert_eq!(String::from("foo"), editor.input.clone().unwrap());
        editor.cancel();
        assert_eq!(false, editor.editing());
    }

