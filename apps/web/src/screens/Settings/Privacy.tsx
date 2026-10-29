import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { remotePolicy, type RemoteMode } from '../../api/remote-content.ts';
import { isNativeAuth } from '../../api/serverAddress.ts';
import { TrustedLinkDomains } from './TrustedLinkDomains.tsx';
import * as css from './styles.css.ts';
export function Privacy(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); void loadCatalog('settings-policy'); });
  const mode = useSetting<RemoteMode>('privacy.remoteContent', 'ask', (v): v is RemoteMode => typeof v === 'string' && ['block','ask','proxy','direct'].includes(v));
  const trackers = useSetting('privacy.trackers', true);
  const links = useSetting('privacy.cleanLinks', true);
  const stripMailerHeaders = useSetting('privacy.stripMailerHeaders', true);
  const imageDownloads=useSetting('privacy.remoteImageDownloads',false);
  const imageToast=useSetting('privacy.remoteImagesToast',true);
  const [policy] = createResource(async () => { try { return await remotePolicy(); } catch { return null; } });
  const canDirect = (): boolean => !isNativeAuth() && policy.state === 'ready' && policy.latest?.allowDirect === true;
  return <section class={css.section} aria-label={t('privacy-title')}>
    <h3 class={css.heading}>{t('privacy-title')}</h3><p class={css.meta}>{t('privacy-help')}</p>
    <div class={css.field}><label for="privacy-mode">{t('privacy-remote-mode')}</label><select id="privacy-mode" class={css.select} value={mode.value() === 'direct' && !canDirect() ? 'ask' : mode.value()} disabled={!mode.entry() || mode.locked() || mode.saving()} onChange={e => void mode.set(e.currentTarget.value as RemoteMode)}><For each={['block','ask','proxy','direct'] as const}>{value => <option value={value} disabled={value === 'direct' && !canDirect()}>{t(`privacy-mode-${value}`)}</option>}</For></select><LockNote id="privacy-mode-lock" locked={mode.locked()} /><Show when={!canDirect()}><p class={css.meta}>{t(isNativeAuth() ? 'privacy-direct-native' : 'privacy-direct-unavailable')}</p></Show><Show when={mode.error()}><p role="alert">{mode.error()}</p></Show><button type="button" class="btn" disabled={mode.locked() || mode.saving() || !mode.entry()} onClick={() => void mode.reset()}>{t('privacy-inherit')}</button></div>
    <For each={[{ setting: trackers, label: 'privacy-trackers' }, { setting: links, label: 'privacy-clean-links' },{setting:imageDownloads,label:'privacy-image-downloads'},{setting:imageToast,label:'privacy-image-toast'}, ...(policy.state === 'ready' && policy.latest?.inlineParts === true ? [{setting:stripMailerHeaders,label:'privacy-strip-mailer-headers'}] : [])]}>{item => <div class={css.field}><label><input type="checkbox" checked={item.setting.value()} disabled={!item.setting.entry() || item.setting.locked() || item.setting.saving()} onChange={e => void item.setting.set(e.currentTarget.checked)} /> {t(item.label)}</label><LockNote locked={item.setting.locked()} /><Show when={item.setting.error()}><p role="alert">{item.setting.error()}</p></Show><button type="button" class="btn" disabled={item.setting.locked() || item.setting.saving() || !item.setting.entry()} onClick={() => void item.setting.reset()}>{t('privacy-inherit')}</button></div>}</For>
    <Show when={policy.state === 'ready' && policy.latest?.inlineParts === true}><p class={css.meta}>{t('privacy-strip-mailer-help')}</p></Show>
    <TrustedLinkDomains />
  </section>;
}
