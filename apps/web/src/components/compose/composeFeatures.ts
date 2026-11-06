// The shared composer feature-flag contract. Every composer control that an
// administrator or user may switch off has one entry in COMPOSE_FEATURES and is
// wrapped with `useComposeFeature(id)`. The value lives in the settings model as
// `compose.feature.<id>` (Bool, default true, user + admin scope with lock and
// default), so the Settings and Admin cards render straight from this list and a
// new entry appears in both automatically.
//
// Agents append entries in their group's section only (format / insert / layout /
// options / delivery). Do not reorder the sections.

import { loadSettings, settings } from '../../api/settings.ts';

export type ComposeFeatureGroup = 'format' | 'insert' | 'layout' | 'options' | 'delivery';

export interface ComposeFeature {
  /** `[a-z][A-Za-z0-9.]*`, at most 52 characters (the registry family rule). */
  id: string;
  group: ComposeFeatureGroup;
  /** Fluent id (catalog `compose-features`). */
  labelKey: string;
  descKey?: string;
  /** What the feature is when neither user nor admin set anything. */
  defaultOn: boolean;
  /**
   * Rich or risky: switched off by the Simple and Locked-down presets. Basic
   * formatting is not advanced.
   */
  advanced?: boolean;
  /** Only an administrator may change it (no user toggle in Settings). */
  adminOnly?: boolean;
}

export const COMPOSE_FEATURE_GROUPS: readonly ComposeFeatureGroup[] = ['format', 'insert', 'layout', 'options', 'delivery'];

export const COMPOSE_FEATURES: readonly ComposeFeature[] = [
  // ── format (agent FORMAT appends here) ───────────────────────────────────────

  // ── insert (agent INSERT appends here) ───────────────────────────────────────
  { id: 'insert.attachFile', group: 'insert', labelKey: 'cf-insert-attachFile', descKey: 'cf-insert-attachFile-desc', defaultOn: true },
  { id: 'insert.attachFromMessages', group: 'insert', labelKey: 'cf-insert-attachFromMessages', descKey: 'cf-insert-attachFromMessages-desc', defaultOn: true },
  { id: 'insert.pictureFile', group: 'insert', labelKey: 'cf-insert-pictureFile', descKey: 'cf-insert-pictureFile-desc', defaultOn: true },
  { id: 'insert.pictureFromMessages', group: 'insert', labelKey: 'cf-insert-pictureFromMessages', descKey: 'cf-insert-pictureFromMessages-desc', defaultOn: true, advanced: true },
  { id: 'insert.shapes', group: 'insert', labelKey: 'cf-insert-shapes', descKey: 'cf-insert-shapes-desc', defaultOn: true, advanced: true },
  { id: 'insert.screenshot', group: 'insert', labelKey: 'cf-insert-screenshot', descKey: 'cf-insert-screenshot-desc', defaultOn: true, advanced: true },
  { id: 'insert.businessCard', group: 'insert', labelKey: 'cf-insert-businessCard', descKey: 'cf-insert-businessCard-desc', defaultOn: true, advanced: true },
  { id: 'insert.calendar', group: 'insert', labelKey: 'cf-insert-calendar', descKey: 'cf-insert-calendar-desc', defaultOn: true, advanced: true },
  { id: 'insert.event', group: 'insert', labelKey: 'cf-insert-event', descKey: 'cf-insert-event-desc', defaultOn: true, advanced: true },
  { id: 'insert.symbols', group: 'insert', labelKey: 'cf-insert-symbols', descKey: 'cf-insert-symbols-desc', defaultOn: true },
  { id: 'insert.horizontalLine', group: 'insert', labelKey: 'cf-insert-horizontalLine', descKey: 'cf-insert-horizontalLine-desc', defaultOn: true },
  { id: 'insert.dateStamp', group: 'insert', labelKey: 'cf-insert-dateStamp', descKey: 'cf-insert-dateStamp-desc', defaultOn: true },
  { id: 'insert.link', group: 'insert', labelKey: 'cf-insert-link', descKey: 'cf-insert-link-desc', defaultOn: true },
  { id: 'insert.dragDropAttach', group: 'insert', labelKey: 'cf-insert-dragDropAttach', descKey: 'cf-insert-dragDropAttach-desc', defaultOn: true },
  { id: 'insert.dragDropInline', group: 'insert', labelKey: 'cf-insert-dragDropInline', descKey: 'cf-insert-dragDropInline-desc', defaultOn: true, advanced: true },
  { id: 'insert.dropMessages', group: 'insert', labelKey: 'cf-insert-dropMessages', descKey: 'cf-insert-dropMessages-desc', defaultOn: true, advanced: true },

  // ── layout ───────────────────────────────────────────────────────────────────
  { id: 'layout.columns', group: 'layout', labelKey: 'cf-layout-columns', descKey: 'cf-layout-columns-desc', defaultOn: true, advanced: true },
  { id: 'layout.contentWidth', group: 'layout', labelKey: 'cf-layout-contentWidth', descKey: 'cf-layout-contentWidth-desc', defaultOn: true, advanced: true },
  { id: 'layout.pageColor', group: 'layout', labelKey: 'cf-layout-pageColor', descKey: 'cf-layout-pageColor-desc', defaultOn: true, advanced: true },
  { id: 'layout.themes', group: 'layout', labelKey: 'cf-layout-themes', descKey: 'cf-layout-themes-desc', defaultOn: true, advanced: true },
  { id: 'layout.defaultFont', group: 'layout', labelKey: 'cf-layout-defaultFont', descKey: 'cf-layout-defaultFont-desc', defaultOn: true },
  { id: 'layout.spacing', group: 'layout', labelKey: 'cf-layout-spacing', descKey: 'cf-layout-spacing-desc', defaultOn: true, advanced: true },

  // ── options ──────────────────────────────────────────────────────────────────
  { id: 'options.sentFolder', group: 'options', labelKey: 'cf-options-sentFolder', descKey: 'cf-options-sentFolder-desc', defaultOn: true, advanced: true },
  { id: 'options.replyTo', group: 'options', labelKey: 'cf-options-replyTo', descKey: 'cf-options-replyTo-desc', defaultOn: true, advanced: true },

  // ── delivery ─────────────────────────────────────────────────────────────────
];

