use crate::catalog::{Catalog};
use crate::completion::candidates;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum InputKind {
   AddTag, DeleteTag, Search, SearchLabel, Label, Relabel, Index, }

pub struct Editor {
    input: Option<String>,
    input_kind: Option<InputKind>,
    completion: bool,
    pub tags: HashSet<String>,
    candidates: Vec<String>,
}

impl Editor {

    pub fn new() -> Self {
        Editor {
            input: None,
            input_kind: None,
            completion: false,
            tags: HashSet::new(),
            candidates: vec![],
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
        self.completion = false
    }

    pub fn editing(&self) -> bool {
        self.input_kind.is_some()
    }

    pub fn cancel(&mut self) {
        self.input_kind = None;
        self.completion = false
    }

    pub fn candidates(&self) -> String {
        if self.completion {
            self.candidates.join(",")
        } else {
            String::from("")
        }
    }

    pub fn confirm(&mut self, catalog: &mut Catalog) {
        let input = &self.input.clone().unwrap();
        if let Some(kind) = self.input_kind.clone() {
            match kind {
                InputKind::AddTag => {
                    let _ = catalog.tag_current_entry(input);
                },
                InputKind::DeleteTag => {
                    let _ = catalog.untag_current_entry(input);
                },
                InputKind::Search => {
                    catalog.move_to_input_pattern(input);
                },
                InputKind::SearchLabel => {
                    catalog.move_to_label_pattern(input);
                },
                InputKind::Index => {
                    if let Ok(index) = input.parse::<usize>() {
                        if index < catalog.navigator().length() && catalog.navigator().can_move_to_index(index) {
                            catalog.mut_navigator().move_to_index(index)
                        }
                    }
                },
                InputKind::Label => {
                    let _ = catalog.label_current_entry(input);
                },
                InputKind::Relabel => {
                    let _ = catalog.set_selected_labels_with_input(input);
                },
            }
        }
        self.completion = false;
        self.input_kind = None;
        self.input = None
    }

    pub fn delete(&mut self) {
        self.input = self.input.clone().map (|s| {
            let mut t = s.clone();
            t.pop();
            t });
        self.completion = false;
    }

    pub fn append(&mut self, ch: char) {
        if let Some(kind) = self.input_kind.clone() {
            let ch_is_ok: bool = match kind {
                InputKind::Index => ch.is_ascii_digit(),
                InputKind::AddTag | InputKind::DeleteTag | InputKind::Label | InputKind::Relabel | InputKind::SearchLabel => matches!(ch, 'a'..='z' | '0'..='9' | '-' | '_'),
                InputKind::Search => true,
            };
            if ch_is_ok {
                self.input = self.input.clone().map( |s| {
                    let mut t = s.clone();
                    t.push(ch);
                    t
                });
                self.completion = false;
            }
        }
    }

    pub fn complete(&mut self) {
        if let Some(kind) = self.input_kind.clone(){
            if [InputKind::AddTag,InputKind::DeleteTag,InputKind::Label,InputKind::Relabel,InputKind::SearchLabel].contains(&kind) {
                if let Some(prefix) = &self.input {
                    let candidates = candidates(prefix, &self.tags);
                    match candidates.len() {
                        0 => { self.candidates = vec![] } ,
                        1 => {
                            self.input = Some(candidates[0].clone());
                            self.candidates = vec![];
                        },
                        _ => { self.candidates = candidates.clone() },
                    }
                };
                self.completion = true
            }
        }
    }
}

#[cfg(test)]

    #[test]
    fn editing_input() {
        let mut editor = Editor::new();
        assert_eq!(false, editor.editing());
        editor.begin_input(InputKind::Label, HashSet::new());
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

