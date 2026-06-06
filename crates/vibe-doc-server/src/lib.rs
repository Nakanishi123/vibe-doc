//! API server and SPA host crate for vibe-doc.
//!
//! Server behavior is added by later implementation tasks.

#[cfg(test)]
mod tests {
    #[test]
    fn depends_on_core_crate() {
        assert_eq!(vibe_doc_core::CRATE_NAME, "vibe-doc-core");
    }
}
