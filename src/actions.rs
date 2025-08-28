use crate::rank::Rank;

#[derive(Debug, Clone)]
pub enum Action {
    Label { label: String },
    Unlabel,
    AddTag { label: String },
    DeleteTag { label: String },
    Rank { rank: Rank },
    Select,
    Delete,
}
