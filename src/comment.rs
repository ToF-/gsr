
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
    /// Mark the picture entry as selected.
    Select,
    /// Mark the picture entry as deleted.
    Delete,
}
