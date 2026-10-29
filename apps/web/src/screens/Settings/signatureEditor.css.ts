import { globalStyle, style } from '@vanilla-extract/css';
import { vars } from '../../theme/contract.css.ts';

// The signature editor's body (t49): the composer's editor, given a height of
// its own so it fills the card the way the composer body fills its dialog
// (t37): the editor surface takes what the ribbon and footer leave and scrolls
// inside, rather than growing the page with every line.
export const body = style({
  display: 'flex',
  flexDirection: 'column',
  minInlineSize: 0,
  blockSize: 'clamp(14rem, calc(100dvh - 26rem), 36rem)',
  border: `${vars.border.rule} solid ${vars.color.border}`,
  borderRadius: vars.radius.sm,
  background: vars.color.bg,
  overflow: 'hidden',
});

globalStyle(`${body} > :is(.compose-rt, .compose-source)`, { flex: '1 1 0', minBlockSize: 0, display: 'flex', flexDirection: 'column' });
globalStyle(`${body} .compose-rt__surface`, { flex: '1 1 0', minBlockSize: 0, maxBlockSize: 'none', overflow: 'auto' });
globalStyle(`${body} .ProseMirror`, { minBlockSize: '100%', boxSizing: 'border-box' });
globalStyle(`${body} .compose-source > textarea`, { flex: '1 1 0', minBlockSize: 0, resize: 'none' });
globalStyle(`${body} > textarea`, { flex: '1 1 0', minBlockSize: 0, resize: 'none', border: 0, borderRadius: 0 });

export const pending = style({ margin: vars.space[3] });
