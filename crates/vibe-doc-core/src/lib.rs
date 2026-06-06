//! Shared core logic for vibe-doc.
//!
//! This crate is intentionally minimal until the document model, scanner, and
//! validation tasks add behavior.

/// Stable crate identifier used by workspace smoke tests.
pub const CRATE_NAME: &str = "vibe-doc-core";

#[cfg(test)]
mod tests {
    use super::CRATE_NAME;

    #[test]
    fn exposes_core_crate_name() {
        assert_eq!(CRATE_NAME, "vibe-doc-core");
    }
}
