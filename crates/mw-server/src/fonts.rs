//! `mailwoman fonts pull` (plan §9, §3 e10): self-host Google Fonts so the app
//! can ship `font-src 'self'` with no third-party font CDN.
//!
//! Google Fonts already serves **per-`unicode-range` subsetted** `woff2` files —
//! one `@font-face` block per script (latin, latin-ext, cyrillic, …). Pulling
//! therefore means: fetch the `css2` stylesheet, download each block's already
//! range-subset `woff2`, write them under `fonts/`, and rewrite the stylesheet's
//! `url()`s to origin-relative paths. We preserve Google's subsetting rather than
//! re-subsetting glyphs (no font-manipulation dependency needed).
//!
//! The parser/rewriter are pure and unit-tested; the network is behind the
//! [`FontSource`] trait so CI runs entirely over a recorded fixture (no live
//! fetch).

use std::future::Future;
use std::path::PathBuf;

use anyhow::{Context, anyhow};

/// One parsed `@font-face` rule from a Google Fonts stylesheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontFace {
    pub family: String,
    pub style: String,
    pub weight: String,
    pub unicode_range: Option<String>,
    /// The `/* latin */`-style subset label preceding the rule, if any.
    pub subset: Option<String>,
    /// The remote `woff2` URL from `src: url(..) format('woff2')`.
    pub src_url: String,
}

impl FontFace {
    /// A deterministic, filesystem-safe local filename for this face's `woff2`.
    pub fn local_name(&self) -> String {
        let fam = sanitize(&self.family);
        let subset = self
            .subset
            .as_deref()
            .map(sanitize)
            .unwrap_or_else(|| "all".into());
        let weight = sanitize(&self.weight);
        let style = sanitize(&self.style);
        format!("{fam}-{subset}-{weight}-{style}.woff2")
    }
}

