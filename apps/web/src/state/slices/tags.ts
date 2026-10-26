// Tags/labels registry slice (plan §3 e7, §1.5). Labels themselves are JMAP
// keywords (round-tripped to IMAP keywords by the engine); this slice owns the
// per-user *color/icon registry* — the metadata IMAP can't hold — as
// `Tag {id,name,color,icon}`. The engine persists it in mw-store settings at
// account UI preferences. Signed-in registries synchronize by account and tag
// ID; the legacy local cache is only used before authentication.
//
// Applying/removing a tag on a message is a keyword mutation and lives in the
// mail slice (it mutates the message list); this slice only answers "what does
// keyword `work` look like?".

import { createSignal, createMemo, onCleanup, type Accessor } from 'solid-js';
import {accountUiPreferences,setAccountUiPreferences,FOLDER_ICONS,type FolderIcon} from '../../api/accountPreferences.ts';
import {observePreferenceAccount,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import type { SliceContext } from './context.ts';

/** A user label: `id` doubles as the JMAP keyword it maps to (lowercased). */
export interface Tag {
  id: string;
  name: string;
  /** Any CSS color; rendered as the chip background. */
  color: string;
  /** A native theme-colored icon name. */
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
  { id: 'work', name: 'Work', color: '#2563eb', icon: 'archive' },
  { id: 'personal', name: 'Personal', color: '#15803d', icon: 'users' },
  { id: 'important', name: 'Important', color: '#b91c1c', icon: 'star' },
  { id: 'later', name: 'Later', color: '#a16207', icon: 'clock' },
  { id: 'receipts', name: 'Receipts', color: '#7c3aed', icon: 'file' },
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
    .normalize('NFKD').replace(/\p{M}/gu,'')
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
  const [localTags, setTags] = createSignal<Tag[]>(load());
  const [owner,setOwner]=createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setOwner));
  const tags=createMemo(()=>owner()?Object.entries(accountUiPreferences().tags??Object.fromEntries(DEFAULT_TAGS.map(({id,...tag})=>[id,tag]))).map(([id,tag])=>({id,...tag})):localTags());

  function persist(next: Tag[]): void {
    if(owner()){
      setAccountUiPreferences({tags:Object.fromEntries(next.slice(0,100).map(({id,name,color,icon})=>[id,{name:name.slice(0,128),color:/^#[0-9a-f]{6}$/i.test(color)?color:'#2563eb',icon:(FOLDER_ICONS as readonly string[]).includes(icon)?icon as FolderIcon:'tag'}]))});
      return;
    }
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
    addTag(name, color, icon = 'tag') {
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