/** The settings-model key of a feature. */
export function composeFeatureKey(id: string): string {
  return `compose.feature.${id}`;
}

/** Whether a feature id is shaped like the registry family allows. */
export function isComposeFeatureId(id: string): boolean {
  return /^[a-z][A-Za-z0-9.]{0,51}$/.test(id);
}

export function composeFeature(id: string): ComposeFeature | undefined {
  return COMPOSE_FEATURES.find((f) => f.id === id);
}

/** Whether `id` is on, given a settings map (the default when nothing is stored). */
export function composeFeatureOn(id: string, map: Record<string, { value: unknown } | undefined> = settings()): boolean {
  const entry = map[composeFeatureKey(id)];
  if (entry !== undefined && typeof entry.value === 'boolean') return entry.value;
  return composeFeature(id)?.defaultOn ?? true;
}

let requested = false;

/**
 * An accessor for one feature. Falls back to the feature's `defaultOn` while the
 * settings load, or when the server does not know the key (older server).
 */
export function useComposeFeature(id: string): () => boolean {
  if (!requested) {
    requested = true;
    void loadSettings().catch(() => undefined);
  }
  return () => composeFeatureOn(id);
}

export type ComposePresetId = 'everything' | 'simple' | 'lockedDown';

export const COMPOSE_PRESETS: readonly { id: ComposePresetId; labelKey: string; descKey: string }[] = [
  { id: 'everything', labelKey: 'cf-preset-everything', descKey: 'cf-preset-everything-desc' },
  { id: 'simple', labelKey: 'cf-preset-simple', descKey: 'cf-preset-simple-desc' },
  { id: 'lockedDown', labelKey: 'cf-preset-lockedDown', descKey: 'cf-preset-lockedDown-desc' },
];

/** The value a preset gives one feature. */
export function presetValue(preset: ComposePresetId, f: ComposeFeature): boolean {
  switch (preset) {
    case 'everything':
      return true;
    case 'simple':
      return !f.advanced;
    case 'lockedDown':
      return f.group === 'format' ? !f.advanced : f.group === 'delivery' && !f.advanced;
  }
}

/** Every feature a preset sets, as `[id, value]`; admin-only gates are left alone for users. */
export function presetValues(preset: ComposePresetId, opts: { admin: boolean }): [string, boolean][] {
  return COMPOSE_FEATURES.filter((f) => opts.admin || !f.adminOnly).map((f) => [f.id, presetValue(preset, f)]);
}
