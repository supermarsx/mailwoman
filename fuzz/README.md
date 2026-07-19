# Fuzz targets

Coverage-guided fuzz harnesses for Mailwoman's parsers that consume untrusted,
network- or attacker-controlled bytes. Every target asserts the same contract:
the parser must never panic on arbitrary input.

| Target | Crate | Input |
|--------|-------|-------|
| `mime_parse` | `mw-mime` | raw RFC822 message bytes |
| `sanitize_html` | `mw-sanitize` | untrusted HTML email bodies |
| `imap_parse_response` | `mw-imap` | IMAP server responses off the wire |
| `pop3_parse` | `mw-pop3` | POP3 CAPA/UIDL/LIST bodies off the wire |
| `sieve_parse` | `mw-sieve` | user-supplied Sieve scripts |

## Running

Requires a nightly toolchain and `cargo-fuzz` (`cargo install cargo-fuzz`).
libFuzzer's sanitizer runtime is supported on Linux and macOS; on Windows run
the targets through the container/CI path instead.

```sh
# Continuous run (Ctrl-C to stop):
cargo +nightly fuzz run mime_parse

# Bounded smoke pass (what CI runs on every push):
cargo +nightly fuzz run mime_parse -- -max_total_time=60 -runs=100000
```

Discovered corpus entries live under `corpus/<target>/`; a crash reproducer is
written to `artifacts/<target>/` — both are git-ignored. Commit a minimized
reproducer as a regression test in the owning crate when a bug is found.

## Layout

This directory is a standalone cargo package with its own empty `[workspace]`
table, so `cargo` at the repository root ignores the sanitizer-instrumented
build. It is intentionally **not** a member of the main workspace.
