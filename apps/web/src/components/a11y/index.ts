// Shared accessibility primitives (plan §3 e3 — the canonical WCAG 2.2 focus
// helpers). Import from here:
//
//   import { createFocusTrap, createRovingTabindex } from '../components/a11y';
//
// These are the reusable building blocks for accessible dialogs and menus:
//   • createFocusTrap        — modal focus trap + focus restore + Esc-to-close
//   • createRovingTabindex   — WAI-ARIA roving-tabindex for menus/toolbars/nav
//   • focusableWithin / firstFocusable / isFocusable — the shared focus queries
//
// This batch each web area implements focus locally to avoid a build-dependency
// race; this module is the single clean version a later consolidation pass (and
// the engine executor e8) migrates everything onto.

export {
  createFocusTrap,
  type FocusTrapOptions,
} from './focusTrap.ts';

export {
  createRovingTabindex,
  type RovingTabindexOptions,
  type RovingOrientation,
} from './rovingTabindex.ts';

export {
  focusableWithin,
  firstFocusable,
  isFocusable,
  isVisible,
} from './focusable.ts';
