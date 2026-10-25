//! `docs/protocols/imap-extensions.md` is generated from
//! `mw_imap::extensions::EXTENSIONS`; this test fails when the two differ.
//!
//! Regenerate with `scripts/imap/gen-matrix.sh` (or `.ps1`), which runs this
//! test with `MW_WRITE_IMAP_MATRIX=1` so it writes the document instead.

use std::path::PathBuf;

fn doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/protocols/imap-extensions.md")
}

#[test]
fn compliance_doc_matches_the_table() {
    let want = mw_imap::extensions::markdown();
    let path = doc_path();
    if std::env::var_os("MW_WRITE_IMAP_MATRIX").is_some() {
        std::fs::write(&path, &want).expect("write the compliance doc");
        return;
    }
    let have = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        have == want,
        "{} is out of date; run scripts/imap/gen-matrix.sh",
        path.display()
    );
}
