# About, software inventories and CRA preparation

Settings → About → About Mailwoman, and Administration → Compliance → About
Mailwoman, show generated build facts, security policy, support information,
vulnerability-reporting and advisory links, and downloadable software inventories.
No remote advisory feed or update service is contacted when this view loads.

## Automatic build outputs

Every server build generates an embedded CycloneDX 1.6 inventory from the current
Cargo.lock: identities, versions, registry archive checksums, dependency references,
a lockfile digest and coverage properties. Public read-only build facts are at
`/api/product-security`, the inventory at `/api/product-security/sbom`, and the
embedded SECURITY.md at `/api/product-security/policy`. Deployment settings,
credentials and local filesystem paths are excluded.

This is a **workspace lock inventory**, including development and target-specific
packages; it is not an exact attestation of packages linked into one binary.
Checksums identify registry archives, not compiled contents or vendored changes.
Native libraries, containers, plugins, vendored assets and packaging components
need separate coverage.

Every Vite production build emits `software-bom.frontend.json`, covering npm
packages whose code or assets were observed in that build, including workers.
Package-local paths are excluded. The third-party licence inventory remains
available separately; it is not a fresh vulnerability assessment.

Backend build facts include version, source revision, exact tag when available,
compiler target, lock hashes, security-policy hash and SBOM digest. A source
revision is not a signature or proof of a clean checkout. SOURCE_DATE_EPOCH can
declare reproducible build time. Build-time MW_PRODUCT_SUPPORT_UNTIL declares an
actual YYYY-MM-DD support-end date (also accepted as a Docker build argument);
otherwise the UI shows not declared. The
project policy supports the latest tagged rolling release. A fixed CRA support
commitment is not inferred from that policy or an open-source licence.

## CI artifacts

The Rust job runs `node scripts/about/export-product-security.mjs --out <dir>`.
It uses the exact package ID and build-script output directory from Cargo,
validates source/inventory digests and dependency references, then exports
product-security.json, software-bom.cdx.json, SECURITY.md and SHA256SUMS. It never
selects an arbitrary old target directory. The web job uploads its newly generated
frontend inventory. These are CI artifacts; release publication and signing are
separate. The export command also works locally after dependencies are present;
it uses locked, offline Cargo resolution.

## CRA boundaries

These outputs support transparency and vulnerability handling. They do not
establish a complete technical file, conformity assessment, CE declaration,
minimum support commitment, regulatory reporting or a distributor/steward's legal
role. Advisories and releases must still be authored and published. About links
to the real channels and never manufactures a no-vulnerabilities claim.

References: [CRA summary](https://digital-strategy.ec.europa.eu/en/policies/cra-summary),
[open-source roles](https://digital-strategy.ec.europa.eu/en/policies/cra-open-source).
