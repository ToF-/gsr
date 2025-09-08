
use crate::rank::Rank;

/// Comment one can apply to picture entries in the gallery
#[derive(Debug, Clone)]
pub enum Comment {
    /// Put a label on a picture entry
    Label { label: String },
    /// Remove the label on a picture entry
    Unlabel,
    /// Add a tag on a picture entry. A given tag can only appear once.
    AddTag { label: String },
    /// Delete the given tag on the picture entry.
    DeleteTag { label: String },
    /// Rank the picture entry.
    Rank { rank: Rank },
    /// Make the picture a cover for the directory
    Cover,
    /// Unmake the picture a cover for the directory
    Uncover,
    /// Mark the picture entry as selected.
    ToggleSelect,
    /// Mark the picture entry as deleted.
    ToggleDelete,
}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
