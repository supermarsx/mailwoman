// Settings > Retention > Folder defaults: one line per folder, in tree order, with
// what happens to mail in it after a number of days (or "inherit" for none of its
// own). The administrator's floor and ceiling are shown and checked here; the
// server checks them again on save. Saved through the same settings call as the
// rest of the page (`folderDefaults`), so rules and defaults stay in step.
import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import type { Mailbox } from '../../api/jmap-types.ts';
import type { FolderDefault, RetentionPolicy } from '../../api/retention.ts';
import {
  FOLDER_CHOICES, defaultsPayload, flattenFolders, folderProblem, isDestructive, settingsFromDefaults,
  type DefaultsPayload, type FolderChoice, type FolderSetting,
} from './retentionModel.ts';
import * as css from './styles.css.ts';

export function FolderRetention(props: {
  folders: readonly Mailbox[];
  defaults: readonly FolderDefault[];
  policy: RetentionPolicy;
  busy: boolean;
  onSave: (payload: DefaultsPayload[]) => Promise<void>;
}): JSX.Element {
  const flat = createMemo(() => flattenFolders(props.folders.filter((f) => f.mailwomanSearchQuery === undefined)));
  const [edits, setEdits] = createSignal<Map<string, FolderSetting>>(settingsFromDefaults(props.defaults));
  const [dirty, setDirty] = createSignal(false);
  const setting = (id: string): FolderSetting => edits().get(id) ?? { choice: 'inherit', days: props.policy.minDays };
  const patch = (id: string, next: Partial<FolderSetting>): void => {
    setEdits((m) => new Map(m).set(id, { ...setting(id), ...next }));
    setDirty(true);
  };
  const problems = createMemo(() => flat().flatMap((f) => { const p = folderProblem(setting(f.id), props.policy); return p ? [{ id: f.id, name: f.name, text: t(p.id, p.args) }] : []; }));
  const mandatory = (role: string | null): number | null => (role ? props.policy.mandatory.find((m) => m.role === role)?.days ?? null : null);
  const protectedFolder = (f: { name: string; role: string | null }): boolean =>
    props.policy.protectedFolders.some((p) => p === f.name || p === f.role);

  return <div class={css.section} data-testid="retention-folder-defaults">
    <h3 class={css.subheading}>{t('retention-fd-title')}</h3>
    <p class={css.meta}>{t('retention-fd-help')}</p>
    <p class={css.meta} data-testid="retention-fd-limits">{props.policy.maxDays > 0 ? t('retention-limits-both', { min: props.policy.minDays, max: props.policy.maxDays }) : t('retention-limits-min', { min: props.policy.minDays })}</p>
    <ul class={css.list}>
      <For each={flat()}>{(f) => {
        const s = (): FolderSetting => setting(f.id);
        const floor = (): number | null => mandatory(f.role);
        const locked = (): boolean => protectedFolder(f) || floor() !== null;
        return <li class={css.item} data-testid={`retention-fd-${f.id}`} style={{ 'padding-inline-start': `${f.depth * 1.25}rem` }}>
          <div class={css.itemMain}>
            <span class={css.itemName}>{f.name}</span>
            <Show when={floor() !== null}><span class={css.meta}>{t('retention-fd-mandatory', { days: floor() ?? 0 })}</span></Show>
            <Show when={protectedFolder(f)}><span class={css.meta}>{t('retention-fd-protected')}</span></Show>
          </div>
          <div class={css.actions}>
            <select class={css.select} aria-label={t('retention-fd-action', { folder: f.name })} value={s().choice} disabled={props.busy || locked()}
              onChange={(e) => patch(f.id, { choice: e.currentTarget.value as FolderChoice })}>
              <For each={FOLDER_CHOICES}>{(c) => <option value={c}>{c === 'inherit' ? t('retention-fd-inherit') : t(`retention-action-${c}`)}</option>}</For>
            </select>
            <Show when={s().choice !== 'inherit' && s().choice !== 'keep'}>
              <input class={css.input} type="number" min={isDestructive(s().choice as never) ? props.policy.minDays : 0}
                max={props.policy.maxDays > 0 && isDestructive(s().choice as never) ? props.policy.maxDays : 3650}
                aria-label={t('retention-fd-days', { folder: f.name })} value={s().days} disabled={props.busy || locked()}
                onInput={(e) => patch(f.id, { days: e.currentTarget.valueAsNumber })} />
            </Show>
          </div>
        </li>;
      }}</For>
    </ul>
    <Show when={problems().length > 0}>
      <ul role="alert" class={css.error} data-testid="retention-fd-problems"><For each={problems()}>{(p) => <li>{p.name}: {p.text}</li>}</For></ul>
    </Show>
    <div class={css.actions}>
      <button type="button" class={css.button} data-testid="retention-fd-save" disabled={props.busy || !dirty() || problems().length > 0}
        onClick={() => void props.onSave(defaultsPayload(flat(), edits(), props.defaults)).then(() => setDirty(false))}>{t('retention-fd-save')}</button>
    </div>
  </div>;
}
