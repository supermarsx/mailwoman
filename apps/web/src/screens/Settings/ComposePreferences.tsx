import { Show, type JSX } from 'solid-js';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { t } from '../../i18n';
import * as css from './styles.css.ts';

export function ComposePreferences(): JSX.Element {
  const quote = useSetting<'classic'|'outlook'>('mail.quoteStyle', 'classic', (v): v is 'classic'|'outlook' => v === 'classic' || v === 'outlook');
  const disabled = () => !quote.entry() || quote.locked() || quote.saving();
  return <section class={css.section} aria-label={t('settings-compose-title')}><h3 class={css.heading}>{t('settings-compose-title')}</h3>
    <div class={css.field}><label for="quote-style">{t('settings-quote-style')}</label><select id="quote-style" class={css.select} value={quote.value()} disabled={disabled()} onChange={event => void quote.set(event.currentTarget.value as 'classic'|'outlook')} aria-describedby="quote-style-help">
      <option value="classic">{t('settings-quote-classic')}</option><option value="outlook">{t('settings-quote-outlook')}</option>
    </select><p id="quote-style-help" class={css.meta}>{t('settings-quote-help')}</p><LockNote locked={quote.locked()}/>
    <Show when={quote.error()}><p role="alert">{quote.error()}</p></Show>
    <button type="button" class={css.button} disabled={disabled()} onClick={() => void quote.reset()}>{t('reader-inherit')}</button></div>
  </section>;
}
