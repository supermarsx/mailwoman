# Global branding

Administrators with unscoped `branding.read` can open **Branding**. Changes require unscoped `branding.manage`. Domain-scoped grants do not authorize global changes.

The product name and sign-in notice use the existing `appearance.brandName` and `login.notice` registry values. The editor preserves their policy locks. Saves require the read ETag and compare the prior configuration inside a database transaction, serialized with generic writes to those two registry keys.

Logo, browser icon and sign-in background uploads are stored in the database. They accept PNG or JPEG up to 1 MB, with maximum dimensions of 512, 1024 and 2560 pixels respectively. Oversized images must be resized before uploading. SVG and remote image URLs are refused. Image headers are checked before the existing bounded WASM image decoder re-encodes pixels to PNG, removing source metadata. At most two uploads decode concurrently.

An upload does not publish the image. Only an image referenced by saved global branding is served anonymously at its same-origin, content-addressed asset URL. Clearing its reference restores the bundled presentation and stops new origin requests to that URL; previously public immutable browser caches can retain the old image. Branding is public information, so uploads must not contain secrets.

The browser applies the name and logo to the mailbox and admin shell, and both sign-in screens. The background is decorative. Browser favicon and Apple-touch links follow the saved icon and reset to their bundled paths when removed. Document titles update when branding loads. Native shell identities and installed-app/PWA manifests remain build-time/default assets in this checkpoint; this is not a generated platform-icon pipeline.

This section currently edits global branding. Domain overrides, custom CSS, theme design, branding export/import, image resizing and generated ICO/PWA icon variants are not exposed by this editor.
