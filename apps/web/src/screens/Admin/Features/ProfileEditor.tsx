// One feature profile's name and switches (t39 N9).
//
// Each catalogue feature is one row: "Not set" (the profile leaves it to the
// next profile, the domain or the default), "Allowed" or "Not allowed", and a
// lock. A locked value cannot be changed by the user or a lower level. Values the
// profile holds for keys outside the catalogue are kept as they are.

import { For, Show, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { FormField } from '../shell/FormField.tsx';
import { settingHelp, settingLabel } from '../Settings/labels.ts';
import type { FeatureCatalogue, FeatureProfile, ProfileValue } from './api.ts';
import * as css from '../admin.css.ts';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

type Choice = 'unset' | 'on' | 'off';

export interface ProfileEditorProps {
  catalogue: FeatureCatalogue;
  profile?: FeatureProfile;
  onSave: (name: string, values: Record<string, ProfileValue>) => Promise<boolean>;
  onDelete?: () => void;
}

/** The capability key a feature row edits (the catalogue gives one per feature). */
const keyOf = (f: FeatureCatalogue['catalogue'][number]): string | undefined => f.capabilityKeys[0];

export function ProfileEditor(props: ProfileEditorProps): JSX.Element {
  const initial = props.profile?.values ?? {};
  const [name, setName] = createSignal(props.profile?.name ?? '');
  const [values, setValues] = createSignal<Record<string, ProfileValue>>({ ...initial });
  const [busy, setBusy] = createSignal(false);
  const [nameError, setNameError] = createSignal<string | null>(null);

  const choice = (key: string): Choice => {
    const v = values()[key];
    return v === undefined ? 'unset' : v.value === true ? 'on' : 'off';
  };
  const setChoice = (key: string, c: Choice): void => {
    setValues((all) => {
      const next = { ...all };
      if (c === 'unset') delete next[key];
      else next[key] = { value: c === 'on', locked: all[key]?.locked ?? false };
      return next;
    });
  };
  const setLocked = (key: string, locked: boolean): void => {
    setValues((all) => (all[key] ? { ...all, [key]: { ...all[key]!, locked } } : all));
  };

  async function submit(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const trimmed = name().trim();
    if (trimmed === '' || trimmed.length > 128) {
      setNameError(t('admin-features-name-invalid'));
      return;
    }
    setNameError(null);
    setBusy(true);
    try {
      await props.onSave(trimmed, values());
    } finally {
      setBusy(false);
    }
  }

  return (
    <form class={ux.form} onSubmit={(e) => void submit(e)} aria-label={t('admin-features-editor')}>
      <FormField label={t('admin-features-name')} help={t('admin-features-name-help')} error={nameError()}>
        {(w) => (
          <input id={w.id} class={ux.control} value={name()} maxlength={128} aria-required="true"
            aria-describedby={w.describedBy} aria-invalid={w.invalid}
            onInput={(e) => setName(e.currentTarget.value)} />
        )}
      </FormField>
      <fieldset class={ux.fieldset}>
        <legend class={ux.legend}>{t('admin-features-switches')}</legend>
        <p class={ux.fieldsetHelp}>{t('admin-features-switches-help')}</p>
        <div class={css.tableWrap} tabindex="0" role="group" aria-label={t('admin-features-switches')}>
          <table class={css.table}>
            <caption class={shellCss.srOnly}>{t('admin-features-switches')}</caption>
            <thead><tr>
              <th scope="col">{t('admin-features-col-feature')}</th>
              <th scope="col">{t('admin-features-col-value')}</th>
              <th scope="col">{t('admin-features-col-lock')}</th>
            </tr></thead>
            <tbody>
              <For each={props.catalogue.catalogue.filter((f) => keyOf(f) !== undefined)}>
                {(f) => {
                  const key = keyOf(f)!;
                  const label = (): string => settingLabel(key);
                  return (
                    <tr data-feature={f.id}>
                      <th scope="row"><span dir="auto">{label()}</span><br /><small class={css.note}>{settingHelp(key)}</small></th>
                      <td>
                        <select aria-label={t('admin-features-value-for', { feature: label() })} value={choice(key)}
                          onChange={(e) => setChoice(key, e.currentTarget.value as Choice)}>
                          <option value="unset">{t('admin-features-unset')}</option>
                          <option value="on">{t('admin-features-on')}</option>
                          <option value="off">{t('admin-features-off')}</option>
                        </select>
                      </td>
                      <td>
                        <label class={ux.checkboxRow}>
                          <input type="checkbox" checked={values()[key]?.locked ?? false} disabled={choice(key) === 'unset'}
                            onChange={(e) => setLocked(key, e.currentTarget.checked)} />
                          <span>{t('admin-features-lock')}<span class={shellCss.srOnly}> {label()}</span></span>
                        </label>
                      </td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </div>
      </fieldset>
      <div class={ux.pageActions}>
        <button type="submit" class="btn btn--primary" disabled={busy()}>{props.profile ? t('admin-features-save') : t('admin-features-create')}</button>
        <Show when={props.onDelete}>{(del) => <button type="button" class="btn btn--danger" disabled={busy()} onClick={() => del()()}>{t('admin-features-delete')}</button>}</Show>
      </div>
    </form>
  );
}
