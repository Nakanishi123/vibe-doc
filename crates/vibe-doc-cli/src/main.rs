//! Command-line entry point for `vdoc`.
//!
//! Command behavior is added by later implementation tasks.

fn main() {
    let _ = vibe_doc_core::CRATE_NAME;
}

#[cfg(test)]
mod tests {
    #[test]
    fn depends_on_core_crate() {
        assert_eq!(vibe_doc_core::CRATE_NAME, "vibe-doc-core");
    }
}
