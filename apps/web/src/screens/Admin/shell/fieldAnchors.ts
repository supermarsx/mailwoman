// Field anchors: the console's index of every UI field (t32-cfg UX10, DC12).
//
// A field anchor names one field the admin can edit (or read) on one screen:
// its id, the section and sub-route it lives on, its label and the capability
// that shows it. The command palette searches the index, and choosing a hit
// opens `#/<section>[/<rest>]?field=<id>`; the shell then focuses and highlights
// that field (`focusFieldWhenReady`).
//
// Each screen lane adds its anchors in its own file next to its screen,
// `<Screen>.anchors.ts`, whose default export is `defineFieldAnchors({...})`.
// The files are collected here at build time, so no lane edits a shared list.
// In the screen, the element that holds the field carries
// `{...fieldAnchorAttrs(id)}` (SettingsFormPage does this for every field).
//
// Field ids are the registry key or catalogue knob id where there is one
// (`signin.newUsers`, `upstream.proxyTimeoutSecs`), so a catalogue entry and its
// field share one name (e71b checks every `Ui` catalogue entry has an anchor).

import type { Capability } from './can.ts';

export interface FieldAnchor {
  /** Unique across the console. */
  id: string;
  /** Section id (`config`, `users`, ...). */
  section: string;
  /** Segments after the section, for fields on a sub-page. */
  rest?: readonly string[];
  /** Fluent id of the field label. */
  labelId: string;
  /** The catalog module holding `labelId` (loaded before the label is shown). */
  catalog?: string;
  /** Extra search terms that are names, not prose: registry key, env variable. */
  keywords?: readonly string[];
  /** Capability needed to see the field, when it is not the section's own. */
  capability?: Capability;
}

export interface FieldAnchorSet {
  section: string;
  catalog?: string;
  anchors: readonly Omit<FieldAnchor, 'section' | 'catalog'>[];
}

/** Typed identity for a `<Screen>.anchors.ts` default export. */
export function defineFieldAnchors(set: FieldAnchorSet): FieldAnchorSet {
  return set;
}

function flatten(set: FieldAnchorSet): FieldAnchor[] {
  return set.anchors.map((a) => ({
    ...a,
    section: set.section,
    ...(set.catalog !== undefined ? { catalog: set.catalog } : {}),
  }));
}

const collected: FieldAnchor[] = Object.values(
  import.meta.glob<FieldAnchorSet>('../**/*.anchors.ts', { eager: true, import: 'default' }),
).flatMap(flatten);

const extra: FieldAnchor[] = [];

/** Add anchors at run time (tests, and screens built from server data). Returns a remover. */
export function registerFieldAnchors(set: FieldAnchorSet): () => void {
  const added = flatten(set);
  extra.push(...added);
  return () => {
    for (const a of added) {
      const i = extra.indexOf(a);
      if (i !== -1) extra.splice(i, 1);
    }
  };
}

/** Every known field anchor. */
export function fieldAnchors(): readonly FieldAnchor[] {
  return [...collected, ...extra];
}

export function findFieldAnchor(id: string): FieldAnchor | undefined {
  return fieldAnchors().find((a) => a.id === id);
}

/** The attribute that marks a field's holder element. */
export const FIELD_ANCHOR_ATTR = 'data-field-anchor';
/** Present while a field is highlighted after a deep link. */
export const FIELD_HIGHLIGHT_ATTR = 'data-field-highlight';
/** The fragment query parameter that names a field: `#/config?field=<id>`. */
export const FIELD_PARAM = 'field';

/** Spread on the element holding a field: `<div {...fieldAnchorAttrs('smtp.port')}>`. */
export function fieldAnchorAttrs(id: string): { 'data-field-anchor': string } {
  return { 'data-field-anchor': id };
}

function holderOf(id: string, root: ParentNode): HTMLElement | null {
  for (const el of root.querySelectorAll<HTMLElement>(`[${FIELD_ANCHOR_ATTR}]`)) {
    if (el.getAttribute(FIELD_ANCHOR_ATTR) === id) return el;
  }
  return null;
}

const CONTROL = 'input:not([type="hidden"]), select, textarea, button, [tabindex]:not([tabindex="-1"])';

function controlOf(holder: HTMLElement): HTMLElement {
  const marked = holder.querySelector<HTMLElement>('[data-field-control]');
  if (marked && !(marked as HTMLInputElement).disabled) return marked;
  for (const el of holder.querySelectorAll<HTMLElement>(CONTROL)) {
    if (!(el as HTMLInputElement).disabled) return el;
  }
  // A pinned (disabled) field cannot take focus; the holder takes it so the
  // reason next to the field is read.
  if (!holder.hasAttribute('tabindex')) holder.setAttribute('tabindex', '-1');
  return holder;
}

const highlightTimers = new WeakMap<HTMLElement, ReturnType<typeof setTimeout>>();

/** Focus, scroll to and briefly highlight field `id`. False when it is not on screen. */
export function focusField(id: string, root: ParentNode = document): boolean {
  const holder = holderOf(id, root);
  if (!holder) return false;
  const control = controlOf(holder);
  control.focus({ preventScroll: true });
  holder.scrollIntoView?.({ block: 'center', behavior: 'auto' });
  holder.setAttribute(FIELD_HIGHLIGHT_ATTR, '');
  const prev = highlightTimers.get(holder);
  if (prev !== undefined) clearTimeout(prev);
  highlightTimers.set(
    holder,
    setTimeout(() => holder.removeAttribute(FIELD_HIGHLIGHT_ATTR), 2400),
  );
  return true;
}

/** Focus field `id` once it is rendered (sections load lazily). Resolves false after `timeoutMs`. */
export function focusFieldWhenReady(
  id: string,
  opts: { timeoutMs?: number; root?: ParentNode & Node; signal?: AbortSignal } = {},
): Promise<boolean> {
  const root = opts.root ?? document;
  if (focusField(id, root)) return Promise.resolve(true);
  return new Promise<boolean>((resolve) => {
    let done = false;
    const finish = (ok: boolean): void => {
      if (done) return;
      done = true;
      observer.disconnect();
      clearTimeout(timer);
      resolve(ok);
    };
    const observer = new MutationObserver(() => {
      if (focusField(id, root)) finish(true);
    });
    observer.observe(root === document ? document.body : root, { childList: true, subtree: true });
    const timer = setTimeout(() => finish(false), opts.timeoutMs ?? 5000);
    opts.signal?.addEventListener('abort', () => finish(false), { once: true });
  });
}
