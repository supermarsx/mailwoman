# Global branding and installed apps

Admin → Branding publishes the product name, sign-in notice, logo, browser icon, sign-in background, and three installed-app icon variants. Uploads remain private until the complete branding form is saved. Writes use a revision check; stale editors must reload instead of overwriting a newer configuration.

Installed-app icons must be exactly 192×192 or 512×512 as labelled. The separate 512×512 maskable icon should keep important artwork inside a centred circle whose diameter is 80% of the image. PNG and JPEG inputs are re-encoded through the media sandbox; external image URLs and SVG uploads are not accepted.

The public `/manifest.webmanifest` reflects the published name and icon variants, with bundled icons as individual defaults. Its start URL, scope and icon URLs are relative to the application mount. It carries no account identifier and is revalidated rather than cached indefinitely. Existing installed applications refresh their identity according to their browser/platform lifecycle; saving does not force an immediate OS icon update.

Manifest behavior follows the [Web Application Manifest specification](https://www.w3.org/TR/appmanifest/). This section covers global branding; domain branding and arbitrary custom CSS are separate capabilities.
