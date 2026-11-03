import { createResource, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { LockNote } from './LockNote.tsx';
import { remotePolicy, type RemoteMode } from '../../api/remote-content.ts';
import { isNativeAuth } from '../../api/serverAddress.ts';
import { TrustedLinkDomains } from './TrustedLinkDomains.tsx';
import {featureOn,featuresLoaded} from '../../state/features.ts';
import * as css from './styles.css.ts';
export function Privacy(): JSX.Element {
  onMount(() => { void loadCatalog('privacy'); void loadCatalog('settings-policy'); });
  const mode = useSetting<RemoteMode>('privacy.remoteContent', 'ask', (v): v is RemoteMode => typeof v === 'string' && ['block','ask','proxy','direct'].includes(v));
  const unsafeHtml=useSetting<string>('mail.reader.unsafeHtmlMode','sanitized',(v):v is string=>typeof v==='string'&&['ask','strict','sanitized'].includes(v));
  const originalHtml=useSetting('mail.reader.allowOriginalHtml',false),originalCap=useSetting('cap.reading.original_html',true);
  const originalAvailable=()=>featuresLoaded()&&featureOn('reading.original_html')&&originalCap.value()&&!(unsafeHtml.value()==='strict'&&unsafeHtml.locked());
  const trackers = useSetting('privacy.trackers', true);
  const trackerOverride=useSetting('privacy.trackerOverrideAllowed',true);
  const trackerChoiceAllowed=()=>!(trackers.locked()&&trackers.value());
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
    <div class={css.field}><label for="privacy-unsafe-html">{t('privacy-unsafe-html')}</label><select id="privacy-unsafe-html" class={css.select} value={unsafeHtml.value()} disabled={!unsafeHtml.entry()||unsafeHtml.locked()||unsafeHtml.saving()} onChange={event=>void unsafeHtml.set(event.currentTarget.value)} aria-describedby="privacy-unsafe-html-help"><For each={['ask','strict','sanitized']}>{value=><option value={value}>{t(`privacy-unsafe-html-${value}`)}</option>}</For></select><p id="privacy-unsafe-html-help" class={css.meta}>{t('privacy-unsafe-html-help')}</p><LockNote locked={unsafeHtml.locked()}/><Show when={unsafeHtml.error()}><p role="alert">{unsafeHtml.error()}</p></Show><button type="button" class="btn" disabled={!unsafeHtml.entry()||unsafeHtml.locked()||unsafeHtml.saving()} onClick={()=>void unsafeHtml.reset()}>{t('privacy-inherit')}</button></div>
    <TrustedLinkDomains />
    <div class={css.field}><label><input type="checkbox" checked={trackerChoiceAllowed()&&trackerOverride.value()} disabled={!trackerOverride.entry()||trackerOverride.locked()||trackerOverride.saving()||!trackerChoiceAllowed()} aria-describedby="privacy-tracker-choice-help" onChange={event=>{if(trackerChoiceAllowed())void trackerOverride.set(event.currentTarget.checked);}}/> {t('privacy-tracker-choice')}</label><p id="privacy-tracker-choice-help" class={css.meta}>{t('privacy-tracker-choice-help')}</p><LockNote locked={trackerOverride.locked()||!trackerChoiceAllowed()}/><Show when={trackerOverride.error()}><p role="alert">{trackerOverride.error()}</p></Show><button type="button" class="btn" disabled={!trackerOverride.entry()||trackerOverride.locked()||trackerOverride.saving()||!trackerChoiceAllowed()} onClick={()=>{if(trackerChoiceAllowed())void trackerOverride.reset();}}>{t('privacy-inherit')}</button></div>
    <div class={css.field}><label><input type="checkbox" checked={originalAvailable()&&originalHtml.value()} disabled={!originalHtml.entry()||originalHtml.locked()||originalHtml.saving()||!originalAvailable()} aria-describedby="privacy-original-html-help" onChange={event=>{if(originalAvailable())void originalHtml.set(event.currentTarget.checked);}}/> {t('privacy-original-html-allow')}</label><p id="privacy-original-html-help" class={css.meta}>{t('privacy-original-html-help')}</p><LockNote locked={originalHtml.locked()||!originalAvailable()}/><Show when={originalHtml.error()}><p role="alert">{originalHtml.error()}</p></Show><button type="button" class="btn" disabled={!originalHtml.entry()||originalHtml.locked()||originalHtml.saving()||!originalAvailable()} onClick={()=>{if(originalAvailable())void originalHtml.reset();}}>{t('privacy-inherit')}</button></div>
  </section>;
}
