// Tags/labels registry slice (plan §3 e7, §1.5). Labels themselves are JMAP
// keywords (round-tripped to IMAP keywords by the engine); this slice owns the
// per-user *color/icon registry* — the metadata IMAP can't hold — as
// `Tag {id,name,color,icon}`. The engine persists it in mw-store settings at
// integration (§1.5); until then it lives here, seeded with defaults and cached
// in localStorage so a page reload keeps the user's palette.
//
// Applying/removing a tag on a message is a keyword mutation and lives in the
// mail slice (it mutates the message list); this slice only answers "what does
// keyword `work` look like?".

import { createSignal, type Accessor } from 'solid-js';
import type { SliceContext } from './context.ts';

/** A user label: `id` doubles as the JMAP keyword it maps to (lowercased). */
export interface Tag {
  id: string;
  name: string;
  /** Any CSS color; rendered as the chip background. */
  color: string;
  /** A short glyph (emoji) shown before the name. */
  icon: string;
}

/** Keyword tokens Mailwoman reserves as system flags — never shown as labels. */
export const SYSTEM_KEYWORDS = new Set([
  '$seen',
  '$flagged',
  '$answered',
  '$draft',
  '$forwarded',
  '$junk',
  '$notjunk',
]);

/** Is `keyword` a user label (vs. a system `$`-flag)? */
export function isLabelKeyword(keyword: string): boolean {
  return !keyword.startsWith('$') && !SYSTEM_KEYWORDS.has(keyword);
}

const DEFAULT_TAGS: Tag[] = [
  { id: 'work', name: 'Work', color: '#2563eb', icon: '💼' },
  { id: 'personal', name: 'Personal', color: '#15803d', icon: '🏠' },
  { id: 'important', name: 'Important', color: '#b91c1c', icon: '⭐' },
  { id: 'later', name: 'Later', color: '#a16207', icon: '🕒' },
  { id: 'receipts', name: 'Receipts', color: '#7c3aed', icon: '🧾' },
];

const STORAGE_KEY = 'mw.tags.v1';

export interface TagsSlice {
  tags: Accessor<Tag[]>;
  /** The registry entry for a keyword, or `undefined` for an unregistered one. */
  tagByKeyword(keyword: string): Tag | undefined;
  /** Create/register a label; returns the normalized keyword id. */
  addTag(name: string, color: string, icon?: string): string;
  /** Delete a label from the registry (distinct from the mail slice's
   *  `removeTag`, which removes a keyword from a message). */
  deleteTag(id: string): void;
  updateTag(id: string, patch: Partial<Omit<Tag, 'id'>>): void;
}

/** Normalize a label name into a keyword token (lowercase, no spaces/`$`). */
export function keywordFor(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/^\$+/, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function load(): Tag[] {
  try {
    const raw = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (raw === null || raw === undefined) return DEFAULT_TAGS;
    const parsed = JSON.parse(raw) as Tag[];
    if (Array.isArray(parsed) && parsed.length > 0) return parsed;
  } catch {
    // Corrupt/absent storage falls back to defaults.
  }
  return DEFAULT_TAGS;
}

export function createTagsSlice(_ctx: SliceContext): TagsSlice {
  const [tags, setTags] = createSignal<Tag[]>(load());

  function persist(next: Tag[]): void {
    setTags(next);
    try {
      globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(next));
    } catch {
      // Non-fatal: the registry still works in-memory this session.
    }
  }

  return {
    tags,
    tagByKeyword: (keyword) => tags().find((t) => t.id === keyword),
    addTag(name, color, icon = '🏷️') {
      const id = keywordFor(name);
      if (id.length === 0) return id;
      const existing = tags().find((t) => t.id === id);
      if (existing !== undefined) {
        persist(tags().map((t) => (t.id === id ? { ...t, name, color, icon } : t)));
      } else {
        persist([...tags(), { id, name, color, icon }]);
      }
      return id;
    },
    deleteTag(id) {
      persist(tags().filter((t) => t.id !== id));
    },
    updateTag(id, patch) {
      persist(tags().map((t) => (t.id === id ? { ...t, ...patch } : t)));
    },
  };
}
