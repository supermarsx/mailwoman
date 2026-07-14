// TypeScript UI-plugin tier styles (t10 plan §3 e10). Token-native — reuses the frozen
// theme + a11y contract (`vars.*`); no new design tokens, no raw colours, so contrast +
// focus behaviour follow the deployment theme and the axe WCAG 2.2 AA gate.

import { style } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

export const tier = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[3],
  fontFamily: vars.font.ui,
  color: vars.color.text,
});

// The persistent, non-dismissable-by-plugin trust banner for approved-but-UNSIGNED
// plugins. Rendered by the HOST, outside every sandboxed iframe, so a plugin can neither
// hide nor style it. Uses `vars.color.warning` (not colour alone — carries an icon + text
// label) and a `1px` border so it reads in high-contrast themes.
export const banner = style({
  display: 'flex',
  alignItems: 'flex-start',
  gap: vars.space[3],
  padding: `${vars.space[3]} ${vars.space[4]}`,
  borderRadius: vars.radius.md,
  border: `1px solid ${vars.color.warning}`,
  background: vars.color.bgAlt,
  color: vars.color.text,
});

export const bannerIcon = style({
  fontSize: '1.1rem',
  lineHeight: 1.2,
  flexShrink: 0,
  // Decorative glyph; the accessible text lives in the adjacent copy.
});

export const bannerBody = style({ display: 'flex', flexDirection: 'column', gap: vars.space[1] });
export const bannerTitle = style({ fontSize: '0.9rem', fontWeight: 600, margin: 0 });
export const bannerText = style({ fontSize: '0.82rem', color: vars.color.textDim, margin: 0 });
export const bannerList = style({ margin: 0, padding: 0, listStyle: 'none', display: 'flex', gap: vars.space[2], flexWrap: 'wrap' });
export const bannerPluginId = style({ fontFamily: vars.font.mono, fontSize: '0.78rem' });

// One mounted plugin slot: a labelled region wrapping its sandboxed frame.
export const slot = style({
  display: 'flex',
  flexDirection: 'column',
  gap: vars.space[2],
  border: `1px solid ${vars.color.border}`,
  borderRadius: vars.radius.md,
  padding: vars.space[2],
});

export const slotLabel = style({
  fontSize: '0.8rem',
  fontWeight: 600,
  color: vars.color.textDim,
});

// The sandboxed guest frame. Sizing only — the security posture is the `sandbox` attr.
export const frame = style({
  width: '100%',
  minHeight: '3rem',
  border: 'none',
  borderRadius: vars.radius.sm,
  background: vars.color.surface,
});
