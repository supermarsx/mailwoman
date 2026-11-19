//! Regenerate the settings reference without compiling the HTTP server.
fn main() -> std::io::Result<()> {
    let reference = mw_settings::knobs::render_docs();
    if let Some(path) = std::env::args_os().nth(1) {
        std::fs::write(path, reference)
    } else {
        print!("{reference}");
        Ok(())
    }
}
