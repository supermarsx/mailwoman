// The desktop entry point (used for `cargo run` on the host during development).
// On Android/iOS the app is launched through the `mobile_entry_point` in lib.rs.
fn main() {
    mailwoman_mobile_lib::run();
}