fn sanitize(s: &str) -> String {
    s.trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Parse every `@font-face` rule (with its preceding subset comment) from a
/// Google Fonts `css2` stylesheet.
pub fn parse_font_faces(css: &str) -> Vec<FontFace> {
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = css[search_from..].find("@font-face") {
        let at = search_from + rel;
        // Nearest preceding `/* .. */` becomes the subset label.
        let subset = css[..at]
            .rfind("/*")
            .and_then(|s| {
                css[s + 2..at]
                    .find("*/")
                    .map(|e| css[s + 2..s + 2 + e].trim().to_string())
            })
            .filter(|s| !s.is_empty());

        let Some(open_rel) = css[at..].find('{') else {
            break;
        };
        let open = at + open_rel;
        let Some(close_rel) = css[open..].find('}') else {
            break;
        };
        let close = open + close_rel;
        let block = &css[open + 1..close];
        search_from = close + 1;

        if let Some(face) = parse_block(block, subset) {
            out.push(face);
        }
    }
    out
}

fn parse_block(block: &str, subset: Option<String>) -> Option<FontFace> {
    let mut family = None;
    let mut style = "normal".to_string();
    let mut weight = "400".to_string();
    let mut unicode_range = None;
    let mut src_url = None;

    for decl in block.split(';') {
        let Some((prop, value)) = decl.split_once(':') else {
            continue;
        };
        let prop = prop.trim().to_ascii_lowercase();
        let value = value.trim();
        match prop.as_str() {
            "font-family" => family = Some(value.trim_matches(['\'', '"']).to_string()),
            "font-style" => style = value.to_string(),
            "font-weight" => weight = value.to_string(),
            "unicode-range" => unicode_range = Some(value.to_string()),
            "src" => src_url = extract_woff2_url(value),
            _ => {}
        }
    }
    Some(FontFace {
        family: family?,
        style,
        weight,
        unicode_range,
        subset,
        src_url: src_url?,
    })
}

/// Pull the first `url(..)` associated with a woff2 `src` value.
fn extract_woff2_url(src: &str) -> Option<String> {
    let start = src.find("url(")? + 4;
    let rest = &src[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().trim_matches(['\'', '"']).to_string())
}

/// Rewrite each remote `woff2` `url()` to an origin-relative path so the served
/// stylesheet honours `font-src 'self'`.
pub fn rewrite_css(css: &str, faces: &[FontFace], url_prefix: &str) -> String {
    let prefix = url_prefix.trim_end_matches('/');
    let mut out = css.to_string();
    for face in faces {
        let local = format!("{prefix}/{}", face.local_name());
        out = out.replace(&face.src_url, &local);
    }
    out
}

// ---------------------------------------------------------------------------
// Fetch source (network in prod; a recorded fixture in tests)
// ---------------------------------------------------------------------------

/// Where the stylesheet + woff2 payloads come from. Abstracted so CI never hits
/// the network (plan §3 e10 acceptance).
pub trait FontSource {
    fn stylesheet(
        &self,
        families: &[String],
        text: Option<&str>,
    ) -> impl Future<Output = anyhow::Result<String>> + Send;

    fn woff2(&self, url: &str) -> impl Future<Output = anyhow::Result<Vec<u8>>> + Send;
}

/// The live Google Fonts `css2` endpoint. A browser-like `User-Agent` is
/// required or Google serves legacy `ttf` instead of `woff2`.
pub struct GoogleFonts {
    client: reqwest::Client,
}

impl GoogleFonts {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    fn css_url(families: &[String], text: Option<&str>) -> String {
        let mut url = String::from("https://fonts.googleapis.com/css2?");
        for (i, fam) in families.iter().enumerate() {
            if i > 0 {
                url.push('&');
            }
            url.push_str("family=");
            url.push_str(&urlencode(fam));
        }
        match text {
            Some(t) => {
                url.push_str("&text=");
                url.push_str(&urlencode(t));
            }
            None => url.push_str("&display=swap"),
        }
        url
    }
}

impl Default for GoogleFonts {
    fn default() -> Self {
        Self::new()
    }
}

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
    (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

impl FontSource for GoogleFonts {
    async fn stylesheet(&self, families: &[String], text: Option<&str>) -> anyhow::Result<String> {
        let url = Self::css_url(families, text);
        let resp = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, UA)
            .send()
            .await
            .context("fetching Google Fonts stylesheet")?
            .error_for_status()?;
        Ok(resp.text().await?)
    }

    async fn woff2(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let resp = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, UA)
            .send()
            .await
            .with_context(|| format!("downloading {url}"))?
            .error_for_status()?;
        Ok(resp.bytes().await?.to_vec())
    }
}

/// Percent-encode the characters Google Fonts family/text params care about,
/// keeping the readable `Family:wght@400;700` syntax intact.
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'z' | b'0'..=b'9' | b':' | b'@' | b';' | b',' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The pull driver
// ---------------------------------------------------------------------------

/// Options for one `fonts pull` invocation.
#[derive(Debug, Clone)]
pub struct PullOptions {
    pub families: Vec<String>,
    pub text: Option<String>,
    pub out_dir: PathBuf,
    /// Origin-relative prefix the rewritten stylesheet points `url()`s at.
    pub url_prefix: String,
    /// Filename of the rewritten stylesheet written into `out_dir`.
    pub css_name: String,
}

/// What a pull produced.
#[derive(Debug, PartialEq, Eq)]
pub struct PullReport {
    pub faces: usize,
    pub css_path: PathBuf,
    pub woff2: Vec<PathBuf>,
}

/// Fetch, download, subset-preserving-write, and rewrite. Generic over the
/// source so tests drive it from a fixture.
pub async fn pull<S: FontSource>(src: &S, opts: &PullOptions) -> anyhow::Result<PullReport> {
    let css = src.stylesheet(&opts.families, opts.text.as_deref()).await?;
    let faces = parse_font_faces(&css);
    if faces.is_empty() {
        return Err(anyhow!("no @font-face rules in the fetched stylesheet"));
    }
    std::fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("creating {}", opts.out_dir.display()))?;

    let mut written = Vec::new();
    for face in &faces {
        let bytes = src.woff2(&face.src_url).await?;
        let path = opts.out_dir.join(face.local_name());
        std::fs::write(&path, &bytes).with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }

    let rewritten = rewrite_css(&css, &faces, &opts.url_prefix);
    let css_path = opts.out_dir.join(&opts.css_name);
    std::fs::write(&css_path, rewritten)
        .with_context(|| format!("writing {}", css_path.display()))?;

    Ok(PullReport {
        faces: faces.len(),
        css_path,
        woff2: written,
    })
}

