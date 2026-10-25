//! Generated reference and structural catalogue invariants.
#[test]
fn generated_reference_matches_catalogue() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/admin/configuration.md");
    let rendered = mw_server::knobs::render_docs();
    if std::env::var("BLESS").as_deref() == Ok("1") {
        std::fs::write(&path, &rendered).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(path).expect("generate with BLESS=1"),
        rendered
    );
}

fn literal_env_reads(source: &str) -> Vec<&str> {
    let mut result = Vec::new();
    for (offset, _) in source.match_indices("\"MW_") {
        let prefix = source[..offset].trim_end();
        if !["var(", "var_os(", "option_env!(", "env!(", "env ="]
            .iter()
            .any(|p| prefix.ends_with(p))
        {
            continue;
        }
        let Some(end) = source[offset + 1..].find('"') else {
            continue;
        };
        result.push(&source[offset + 1..offset + 1 + end]);
    }
    result
}

#[test]
fn production_environment_reads_are_catalogued() {
    fn visit(path: &std::path::Path, unknown: &mut Vec<String>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, unknown)
            } else if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).unwrap();
                let production = source.split("#[cfg(test)]").next().unwrap();
                for key in literal_env_reads(production) {
                    if !mw_server::knobs::known_env_vars().any(|known| known == key) {
                        unknown.push(format!("{}: {key}", path.display()));
                    }
                }
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut unknown = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let src = entry.unwrap().path().join("src");
        if src.is_dir() {
            visit(&src, &mut unknown);
        }
    }
    assert!(
        unknown.is_empty(),
        "uncatalogued environment reads: {unknown:#?}"
    );
    // A synthetic newly introduced knob must be detected independently of the catalogue.
    assert_eq!(
        literal_env_reads("std::env::var(\"MW_FOO\")"),
        vec!["MW_FOO"]
    );
    assert!(!mw_server::knobs::known_env_vars().any(|v| v == "MW_FOO"));
}
