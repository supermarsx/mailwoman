// Settings > Retention: rules that move or delete the signed-in person's own mail
// after a number of days, by folder, label, sender, size, read state or
// attachment. The administrator's limits (days floor and ceiling, whether mail
// may be deleted for good, protected folders, organisation-wide rules) are shown
// beside the form and enforced by the server. A preview counts what a rule would
// change before it is saved. Mail goes to Trash first and is purged after an undo
// window unless the administrator allows permanent deletion.
import { createEffect, createResource, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import type { Mailbox } from '../../api/jmap-types.ts';
import * as api from '../../api/retention.ts';
import {
  ACTION_KINDS, RetentionError, blankRule,
  type ActionKind, type Preview, type ReadState, type Rule, type RuleInput,
} from '../../api/retention.ts';
import { retentionFolder, setRetentionFolder } from '../../state/retentionFocus.ts';
import { actionMessage, daysFromNow, isDestructive, joinList, needsDays, ruleProblem, splitList } from './retentionModel.ts';
import * as css from './styles.css.ts';

export interface RetentionApi {
  load: typeof api.loadRetention;
  create: typeof api.createRule;
  update: typeof api.updateRule;
  remove: typeof api.deleteRule;
  adopt: typeof api.adoptTemplate;
  settings: typeof api.saveSettings;
  preview: typeof api.previewRules;
  run: typeof api.runRules;
}

const live: RetentionApi = {
  load: api.loadRetention, create: api.createRule, update: api.updateRule, remove: api.deleteRule,
  adopt: api.adoptTemplate, settings: api.saveSettings, preview: api.previewRules, run: api.runRules,
};

const when = (seconds: number): string => (seconds > 0 ? formatDateTime(seconds * 1000) : t('retention-never'));
const READ_STATES: readonly ReadState[] = ['any', 'read', 'unread'];

/** Folder choices: ids by name, plus the role shortcuts the server resolves per account. */
function folderName(folders: readonly Mailbox[], spec: string): string {
  if (spec.startsWith('role:')) return t(`retention-role-${spec.slice(5)}`);
  return folders.find((f) => f.id === spec)?.name ?? spec;
}

const clone = <T,>(value: T): T => JSON.parse(JSON.stringify(value)) as T;

function toInput(r: Rule): RuleInput {
  return { name: r.name, enabled: r.enabled, scope: r.scope, action: r.action, exclude: r.exclude };
}

function Editor(props: {
  initial: RuleInput;
  folders: readonly Mailbox[];
  policy: api.RetentionPolicy;
  api: RetentionApi;
  onSave: (rule: RuleInput) => Promise<void>;
  onCancel: () => void;
}): JSX.Element {
  const [rule, setRule] = createSignal<RuleInput>(clone(props.initial));
  const [labels, setLabels] = createSignal(joinList(props.initial.scope.labels));
  const [senders, setSenders] = createSignal(joinList(props.initial.scope.senders));
  const [domains, setDomains] = createSignal(joinList(props.initial.scope.domains));
  const [tags, setTags] = createSignal(joinList(props.initial.exclude.tags));
  const [preview, setPreview] = createSignal<Preview | null>(null);
  const [message, setMessage] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const patch = (fn: (r: RuleInput) => void): void => { setRule((r) => { const c = clone(r); fn(c); return c; }); setPreview(null); };
  const current = (): RuleInput => {
    const c = clone(rule());
    c.scope.labels = splitList(labels());
    c.scope.senders = splitList(senders());
    c.scope.domains = splitList(domains());
    c.exclude.tags = splitList(tags());
    if (c.action.kind === 'keep') c.action.days = 0;
    if (c.action.kind !== 'move') c.action.target = null;
    return c;
  };
  const problem = (): string | null => { const p = ruleProblem(current(), props.policy); return p ? t(p.id, p.args) : null; };
  async function guarded(work: () => Promise<void>): Promise<void> {
    setBusy(true); setMessage('');
    try { await work(); } catch (e) { setMessage(e instanceof RetentionError ? e.message : t('retention-error')); } finally { setBusy(false); }
  }
  const folderChoices = (): Mailbox[] => props.folders.filter((f) => f.mailwomanSearchQuery === undefined);

  return <form class={css.section} data-testid="retention-editor" onSubmit={(e) => { e.preventDefault(); const p = problem(); if (p) { setMessage(p); return; } void guarded(() => props.onSave(current())); }}>
    <h3 class={css.subheading}>{props.initial.name === '' ? t('retention-new') : t('retention-edit')}</h3>
    <div class={css.field}>
      <label class={css.label} for="ret-name">{t('retention-name')}</label>
      <input id="ret-name" class={css.input} value={rule().name} maxLength={80} onInput={(e) => patch((r) => { r.name = e.currentTarget.value; })} />
    </div>
    <div class={css.row}>
      <div class={css.field}>
        <label class={css.label} for="ret-action">{t('retention-action')}</label>
        <select id="ret-action" class={css.select} value={rule().action.kind} onChange={(e) => patch((r) => { r.action.kind = e.currentTarget.value as ActionKind; })}>
          <For each={ACTION_KINDS}>{(k) => <option value={k}>{t(`retention-action-${k}`)}</option>}</For>
        </select>
      </div>
      <Show when={needsDays(rule().action.kind)}>
        <div class={css.field}>
          <label class={css.label} for="ret-days">{t('retention-days')}</label>
          <input id="ret-days" class={css.input} type="number" min="0" max="3650" value={rule().action.days} onInput={(e) => patch((r) => { r.action.days = e.currentTarget.valueAsNumber; })} />
        </div>
      </Show>
      <Show when={rule().action.kind === 'move'}>
        <div class={css.field}>
          <label class={css.label} for="ret-target">{t('retention-target')}</label>
          <select id="ret-target" class={css.select} value={rule().action.target ?? ''} onChange={(e) => patch((r) => { r.action.target = e.currentTarget.value || null; })}>
            <option value="">{t('retention-choose-folder')}</option>
            <For each={folderChoices()}>{(f) => <option value={f.id}>{f.name}</option>}</For>
          </select>
        </div>
      </Show>
    </div>
    <Show when={isDestructive(rule().action.kind)}>
      <p class={css.meta} data-testid="retention-limits">{props.policy.maxDays > 0 ? t('retention-limits-both', { min: props.policy.minDays, max: props.policy.maxDays }) : t('retention-limits-min', { min: props.policy.minDays })}</p>
    </Show>

    <fieldset class={css.bareFieldset}>
      <legend class={css.subheading}>{t('retention-scope')}</legend>
      <div class={css.field}>
        <label class={css.label} for="ret-folder">{t('retention-folder')}</label>
        <select id="ret-folder" class={css.select} value={rule().scope.folders[0] ?? ''} onChange={(e) => patch((r) => { r.scope.folders = e.currentTarget.value ? [e.currentTarget.value] : []; })}>
          <option value="">{t('retention-any-folder')}</option>
          <For each={['inbox', 'sent', 'archive', 'junk', 'trash']}>{(role) => <option value={`role:${role}`}>{t(`retention-role-${role}`)}</option>}</For>
          <For each={folderChoices().filter((f) => !f.role)}>{(f) => <option value={f.id}>{f.name}</option>}</For>
        </select>
      </div>
      <label class={css.check}><input type="checkbox" class={css.checkbox} checked={rule().scope.subfolders} onChange={(e) => patch((r) => { r.scope.subfolders = e.currentTarget.checked; })} />{t('retention-subfolders')}</label>
      <div class={css.field}><label class={css.label} for="ret-labels">{t('retention-labels')}</label><input id="ret-labels" class={css.input} value={labels()} onInput={(e) => { setLabels(e.currentTarget.value); setPreview(null); }} /></div>
      <div class={css.field}><label class={css.label} for="ret-senders">{t('retention-senders')}</label><input id="ret-senders" class={css.input} value={senders()} onInput={(e) => { setSenders(e.currentTarget.value); setPreview(null); }} /></div>
      <div class={css.field}><label class={css.label} for="ret-domains">{t('retention-domains')}</label><input id="ret-domains" class={css.input} value={domains()} onInput={(e) => { setDomains(e.currentTarget.value); setPreview(null); }} /></div>
      <div class={css.row}>
        <div class={css.field}><label class={css.label} for="ret-min-size">{t('retention-min-size')}</label><input id="ret-min-size" class={css.input} type="number" min="0" value={rule().scope.minSizeKb ?? ''} onInput={(e) => patch((r) => { r.scope.minSizeKb = Number.isNaN(e.currentTarget.valueAsNumber) ? null : e.currentTarget.valueAsNumber; })} /></div>
        <div class={css.field}><label class={css.label} for="ret-max-size">{t('retention-max-size')}</label><input id="ret-max-size" class={css.input} type="number" min="0" value={rule().scope.maxSizeKb ?? ''} onInput={(e) => patch((r) => { r.scope.maxSizeKb = Number.isNaN(e.currentTarget.valueAsNumber) ? null : e.currentTarget.valueAsNumber; })} /></div>
        <div class={css.field}>
          <label class={css.label} for="ret-read">{t('retention-read')}</label>
          <select id="ret-read" class={css.select} value={rule().scope.read} onChange={(e) => patch((r) => { r.scope.read = e.currentTarget.value as ReadState; })}>
            <For each={READ_STATES}>{(s) => <option value={s}>{t(`retention-read-${s}`)}</option>}</For>
          </select>
        </div>
        <div class={css.field}>
          <label class={css.label} for="ret-attach">{t('retention-attachment')}</label>
          <select id="ret-attach" class={css.select} value={rule().scope.hasAttachment === null ? 'any' : String(rule().scope.hasAttachment)} onChange={(e) => patch((r) => { r.scope.hasAttachment = e.currentTarget.value === 'any' ? null : e.currentTarget.value === 'true'; })}>
            <option value="any">{t('retention-attachment-any')}</option>
            <option value="true">{t('retention-attachment-with')}</option>
            <option value="false">{t('retention-attachment-without')}</option>
          </select>
        </div>
      </div>
    </fieldset>

    <fieldset class={css.bareFieldset}>
      <legend class={css.subheading}>{t('retention-exclude')}</legend>
      <label class={css.check}><input type="checkbox" class={css.checkbox} checked={rule().exclude.flagged} onChange={(e) => patch((r) => { r.exclude.flagged = e.currentTarget.checked; })} />{t('retention-exclude-flagged')}</label>
      <label class={css.check}><input type="checkbox" class={css.checkbox} checked={rule().exclude.unread} onChange={(e) => patch((r) => { r.exclude.unread = e.currentTarget.checked; })} />{t('retention-exclude-unread')}</label>
      <div class={css.field}><label class={css.label} for="ret-tags">{t('retention-exclude-tags')}</label><input id="ret-tags" class={css.input} value={tags()} onInput={(e) => { setTags(e.currentTarget.value); setPreview(null); }} /></div>
      <p class={css.meta}>{t('retention-exclude-note')}</p>
    </fieldset>

    <Show when={preview()}>{(p) => <div role="status" data-testid="retention-preview-result">
      <p>{p().skipped ? t(`retention-skipped-${p().skipped}`) : p().matched === 0 ? t('retention-preview-none') : t('retention-preview-count', { count: p().matched })}</p>
      <Show when={p().truncated}><p class={css.meta}>{t('retention-preview-truncated')}</p></Show>
      <Show when={p().sample.length > 0}><ul class={css.meta} aria-label={t('retention-preview-sample')}><For each={p().sample}>{(s) => <li>{s.subject || t('retention-no-subject')} · {s.from} · {when(s.receivedAt)} · {t(`retention-do-${s.action}`)}</li>}</For></ul></Show>
    </div>}</Show>
    <Show when={message()}><p role="alert" class={css.error}>{message()}</p></Show>
    <div class={css.actions}>
      <button type="button" class={css.ghost} disabled={busy() || problem() !== null} onClick={() => void guarded(async () => { setPreview(await props.api.preview(current())); })} data-testid="retention-preview">{t('retention-btn-preview')}</button>
      <button type="submit" class={css.button} disabled={busy()} data-testid="retention-save">{t('retention-save')}</button>
      <button type="button" class={css.ghost} disabled={busy()} onClick={props.onCancel}>{t('retention-cancel')}</button>
    </div>
  </form>;
}

export function RetentionRules(props: { folders: () => readonly Mailbox[]; api?: RetentionApi }): JSX.Element {
  const a = props.api ?? live;
  onMount(() => { void loadCatalog('retention'); });
  const [data, { refetch }] = createResource(() => a.load());
  const [editing, setEditing] = createSignal<{ id: string | null; input: RuleInput } | null>(null);
  const [message, setMessage] = createSignal('');
  const [summary, setSummary] = createSignal<Preview | null>(null);
  const [busy, setBusy] = createSignal(false);
  async function guarded(work: () => Promise<void>): Promise<void> {
    setBusy(true); setMessage('');
    try { await work(); await refetch(); } catch (e) { setMessage(e instanceof RetentionError ? e.message : t('retention-error')); } finally { setBusy(false); }
  }

  // Opened from a folder's context menu: start a rule for that folder.
  const startForFolder = (): void => {
    const id = retentionFolder();
    if (!id || editing()) return;
    const input = blankRule();
    input.scope.folders = [id];
    input.name = props.folders().find((f) => f.id === id)?.name ?? '';
    setEditing({ id: null, input });
    setRetentionFolder(null);
  };

  createEffect(() => { if (data.latest) startForFolder(); });

  return <section class={css.stack} aria-labelledby="retention-heading" data-testid="retention-rules">
    <div class={css.section}>
      <h2 id="retention-heading" class={css.heading}>{t('retention-title')}</h2>
      <p class={css.prose}>{t('retention-intro')}</p>
      <Show when={data.error}><p role="alert" class={css.error}>{t('retention-error')}</p></Show>
      <Show when={data.latest}>{(d) => <>
        <Show when={!d().policy.enabled}><p role="note" class={css.warn}>{t('retention-disabled')}</p></Show>
        <p class={css.meta} data-testid="retention-status">
          {t('retention-last-run', { when: when(d().settings.lastRunAt) })}
          {' · '}
          {d().settings.nextRunAt ? t('retention-next-run', { days: daysFromNow(d().settings.nextRunAt ?? 0) }) : t('retention-next-run-unknown')}
        </p>
        <div class={css.field}>
          <label class={css.check}><input type="checkbox" class={css.checkbox} checked={d().settings.enabled} disabled={busy() || !d().policy.enabled} onChange={(e) => void guarded(async () => { await a.settings({ enabled: e.currentTarget.checked }); })} data-testid="retention-enabled" />{t('retention-enabled')}</label>
          <label class={css.check}><input type="checkbox" class={css.checkbox} checked={d().settings.trashFirst} disabled={busy() || !d().policy.allowPermanentDelete} onChange={(e) => void guarded(async () => { await a.settings({ trashFirst: e.currentTarget.checked }); })} data-testid="retention-trash-first" />{t('retention-trash-first')}</label>
          <Show when={!d().policy.allowPermanentDelete}><p class={css.meta}>{t('retention-trash-forced')}</p></Show>
        </div>
        <div class={css.field}>
          <label class={css.label} for="ret-undo">{t('retention-undo')}</label>
          <input id="ret-undo" class={css.input} type="number" min="0" max={d().policy.undoDays} value={d().settings.undoDays} disabled={busy()} onChange={(e) => void guarded(async () => { await a.settings({ undoDays: e.currentTarget.valueAsNumber }); })} />
          <p class={css.meta}>{t('retention-undo-help', { max: d().policy.undoDays })}</p>
        </div>
        <div class={css.actions}>
          <Show when={d().settings.paused} fallback={<button type="button" class={css.ghost} disabled={busy()} onClick={() => void guarded(async () => { await a.settings({ pausedUntil: Math.floor(Date.now() / 1000) + 30 * 86400 }); })} data-testid="retention-pause">{t('retention-pause')}</button>}>
            <span class={css.meta}>{t('retention-paused-until', { when: when(d().settings.pausedUntil) })}</span>
            <button type="button" class={css.ghost} disabled={busy()} onClick={() => void guarded(async () => { await a.settings({ pausedUntil: 0 }); })} data-testid="retention-resume">{t('retention-resume')}</button>
          </Show>
        </div>
      </>}</Show>
    </div>

    <Show when={data.latest}>{(d) => <>
      <div class={css.section} data-testid="retention-policy">
        <h3 class={css.subheading}>{t('retention-policy')}</h3>
        <ul class={css.meta}>
          <li>{d().policy.maxDays > 0 ? t('retention-limits-both', { min: d().policy.minDays, max: d().policy.maxDays }) : t('retention-limits-min', { min: d().policy.minDays })}</li>
          <li>{t('retention-max-rules', { count: d().policy.maxRules })}</li>
          <li>{d().policy.allowPermanentDelete ? t('retention-permanent-allowed') : t('retention-permanent-denied', { days: d().policy.undoDays })}</li>
          <Show when={d().policy.protectedFolders.length > 0}><li>{t('retention-protected', { folders: d().policy.protectedFolders.join(', ') })}</li></Show>
          <For each={d().policy.mandatory}>{(m) => <li>{t('retention-mandatory', { folder: t(`retention-role-${m.role}`), days: m.days })}</li>}</For>
        </ul>
      </div>

      <div class={css.section}>
        <div class={css.actions}>
          <h3 class={css.subheading}>{t('retention-rules')}</h3>
          <button type="button" class={css.button} disabled={busy() || !d().policy.enabled || d().rules.length >= d().policy.maxRules} onClick={() => setEditing({ id: null, input: blankRule() })} data-testid="retention-add">{t('retention-add')}</button>
        </div>
        <Show when={d().rules.length === 0}><p class={css.meta}>{t('retention-no-rules')}</p></Show>
        <ul class={css.list}>
          <For each={d().rules}>{(r) => <li class={css.item} data-testid={`retention-rule-${r.id}`}>
            <div class={css.itemMain}>
              <span class={css.itemName}>{r.name}<Show when={!r.enabled}>{' '}<span class={css.badge}>{t('retention-rule-paused')}</span></Show></span>
              <span class={css.meta}>
                {r.scope.folders.length > 0 ? r.scope.folders.map((f) => folderName(props.folders(), f)).join(', ') : t('retention-any-folder')}
                {' · '}
                {(() => { const m = actionMessage(r, d().policy.allowPermanentDelete && !d().settings.trashFirst); return t(m.id, m.args); })()}
              </span>
            </div>
            <div class={css.actions}>
              <label class={css.check}><input type="checkbox" class={css.checkbox} checked={r.enabled} disabled={busy()} onChange={(e) => void guarded(async () => { await a.update(r.id, { ...toInput(r), enabled: e.currentTarget.checked }); })} aria-label={t('retention-rule-enabled', { name: r.name })} />{t('retention-rule-on')}</label>
              <button type="button" class={css.ghost} disabled={busy()} onClick={() => setEditing({ id: r.id, input: toInput(r) })}>{t('retention-edit-short')}</button>
              <button type="button" class={css.ghost} disabled={busy()} onClick={() => void guarded(async () => { await a.remove(r.id); })}>{t('retention-delete')}</button>
            </div>
          </li>}</For>
        </ul>
      </div>

      <Show when={editing()}>{(e) => <Editor
        initial={e().input} folders={props.folders()} policy={d().policy} api={a}
        onSave={async (rule) => { if (e().id) await a.update(e().id as string, rule); else await a.create(rule); setEditing(null); await refetch(); }}
        onCancel={() => setEditing(null)}
      />}</Show>

      <Show when={d().templates.length > 0}>
        <div class={css.section} data-testid="retention-templates">
          <h3 class={css.subheading}>{t('retention-templates')}</h3>
          <p class={css.meta}>{t('retention-templates-help')}</p>
          <ul class={css.list}>
            <For each={d().templates}>{(tp) => <li class={css.item}>
              <span class={css.itemName}>{tp.name}</span>
              <button type="button" class={css.ghost} disabled={busy() || !d().policy.enabled || d().rules.length >= d().policy.maxRules} onClick={() => void guarded(async () => { await a.adopt(tp.id); })} data-testid={`retention-adopt-${tp.id}`}>{t('retention-adopt')}</button>
            </li>}</For>
          </ul>
        </div>
      </Show>

      <div class={css.section}>
        <h3 class={css.subheading}>{t('retention-run-heading')}</h3>
        <div class={css.actions}>
          <button type="button" class={css.ghost} disabled={busy()} onClick={() => void guarded(async () => { setSummary(await a.preview()); })} data-testid="retention-preview-all">{t('retention-preview-all')}</button>
          <button type="button" class={css.button} disabled={busy() || !d().policy.enabled} onClick={() => void guarded(async () => { setSummary(await a.run()); })} data-testid="retention-run">{t('retention-btn-run')}</button>
        </div>
        <Show when={summary()}>{(s) => <div role="status" data-testid="retention-summary">
          <p>{s().skipped ? t(`retention-skipped-${s().skipped}`) : s().done && (s().done.moved + s().done.trashed + s().done.purged + s().done.markedRead + s().done.stamped) > 0
            ? t('retention-run-result', { count: s().done.moved + s().done.trashed + s().done.purged + s().done.markedRead + s().done.stamped })
            : s().matched === 0 ? t('retention-preview-none') : t('retention-preview-count', { count: s().matched })}</p>
          <Show when={s().sample.length > 0}><ul class={css.meta} aria-label={t('retention-preview-sample')}><For each={s().sample}>{(x) => <li>{x.subject || t('retention-no-subject')} · {x.from} · {when(x.receivedAt)} · {t(`retention-do-${x.action}`)}</li>}</For></ul></Show>
        </div>}</Show>
        <Show when={d().runs.length > 0}>
          <table class={css.meta} data-testid="retention-runs">
            <caption>{t('retention-history')}</caption>
            <thead><tr><th scope="col">{t('retention-history-when')}</th><th scope="col">{t('retention-history-matched')}</th><th scope="col">{t('retention-history-changed')}</th><th scope="col">{t('retention-history-errors')}</th></tr></thead>
            <tbody><For each={d().runs}>{(r) => <tr><td>{when(r.startedAt)}</td><td>{r.matched}</td><td>{r.acted}</td><td>{r.errors}</td></tr>}</For></tbody>
          </table>
        </Show>
      </div>
    </>}</Show>
    <Show when={message()}><p role="alert" class={css.error}>{message()}</p></Show>
  </section>;
}
