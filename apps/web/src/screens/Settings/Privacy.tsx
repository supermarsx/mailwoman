import { createResource, For, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { remotePolicy, type RemoteMode } from '../../api/remote-content.ts';
import { isNativeAuth } from '../../api/serverAddress.ts';
import { TrustedLinkDomains } from './TrustedLinkDomains.tsx';
import {featureOn,featuresLoaded} from '../../state/features.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
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
  const imageRetryAttempts=useSetting('privacy.imageRetryAttempts',3);
  const imageRetryDelay=useSetting('privacy.imageRetryDelayMs',5000);
  const [policy] = createResource(async () => { try { return await remotePolicy(); } catch { return null; } });
  const canDirect = (): boolean => !isNativeAuth() && policy.state === 'ready' && policy.latest?.allowDirect === true;
  const toggles = (): { setting: typeof trackers; id: string; label: string; help?: string }[] => [
    { setting: trackers, id: 'privacy-trackers', label: 'privacy-trackers' }, { setting: links, id: 'privacy-clean-links', label: 'privacy-clean-links' },
    { setting: imageDownloads, id: 'privacy-image-downloads', label: 'privacy-image-downloads' }, { setting: imageToast, id: 'privacy-image-toast', label: 'privacy-image-toast' },
    ...(policy.state === 'ready' && policy.latest?.inlineParts === true ? [{ setting: stripMailerHeaders, id: 'privacy-strip-mailer-headers', label: 'privacy-strip-mailer-headers', help: 'privacy-strip-mailer-help' }] : []),
  ];
  const free = (h: { entry(): unknown; locked(): boolean; saving(): boolean }): boolean => !h.entry() || h.locked() || h.saving();
  return <>
    <SettingsCard id="privacy-card-remote" title={t('privacy-title')} description={t('privacy-help')}>
      <SettingRow controlId="privacy-mode" label={t('privacy-remote-mode')} handle={mode} resetLabel={t('privacy-inherit')} {...(!canDirect() ? { help: t(isNativeAuth() ? 'privacy-direct-native' : 'privacy-direct-unavailable') } : {})}>{describedBy =>
        <select id="privacy-mode" class={css.select} aria-describedby={describedBy()} value={mode.value() === 'direct' && !canDirect() ? 'ask' : mode.value()} disabled={free(mode)} onChange={e => void mode.set(e.currentTarget.value as RemoteMode)}><For each={['block','ask','proxy','direct'] as const}>{value => <option value={value} disabled={value === 'direct' && !canDirect()}>{t(`privacy-mode-${value}`)}</option>}</For></select>
      }</SettingRow>
      <For each={toggles()}>{item => <SettingRow controlId={item.id} label={t(item.label)} {...(item.help ? { help: t(item.help) } : {})} handle={item.setting} resetLabel={t('privacy-inherit')}>{describedBy =>
        <input id={item.id} type="checkbox" checked={item.setting.value()} aria-describedby={describedBy()} disabled={free(item.setting)} onChange={e => void item.setting.set(e.currentTarget.checked)} />
      }</SettingRow>}</For>
      <SettingRow controlId="privacy-image-retries" label={t('privacy-image-retry-attempts')} help={t('privacy-image-retry-help')} handle={imageRetryAttempts} resetLabel={t('privacy-inherit')}>{describedBy =>
        <input id="privacy-image-retries" class={css.input} type="number" min="0" max="8" step="1" value={imageRetryAttempts.value()} disabled={free(imageRetryAttempts)} aria-describedby={describedBy()} onChange={event=>{const value=event.currentTarget.valueAsNumber;if(Number.isSafeInteger(value)&&value>=0&&value<=8)void imageRetryAttempts.set(value);}}/>
      }</SettingRow>
      <SettingRow controlId="privacy-image-retry-delay" label={t('privacy-image-retry-delay')} handle={imageRetryDelay} resetLabel={t('privacy-inherit')}>{describedBy =>
        <input id="privacy-image-retry-delay" class={css.input} type="number" min="0.5" max="60" step="0.5" value={imageRetryDelay.value()/1000} disabled={free(imageRetryDelay)} aria-describedby={describedBy()} onChange={event=>{const value=Math.round(event.currentTarget.valueAsNumber*1000);if(Number.isSafeInteger(value)&&value>=500&&value<=60000)void imageRetryDelay.set(value);}}/>
      }</SettingRow>
      <SettingRow controlId="privacy-unsafe-html" label={t('privacy-unsafe-html')} help={t('privacy-unsafe-html-help')} handle={unsafeHtml} resetLabel={t('privacy-inherit')}>{describedBy =>
        <select id="privacy-unsafe-html" class={css.select} value={unsafeHtml.value()} disabled={free(unsafeHtml)} onChange={event=>void unsafeHtml.set(event.currentTarget.value)} aria-describedby={describedBy()}><For each={['ask','strict','sanitized']}>{value=><option value={value}>{t(`privacy-unsafe-html-${value}`)}</option>}</For></select>
      }</SettingRow>
      <SettingRow controlId="privacy-tracker-choice" label={t('privacy-tracker-choice')} help={t('privacy-tracker-choice-help')} handle={trackerOverride} lockNote={trackerOverride.locked()||!trackerChoiceAllowed()} resetLabel={t('privacy-inherit')} resetDisabled={free(trackerOverride)||!trackerChoiceAllowed()} onReset={()=>{if(trackerChoiceAllowed())void trackerOverride.reset();}}>{describedBy =>
        <input id="privacy-tracker-choice" type="checkbox" checked={trackerChoiceAllowed()&&trackerOverride.value()} disabled={free(trackerOverride)||!trackerChoiceAllowed()} aria-describedby={describedBy()} onChange={event=>{if(trackerChoiceAllowed())void trackerOverride.set(event.currentTarget.checked);}}/>
      }</SettingRow>
      <SettingRow controlId="privacy-original-html" label={t('privacy-original-html-allow')} help={t('privacy-original-html-help')} handle={originalHtml} lockNote={originalHtml.locked()||!originalAvailable()} resetLabel={t('privacy-inherit')} resetDisabled={free(originalHtml)||!originalAvailable()} onReset={()=>{if(originalAvailable())void originalHtml.reset();}}>{describedBy =>
        <input id="privacy-original-html" type="checkbox" checked={originalAvailable()&&originalHtml.value()} disabled={free(originalHtml)||!originalAvailable()} aria-describedby={describedBy()} onChange={event=>{if(originalAvailable())void originalHtml.set(event.currentTarget.checked);}}/>
      }</SettingRow>
    </SettingsCard>
    <TrustedLinkDomains />
  </>;
}
