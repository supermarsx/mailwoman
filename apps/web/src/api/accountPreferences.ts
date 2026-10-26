import { createSignal } from 'solid-js';
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';
import { observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent, type PreferenceAccount } from './accountPreferenceLifecycle.ts';

export const FOLDER_ICONS = ['folder', 'star', 'flag', 'tag', 'clock', 'calendar', 'file', 'image', 'lock', 'users', 'archive', 'mail', 'attachment', 'check', 'alert'] as const;
export type FolderIcon = typeof FOLDER_ICONS[number];
export interface AccountTag {name:string;color:string;icon:FolderIcon}
export interface AccountUiPreferences {
  analyticsConsent?: boolean;
  keyboardPreset?: 'default' | 'gmail' | 'outlook' | 'vim' | 'off';
  direction?: 'auto' | 'ltr' | 'rtl';
  markRead?: 'on-open' | 'never';
  markReadDelayMs?: number;
  offlineBudgetMb?: number;
  offlineRetentionDays?: number;
  eviction?: 'lru' | 'oldest' | 'manual';
  workspaceLayout?: 'tabs' | 'side-by-side' | 'stack';
  folderSidebarMode?: 'expanded' | 'compact' | 'hidden';
  folderSidebarWidth?: number;
  messageListWidth?: number;
  messageListHeight?: number;
  readingPane?: 'right' | 'bottom' | 'off';
  messageListCollapsed?: boolean;
  readerCollapsed?: boolean;
  workspaceSplitRatio?: number;
  folderPinned?: string[];
  folderOrder?: string[];
  folderIcons?: Record<string, FolderIcon>;
  tags?: Record<string, AccountTag>;
  folderLabels?: Record<string,string[]>;
}
interface Snapshot { preferences: AccountUiPreferences; etag: string }
export interface AccountPreferencesApi {
  load(): Promise<Snapshot>;
  save(preferences: AccountUiPreferences, etag: string): Promise<Snapshot>;
}
export class PreferenceConflict extends Error {}
const endpoint = '/api/account/preferences';
async function snapshot(response: Response): Promise<Snapshot> {
  if (response.status === 412) throw new PreferenceConflict();
  if (!response.ok) throw new Error('Account preferences could not be synchronized');
  const body = await response.json() as {version?: number; preferences?: AccountUiPreferences};
  const etag = response.headers.get('etag');
  if (body.version !== 1 || !body.preferences || typeof body.preferences !== 'object' || Array.isArray(body.preferences) || !etag) throw new Error('Invalid preferences response');
  return {preferences: body.preferences, etag};
}
const api: AccountPreferencesApi = {
  async load() { return snapshot(await apiFetch(withBase(endpoint))); },
  async save(preferences, etag) {
    return snapshot(await apiFetch(withBase(endpoint), {method:'PUT', headers:{'content-type':'application/json','if-match':etag}, body:JSON.stringify({version:1,preferences})}));
  },
};

/** Each response belongs to one authenticated generation. Local edits are
 * merged by field, so an acknowledgement cannot replace a newer local edit. */
