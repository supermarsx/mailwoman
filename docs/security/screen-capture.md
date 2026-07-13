# Screen-capture protection: an honest statement

**A web browser cannot prevent, block, or detect screenshots or screen
recordings.** There is no web API for it. Any product that claims its *web app*
stops screen capture is wrong or lying. Mailwoman will not make that claim.

What V4 ships is a **visible watermark overlay** — and only that. It is a
deterrent, not a control.

## What the watermark is

When enabled (`MW_WATERMARK=true`), the SPA tiles a low-opacity overlay across
sensitive views stamping the **viewer's identity and a server timestamp**. Its
purpose is to:

- discourage casual sharing, and
- make a leaked screenshot **attributable** to whoever took it.

It is pure DOM/CSS rendered under the existing `script-src 'self'` policy and
pulls no external resource (CSP-safe). Opacity is tunable via
`MW_WATERMARK_OPACITY` (0.0–1.0, default 0.08 — low enough not to impede reading).

Every watermark config response the server returns carries a mandatory honesty
note, so the feature can never ship without the statement. That canonical wording,
mirrored here verbatim from `mw-server`'s `watermark::HONEST_NOTE`:

> This watermark is a visual deterrent only. A web browser cannot prevent, block,
> or detect screenshots or screen recordings, so this overlay cannot stop this
> content from being captured. It stamps the viewer's identity and the time across
> the view to discourage casual sharing and to make a leaked screenshot
> attributable — it is not a security control. Genuine screen-capture protection
> requires the native desktop application, planned for a later release.

## What the watermark is not

It does **not** prevent, block, detect, or even notice a screenshot, a screen
recording, a photo of the monitor, or the OS print-screen key. A determined viewer
captures the content regardless; the watermark only makes the capture identifiable.

## Real capture protection

Genuine screen-capture suppression requires OS-level cooperation from a **native
desktop application** — `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` on
Windows, `FLAG_SECURE` on Android, and the platform equivalents. That native shell
is a later milestone (**V5**) and is deliberately out of scope for this web release.
Until then, treat any sensitive content shown in the browser as capturable.
