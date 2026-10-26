import { For, Show, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { createStore } from 'solid-js/store';
import { Icon } from '../../components/Icon.tsx';
import { t, loadCatalog } from '../../i18n';
import { registerNavigationGuard } from '../../shell/navigationGuard.ts';
import { useEditorGuard } from '../../screens/Settings/entityEditor.tsx';
import type { ContactDraft } from '../../state/slices/contacts.ts';
import type { ContactCard, ContactEmail, ContactValue } from '../../api/pim-types.ts';
import * as css from './contacts.css.ts';

export function ContactEditor(props: {
  mode: 'create' | 'edit';
  source?: ContactCard;
  onCancel: () => void;
  onSave: (draft: ContactDraft) => Promise<string | boolean | null>;
  onSaved: (result: string | boolean) => void;
}): JSX.Element {
  onMount(() => { void loadCatalog('settings'); });
  const src = props.source;
  const [draft, setDraft] = createStore<{
    name: ContactCard['name'];
    organizations: string[];
    titles: string[];
    emails: ContactEmail[];
    phones: ContactValue[];
    notes: string;
    pgpKey: string;
    isFavorite: boolean;
  }>({
    name: src ? { ...src.name } : { full: '', given: '', surname: '', prefix: '', suffix: '' },
    organizations: src ? [...src.organizations] : [],
    titles: src ? [...src.titles] : [],
    emails: src ? src.emails.map((e) => ({ ...e })) : [{ context: '', value: '', pref: 0 }],
    phones: src ? src.phones.map((p) => ({ ...p })) : [{ context: '', value: '' }],
    notes: src?.notes ?? '',
    pgpKey: src?.pgpKey ?? '',
    isFavorite: src?.isFavorite ?? false,
  });

  const baseline = JSON.stringify(draft);
  const [saved, setSaved] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal(false);
  let alive = true;
  onCleanup(() => { alive = false; });
  const guard = useEditorGuard(() => busy() || (!saved() && JSON.stringify(draft) !== baseline), busy);
  onCleanup(registerNavigationGuard(guard.confirm));
  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    if (busy()) return;
    setBusy(true); setError(false);
    const full = draft.name.full.trim() || [draft.name.given, draft.name.surname].filter(Boolean).join(' ').trim();
    const patch: ContactDraft = {
      name: { ...draft.name, full },
      organizations: draft.organizations.filter((o) => o.trim().length > 0),
      titles: draft.titles.filter((t) => t.trim().length > 0),
      emails: draft.emails.filter((em) => em.value.trim().length > 0),
      phones: draft.phones.filter((p) => p.value.trim().length > 0),
      notes: draft.notes,
      pgpKey: draft.pgpKey.trim().length > 0 ? draft.pgpKey.trim() : null,
      isFavorite: draft.isFavorite,
    };
    try {
      const result = await props.onSave(patch);
      if (!alive) return;
      if (result) { setSaved(true); props.onSaved(result); }
      else setError(true);
    } catch { if (alive) setError(true); }
    finally { if (alive) setBusy(false); }
  }

  return (
    <>
    <form class={css.card} onSubmit={e => void submit(e)} aria-busy={busy()} aria-label={props.mode === 'create' ? t('contacts-new-contact') : t('contacts-edit-contact')}>
      <h1 class={css.cardName}>{props.mode === 'create' ? t('contacts-new-contact') : t('contacts-edit-contact')}</h1>

      <fieldset disabled={busy()} class={css.editorSection}><legend>{t('contacts-editor-person')}</legend>
      <div class={css.fieldGroup}>
        <label class={css.fieldLabel} for="c-full">{t('contacts-full-name')}</label>
        <input
          id="c-full"
          class={css.input}
          aria-label={t('contacts-full-name')}
          value={draft.name.full}
          onInput={(e) => setDraft('name', 'full', e.currentTarget.value)}
        />
      </div>
      <div class={css.fieldRow}>
        <input class={css.input} aria-label={t('contacts-given')} placeholder={t('contacts-given-placeholder')} value={draft.name.given} onInput={(e) => setDraft('name', 'given', e.currentTarget.value)} />
        <input class={css.input} aria-label={t('contacts-surname')} placeholder={t('contacts-surname-placeholder')} value={draft.name.surname} onInput={(e) => setDraft('name', 'surname', e.currentTarget.value)} />
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('contacts-organization')}</span>
        <input class={css.input} aria-label={t('contacts-organization')} value={draft.organizations[0] ?? ''} onInput={(e) => setDraft('organizations', [e.currentTarget.value, ...draft.organizations.slice(1)])} />
        <input class={css.input} aria-label={t('contacts-job-title')} placeholder={t('contacts-job-title-placeholder')} value={draft.titles[0] ?? ''} onInput={(e) => setDraft('titles', [e.currentTarget.value, ...draft.titles.slice(1)])} />
      </div>

      </fieldset>
      <fieldset disabled={busy()} class={css.editorSection}><legend>{t('contacts-editor-reach')}</legend>
      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('contacts-email')}</span>
        <For each={draft.emails}>
          {(em, i) => (
            <div class={css.fieldRow}>
              <input class={css.input} aria-label={t('contacts-email-n', { n: i() + 1 })} type="email" value={em.value} onInput={(e) => setDraft('emails', i(), 'value', e.currentTarget.value)} />
              <input class={css.select} aria-label={t('contacts-email-label', { n: i() + 1 })} placeholder={t('contacts-email-label-placeholder')} value={em.context} onInput={(e) => setDraft('emails', i(), 'context', e.currentTarget.value)} />
              <button type="button" class={css.buttonGhost} aria-label={t('contacts-remove-email', { n: i() + 1 })} onClick={() => setDraft('emails', (list) => list.filter((_, j) => j !== i()))}><Icon name="minus" /></button>
            </div>
          )}
        </For>
        <button type="button" class={css.buttonGhost} onClick={() => setDraft('emails', (list) => [...list, { context: '', value: '', pref: 0 }])}>{t('contacts-add-email')}</button>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('contacts-phone')}</span>
        <For each={draft.phones}>
          {(p, i) => (
            <div class={css.fieldRow}>
              <input class={css.input} aria-label={t('contacts-phone-n', { n: i() + 1 })} value={p.value} onInput={(e) => setDraft('phones', i(), 'value', e.currentTarget.value)} />
              <button type="button" class={css.buttonGhost} aria-label={t('contacts-remove-phone', { n: i() + 1 })} onClick={() => setDraft('phones', (list) => list.filter((_, j) => j !== i()))}><Icon name="minus" /></button>
            </div>
          )}
        </For>
        <button type="button" class={css.buttonGhost} onClick={() => setDraft('phones', (list) => [...list, { context: '', value: '' }])}>{t('contacts-add-phone')}</button>
      </div>

      </fieldset>
      <fieldset disabled={busy()} class={css.editorSection}><legend>{t('contacts-editor-other')}</legend>
      <div class={css.fieldGroup}>
        <label class={css.fieldLabel} for="c-notes">{t('contacts-notes')}</label>
        <textarea id="c-notes" class={css.input} aria-label={t('contacts-notes')} rows={3} value={draft.notes} onInput={(e) => setDraft('notes', e.currentTarget.value)} />
      </div>

      <div class={css.fieldGroup}>
        <label class={css.fieldLabel} for="c-key">{t('contacts-security-key-label')}</label>
        <textarea id="c-key" class={css.input} aria-label={t('contacts-security-key')} rows={2} placeholder={t('contacts-security-key-placeholder')} value={draft.pgpKey} onInput={(e) => setDraft('pgpKey', e.currentTarget.value)} />
      </div>

      <label class={css.fieldRow}>
        <input type="checkbox" aria-label={t('contacts-favorite-word')} checked={draft.isFavorite} onChange={(e) => setDraft('isFavorite', e.currentTarget.checked)} />
        <span>{t('contacts-favorite-word')}</span>
      </label>

      </fieldset>
      <Show when={error()}><p role="alert">{t('contacts-editor-save-error')}</p></Show>
      <div class={css.actions}>
        <button type="button" class={css.buttonGhost} disabled={busy()} onClick={() => void guard.leave(props.onCancel)}>{t('common-cancel')}</button>
        <button type="submit" disabled={busy()} class={css.button}>{t('common-save')}</button>
      </div>
    </form>
    {guard.dialog()}
    </>
  );
}
