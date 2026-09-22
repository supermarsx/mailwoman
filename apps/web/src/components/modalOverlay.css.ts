// The dimmed, centering backdrop behind an `aria-modal` dialog.
//
// One shared class, for two reasons. It was duplicated byte-for-byte as an inline
// `style={{…}}` object in `screens/Admin/Plugins/Allowlist.tsx` and
// `screens/Admin/RethreadMaintenance.tsx`; and because every value in that object
// except the `padding` was a compile-time constant, Solid's compiler hoisted those
// six declarations into the compiled template as a literal `style="…"` attribute.
// A literal style attribute is subject to `style-src`, and the shell ships
// `style-src 'self'` with no `'unsafe-inline'` (dropped deliberately in 26.16), so
// the browser refused to apply it — the overlay rendered unstyled. Found by
// t24-e12 in the e2e-crypto Playwright trace:
//
//   Applying inline style violates the following Content Security Policy
//   directive 'style-src 'self''.
//
// A class is served from the bundled `'self'` stylesheet, so it applies under the
// CSP unchanged. Solid applies only DYNAMIC `style={{…}}` values through the CSSOM
// (`el.style.setProperty`), which is NOT subject to `style-src` — static ones become
// an attribute. That distinction is the whole bug: prefer a class for anything
// constant.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const modalOverlay = style({
  position: 'fixed',
  inset: 0,
  display: 'grid',
  placeItems: 'center',
  background: 'rgba(0, 0, 0, 0.5)',
  padding: vars.space[4],
  zIndex: 1000,
});