export function createAccountPreferencesSync(transport: AccountPreferencesApi, publish: (value: AccountUiPreferences) => void, report: (status: string) => void, current: (account: PreferenceAccount) => boolean = preferenceAccountIsCurrent) {
  let account: PreferenceAccount | null = null;
  let base: AccountUiPreferences = {};
  let dirty: AccountUiPreferences = {};
  let etag: string | null = null;
  let serial = 0;
  let edits: Partial<Record<keyof AccountUiPreferences, number>> = {};
  let iconEdits: Record<string, {value: FolderIcon | null; serial: number}> = {};
  let tagEdits: Record<string, {value: AccountTag | null; serial: number}> = {};
  let folderLabelEdits: Record<string,{value:string[]|null;serial:number}>={};
  let busy = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const active = (value: PreferenceAccount): boolean => account === value && current(value);
  function rebaseIcons(): void {
    if (!Object.keys(iconEdits).length) { delete dirty.folderIcons; return; }
    const icons = {...base.folderIcons};
    for (const [id, edit] of Object.entries(iconEdits)) { if (edit.value === null) delete icons[id]; else icons[id] = edit.value; }
    dirty.folderIcons = icons;
  }
  function rebaseTags():void {
    if(!Object.keys(tagEdits).length){if(edits.tags===undefined)delete dirty.tags;return;}
    const tags={...base.tags};for(const [id,edit] of Object.entries(tagEdits)){if(edit.value===null)delete tags[id];else tags[id]=edit.value;}dirty.tags=tags;
  }
  function rebaseFolderLabels():void {
    if(!Object.keys(folderLabelEdits).length){delete dirty.folderLabels;return;}
    const labels={...base.folderLabels};for(const [id,edit] of Object.entries(folderLabelEdits)){if(edit.value===null)delete labels[id];else labels[id]=edit.value;}dirty.folderLabels=labels;
  }
  const emit = (): void => { rebaseIcons();rebaseTags();rebaseFolderLabels(); publish({...base, ...dirty}); };
  function schedule(): void {
    clearTimeout(timer);
    timer = setTimeout(() => { timer = undefined; void flush(); }, 400);
  }
  async function refresh(): Promise<void> {
    const owner = account;
    if (!owner || busy) return;
    busy = true;
    try {
      const loaded = await transport.load();
      if (!active(owner)) return;
      base = loaded.preferences; etag = loaded.etag; emit();
      report(Object.keys(dirty).length ? 'saving' : 'synced');
    } catch { if (active(owner)) report('error'); }
    finally { if (active(owner)) { busy = false; if (Object.keys(dirty).length && etag) schedule(); } }
  }
  async function flush(): Promise<void> {
    const owner = account;
    if (!owner || busy || !Object.keys(dirty).length) return;
    if (!etag) { await refresh(); return; }
    busy = true;
    let reschedule = false;
    const sentEdits = {...edits};
    const sentIcons = {...iconEdits};
    const sentTags = {...tagEdits};
    const sentFolderLabels={...folderLabelEdits};
    const sent = {...base, ...dirty};
    report('saving');
    try {
      const saved = await transport.save(sent, etag);
      if (!active(owner)) return;
      base = saved.preferences; etag = saved.etag;
      for (const key of Object.keys(sentEdits) as (keyof AccountUiPreferences)[]) {
        if (edits[key] === sentEdits[key]) { delete dirty[key]; delete edits[key]; }
      }
      for (const [id, edit] of Object.entries(sentIcons)) {
        if (iconEdits[id]?.serial === edit.serial) delete iconEdits[id];
      }
      for(const [id,edit] of Object.entries(sentTags)){if(tagEdits[id]?.serial===edit.serial)delete tagEdits[id];}
      for(const [id,edit] of Object.entries(sentFolderLabels)){if(folderLabelEdits[id]?.serial===edit.serial)delete folderLabelEdits[id];}
      emit(); report(Object.keys(dirty).length ? 'saving' : 'synced');
      reschedule = true;
    } catch (error) {
      if (!active(owner)) return;
      if (error instanceof PreferenceConflict) {
        etag = null;
        // Reload the other device's fields, retaining only our actual local edits.
        busy = false;
        await refresh();
      } else report('error');
    } finally {
      if (active(owner)) { busy = false; if (reschedule && Object.keys(dirty).length && etag) schedule(); }
    }
  }
  return {
    bind(next: PreferenceAccount | null): void {
      clearTimeout(timer); timer = undefined;
      account = next; base = {}; dirty = {}; edits = {}; iconEdits = {}; tagEdits = {}; folderLabelEdits={}; etag = null; busy = false;
      emit(); report(next ? 'loading' : 'idle');
      if (next) void refresh();
    },
    update(patch: AccountUiPreferences): void {
      if (!account || !active(account)) return;
      if (patch.folderIcons) {
        const previous = dirty.folderIcons ?? base.folderIcons ?? {};
        for (const id of new Set([...Object.keys(previous), ...Object.keys(patch.folderIcons)])) {
          if (previous[id] !== patch.folderIcons[id]) iconEdits[id] = {value: patch.folderIcons[id] ?? null, serial: ++serial};
        }
      }
      if(patch.tags){
        const previous=dirty.tags??base.tags??{};
        for(const id of new Set([...Object.keys(previous),...Object.keys(patch.tags)])){
          if(JSON.stringify(previous[id])!==JSON.stringify(patch.tags[id]))tagEdits[id]={value:patch.tags[id]??null,serial:++serial};
        }
        if(base.tags===undefined)edits.tags=++serial;
      }
      if(patch.folderLabels){const previous=dirty.folderLabels??base.folderLabels??{};for(const id of new Set([...Object.keys(previous),...Object.keys(patch.folderLabels)])){if(JSON.stringify(previous[id])!==JSON.stringify(patch.folderLabels[id]))folderLabelEdits[id]={value:patch.folderLabels[id]??null,serial:++serial};}}
      for (const key of Object.keys(patch) as (keyof AccountUiPreferences)[]) if (key !== 'folderIcons'&&key!=='tags'&&key!=='folderLabels') edits[key] = ++serial;
      dirty = {...dirty, ...patch}; report('saving'); emit(); schedule();
    },
    refresh, flush,
    dispose(): void { this.bind(null); },
  };
}

const [preferences, setPreferences] = createSignal<AccountUiPreferences>({});
const [status, setStatus] = createSignal('idle');
export const accountUiPreferences = preferences;
export const accountUiPreferenceStatus = status;
let controller: ReturnType<typeof createAccountPreferencesSync> | undefined;
let stop: (() => void) | undefined;
export function startAccountUiPreferences(): () => void {
  if (stop) return stop;
  const sync = createAccountPreferencesSync(api, setPreferences, setStatus);
  controller = sync;
  const unsubscribe = observePreferenceAccount(account => sync.bind(account));
  const interval = setInterval(() => { if (preferenceAccount() && document.visibilityState === 'visible') void sync.refresh(); }, 30_000);
  stop = () => { unsubscribe(); clearInterval(interval); sync.dispose(); controller = undefined; stop = undefined; };
  return stop;
}
export function setAccountUiPreferences(patch: AccountUiPreferences): void { controller?.update(patch); }
export async function retryAccountUiPreferences():Promise<void>{await controller?.flush();}
