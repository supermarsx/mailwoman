import { For, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n/index.ts';
import { loadCatalog } from '../../../i18n/catalog.ts';
// i18n: the ids this file shows live in these catalogs, which load with it.
void loadCatalog('compose-insert');

export const browserTimeZone = (): string => {
  try { return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'; } catch { return 'UTC'; }
};

const zones = (): string[] => {
  try { return (Intl as unknown as { supportedValuesOf?: (k: string) => string[] }).supportedValuesOf?.('timeZone') ?? []; } catch { return []; }
};

/** A time zone field with the browser's list as suggestions; an unknown name is not accepted. */
export function TimeZoneInput(props: { value: string; onChange: (tz: string) => void; testId: string }): JSX.Element {
  const id = createUniqueId();
  const valid = (tz: string): boolean => { try { new Intl.DateTimeFormat('en', { timeZone: tz }); return true; } catch { return false; } };
  return (
    <label>{t('ins-timezone')}
      <input list={id} data-testid={props.testId} value={props.value} aria-invalid={!valid(props.value)} onInput={(e) => { const v = e.currentTarget.value; if (valid(v)) props.onChange(v); }} />
      <datalist id={id}><For each={zones()}>{(z) => <option value={z} />}</For></datalist>
    </label>
  );
}
