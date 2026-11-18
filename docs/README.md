# Mailwoman documentation

Start here. The [browsable manual](./index.md) includes a topic sidebar, local full-text search and a dark-first theme.

To preview it locally, run `pnpm install` then `pnpm dev` from `docs/`.
`pnpm build` creates the static site in `.vitepress/dist`. Set `DOCS_BASE` when
hosting under a subdirectory (for example `/mailwoman/` on GitHub Pages).

## For users

- [Sync, offline access and multiple windows](./user/sync-offline.md).

## For operators

- [`deploy/`](./deploy/README.md) — installing, reverse proxies, TLS, Postgres,
  caching, push, packaging, the desktop and Android shells, and
  [`deploy/egress-proxy.md`](./deploy/egress-proxy.md) for routing outbound
  fetches through your own proxy.
- [`security/`](./security/README.md) — the security model, one page per surface.
  [`security/egress.md`](./security/egress.md) states what the outbound-fetch
  policy guarantees **and what it does not**, which is the more useful half.
- [`admin/`](./admin/index.md) — one page per administrator setting area. Newer pages:
  [`compliance.md`](./admin/compliance.md), [`breach-register.md`](./admin/breach-register.md),
  [`consent.md`](./admin/consent.md), [`cookies.md`](./admin/cookies.md),
  [`data-requests.md`](./admin/data-requests.md), [`erasure-coverage.md`](./admin/erasure-coverage.md),
  [`retention.md`](./admin/retention.md), [`encryption.md`](./admin/encryption.md),
  [`sessions.md`](./admin/sessions.md), [`request-protection.md`](./admin/request-protection.md),
  [`metrics-persistence.md`](./admin/metrics-persistence.md), [`observability.md`](./admin/observability.md),
  [`composer-features.md`](./admin/composer-features.md); the settings and configuration references are
  [`settings.md`](./admin/settings.md) and [`configuration.md`](./admin/configuration.md).
- [`bridges/`](./bridges/index.md) — Graph / EWS / Gmail / PIM bridges.
- [`integrations/nextcloud.md`](./integrations/nextcloud.md),
  [`export/msg-oft-docx.md`](./export/msg-oft-docx.md),
  [`assist.md`](./assist.md) (AI privacy and governance).

## For contributors

- **[`engineering-practices.md`](./engineering-practices.md)** — the rules this
  project arrived at by getting them wrong first: why a check that cannot see the
  thing returns a clean-looking answer, the shared-tree git and `cargo fmt` rules,
  why a struct holding a secret writes its own `Debug`, the flake policy, the
  retired migration number, and how to write an assertion that means something.
  **Read this before your first change.**
- [`testing/coverage.md`](./testing/coverage.md),
  [`testing/mutation.md`](./testing/mutation.md) — the coverage ratchet and
  mutation harness.
- [`a11y-manual-checklist.md`](./a11y-manual-checklist.md),
  [`i18n.md`](./i18n.md).
- [`perf/`](./perf/index.md) — build-profile and size-budget measurements, with their
  methods stated.

## Project-level

- [`../SPEC.md`](https://github.com/supermarsx/mailwoman/blob/master/SPEC.md) — the product and architecture specification,
  including what is **cut** or **specced-not-built**, which it says out loud.
- [`../VERSIONING.md`](https://github.com/supermarsx/mailwoman/blob/master/VERSIONING.md) — the rolling `YY.N` scheme and the
  **release history**, where each entry records what a release was believed to
  deliver at the time, with later corrections appended rather than rewritten.
- [`../SECURITY.md`](https://github.com/supermarsx/mailwoman/blob/master/SECURITY.md) — reporting a vulnerability.
- [`ROADMAP-1.0.md`](./ROADMAP-1.0.md).