/// A filesystem-backed [`FontSource`] over a recorded fixture directory: the
/// stylesheet is `<dir>/<css_file>` and each woff2 is looked up by URL basename.
/// Used by the unit test and available for offline/air-gapped pulls.
pub struct DirSource {
    pub dir: PathBuf,
    pub css_file: String,
}

impl FontSource for DirSource {
    async fn stylesheet(
        &self,
        _families: &[String],
        _text: Option<&str>,
    ) -> anyhow::Result<String> {
        let path = self.dir.join(&self.css_file);
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
    }

    async fn woff2(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let base = url.rsplit('/').next().unwrap_or(url);
        let path = self.dir.join(base);
        std::fs::read(&path).with_context(|| format!("reading {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts")
    }

    fn sample_css() -> String {
        std::fs::read_to_string(fixture_dir().join("inter.css")).unwrap()
    }

    #[test]
    fn parses_all_faces_with_subset_labels() {
        let faces = parse_font_faces(&sample_css());
        assert_eq!(faces.len(), 4);
        assert_eq!(faces[0].family, "Inter");
        assert_eq!(faces[0].subset.as_deref(), Some("cyrillic"));
        assert_eq!(faces[1].subset.as_deref(), Some("latin-ext"));
        assert_eq!(faces[2].subset.as_deref(), Some("latin"));
        assert!(
            faces[0]
                .unicode_range
                .as_deref()
                .unwrap()
                .contains("U+0400")
        );
        assert_eq!(faces[3].style, "italic");
        assert_eq!(faces[3].weight, "700");
        assert!(faces[2].src_url.ends_with("inter-latin-400-normal.woff2"));
    }

    #[test]
    fn local_names_are_deterministic_and_distinct() {
        let faces = parse_font_faces(&sample_css());
        let names: Vec<_> = faces.iter().map(|f| f.local_name()).collect();
        assert_eq!(names[2], "inter-latin-400-normal.woff2");
        assert_eq!(names[3], "inter-latin-700-italic.woff2");
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "no filename collisions");
    }

    #[test]
    fn rewrite_points_at_local_paths_and_drops_gstatic() {
        let faces = parse_font_faces(&sample_css());
        let out = rewrite_css(&sample_css(), &faces, "/fonts");
        assert!(
            !out.contains("fonts.gstatic.com"),
            "remote urls remain: {out}"
        );
        assert!(out.contains("url(/fonts/inter-latin-400-normal.woff2)"));
    }

    #[tokio::test]
    async fn pull_over_recorded_fixture_writes_everything() {
        let out_dir = std::env::temp_dir().join(format!("mw-fonts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out_dir);
        let src = DirSource {
            dir: fixture_dir(),
            css_file: "inter.css".into(),
        };
        let opts = PullOptions {
            families: vec!["Inter".into()],
            text: None,
            out_dir: out_dir.clone(),
            url_prefix: "/fonts".into(),
            css_name: "fonts.css".into(),
        };
        let report = pull(&src, &opts).await.unwrap();
        assert_eq!(report.faces, 4);
        assert_eq!(report.woff2.len(), 4);

        // Every woff2 landed with the fixture's bytes (subset preserved verbatim).
        let latin = out_dir.join("inter-latin-400-normal.woff2");
        assert!(latin.exists());
        assert!(std::fs::read(&latin).unwrap().starts_with(b"wOF2"));

        // The rewritten stylesheet is self-hostable.
        let css = std::fs::read_to_string(&report.css_path).unwrap();
        assert!(!css.contains("gstatic.com"));
        assert!(css.contains("url(/fonts/inter-latin-700-italic.woff2)"));
    }

    #[test]
    fn google_css_url_uses_family_and_display() {
        let url = GoogleFonts::css_url(&["Inter:wght@400;700".into()], None);
        assert!(url.contains("family=Inter:wght@400;700"), "{url}");
        assert!(url.contains("display=swap"));
    }
}
