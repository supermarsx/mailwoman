import { createSignal, For, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { toast } from '../../components/toast/index.ts';
import type { SavedSearch } from './types.ts';
import type { SettingsService } from './service.ts';
import { isTransientFailure, showFailure } from '../../components/recoverableAction.ts';
import { useEditorGuard } from './entityEditor.tsx';
import * as css from './styles.css.ts';
import * as folderCss from './searchFolder.css.ts';
import * as layout from './editor.css.ts';

const fields = ['text', 'from', 'to', 'subject', 'body', 'filename', 'inMailbox', 'after', 'before'] as const;

export function SearchFolderEditor(props: { initial?: SavedSearch; service: SettingsService; onSaved: () => void | Promise<void>; onCancel: () => void | Promise<void> }): JSX.Element {
  const initial = props.initial ?? { id: '', name: '', queryJson: '{}', asFolder: true };
  const [name, setName] = createSignal(initial.name);
  const [asFolder, setAsFolder] = createSignal(initial.asFolder);
  let parsed: Record<string, unknown> = {};
  let parseError = false;
  try { const value: unknown = JSON.parse(initial.queryJson); if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error(); parsed = value as Record<string, unknown>; } catch { parseError = true; }
  const [filter, setFilter] = createSignal(parsed);
  const snapshot = (): string => JSON.stringify([name(), asFolder(), filter()]);
  const [baseline, setBaseline] = createSignal(snapshot());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal(parseError ? t('settings-search-invalid') : '');
  const guard = useEditorGuard(() => baseline() !== snapshot(), busy);
  const change = (key: string, value: unknown): void => { setFilter(previous => {
    const next = { ...previous };
    if (value === '' || value === undefined) delete next[key]; else next[key] = value;
    return next;
  }); };
  async function save(): Promise<void> {
    if (busy() || parseError) return;
    if (!name().trim()) { setError(t('settings-search-name-required')); return; }
    setBusy(true); setError('');
    const value = { id: initial.id, name: name().trim(), asFolder: asFolder(), queryJson: JSON.stringify(filter()) }, submitted = snapshot();
    // t91: a create is keyed (service.ts); a transient failure gets one toast
    // whose Retry saves the same search again. A refusal stays at the form.
    const persist = async (): Promise<void> => {
      await props.service.upsertSavedSearch(value);
      setBaseline(submitted); toast.success(t('settings-search-saved', { name: value.name })); await props.onSaved();
    };
    try { await persist(); }
    catch (cause) {
      if (isTransientFailure(cause)) showFailure({ key: 'settings-search-save', error: cause, message: (reason) => t('settings-search-save-failed', { reason }), retry: persist });
      else setError(cause instanceof Error ? cause.message : t('settings-search-error'));
    }
    finally { setBusy(false); }
  }
  return <section class={css.section} aria-label={t('settings-search-editor')}>
    <h2 class={css.heading}>{t('settings-search-editor')}</h2>
    <p class={css.prose}>{t('settings-search-editor-help')}</p>
    <form onSubmit={event => { event.preventDefault(); void save(); }}>
      <fieldset class={layout.fields} disabled={busy() || parseError}>
        <label class={css.field}><span class={css.label}>{t('settings-search-name')}</span><input class={css.input} required maxLength={200} value={name()} onInput={event => setName(event.currentTarget.value)} /></label>
        <div class={folderCss.criteria}><For each={fields}>{key => <label class={css.field}>
          <span class={css.label}>{t(`settings-search-field-${key}`)}</span>
          <input class={css.input} type="text" placeholder={key === 'after' || key === 'before' ? '2026-01-01T00:00:00Z' : undefined} value={typeof filter()[key] === 'string' ? String(filter()[key]) : ''} onInput={event => change(key, event.currentTarget.value)} />
        </label>}</For></div>
        <label class={css.field}><span class={css.label}>{t('settings-search-attachment')}</span><select class={css.select} value={filter().hasAttachment === undefined ? '' : String(filter().hasAttachment)} onChange={event => change('hasAttachment', event.currentTarget.value === '' ? undefined : event.currentTarget.value === 'true')}><option value="">{t('settings-search-any')}</option><option value="true">{t('settings-search-with-attachments')}</option><option value="false">{t('settings-search-without-attachments')}</option></select></label>
        <label class={css.check}><input class={css.checkbox} type="checkbox" checked={filter().notKeyword === '$seen'} onChange={event => change('notKeyword', event.currentTarget.checked ? '$seen' : undefined)} /><span>{t('settings-search-unread')}</span></label>
        <label class={css.check}><input class={css.checkbox} type="checkbox" checked={filter().hasKeyword === '$flagged'} onChange={event => change('hasKeyword', event.currentTarget.checked ? '$flagged' : undefined)} /><span>{t('settings-search-flagged')}</span></label>
        <label class={css.check}><input class={css.checkbox} type="checkbox" checked={asFolder()} onChange={event => setAsFolder(event.currentTarget.checked)} /><span>{t('settings-search-as-folder-label')}</span></label>
        <div class={css.actions}><button class={css.button} type="submit">{t('settings-save')}</button></div>
      </fieldset>
    </form>
    <button class={css.ghost} type="button" disabled={busy()} onClick={() => void guard.leave(props.onCancel)}>{t('settings-cancel')}</button>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>{guard.dialog()}
  </section>;
}
