import {SettingsSubtabs} from './SettingsSubtabs.tsx';
import {ReaderActionPreferences,READER_ACTIONS_CARD} from './ReaderActionPreferences.tsx';
import { MessageBadges, MESSAGE_BADGES_CARD } from './MessageBadges.tsx';
import { For, Show, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { useSetting } from './useSetting.ts';
import { SettingRow } from './layout/SettingRow.tsx';
import { SettingsCard } from './layout/SettingsCard.tsx';
import type { CardSpec } from './layout/rows.tsx';
import * as css from './styles.css.ts';
import { AttachmentsBarSettings, ATTACHMENTS_BAR_CARD_SPEC } from './AttachmentsBarSettings.tsx';
import {ToggleRow} from './layout/rows.tsx';
import { snapZoom, ZOOM_STEPS } from '../../components/readerZoom.ts';
import {AttachmentPreviewPreferences,ATTACHMENT_PREVIEW_CARD} from './AttachmentPreviewPreferences.tsx';

type Field = { key: string; fallback: string | number; choices?: string[]; steps?: readonly number[]; min?: number; max?: number; step?: number };
const fields: Record<string, Field> = {
  attachmentsDensity: { key: 'attachmentsDensity', fallback: 'compact', choices: ['compact', 'comfortable'] },
  actionStyle: { key: 'actionStyle', fallback: 'icons', choices: ['icons', 'labels'] },
  colorMode: { key: 'colorMode', fallback: 'auto', choices: ['auto', 'original', 'adaptive', 'invert'] },
  fontSize: { key: 'fontSize', fallback: 14, min: 12, max: 24, step: 1 },
  lineHeight: { key: 'lineHeight', fallback: 1.6, min: 1.2, max: 2.2, step: 0.1 },
  contentWidth: { key: 'contentWidth', fallback: 'full', choices: ['comfortable', 'wide', 'full'] },
  loading: { key: 'loading', fallback: 'skeleton', choices: ['skeleton', 'indicator', 'none'] },
  loadingDelayMs: { key: 'loadingDelayMs', fallback: 120, min: 0, max: 1000, step: 20 },
  // t59: the reader's zoom buttons write the same key; this sets it directly.
  zoom: { key: 'zoom', fallback: 100, steps: ZOOM_STEPS },
};
// Card -> ordered fields. Every key above appears in exactly one card.
export const READER_CARDS: { id: string; title: string; help: string; keys: string[]; leading?: 'attachmentsCollapsed'|'autoHideHeader' }[] = [
  { id: 'reader-card-text', title: 'reader-card-text', help: 'reader-card-text-help', keys: ['colorMode', 'fontSize', 'lineHeight', 'contentWidth', 'zoom'] },
  { id: 'reader-card-actions', title: 'reader-card-actions', help: 'reader-card-actions-help', keys: ['actionStyle', 'attachmentsDensity'], leading: 'attachmentsCollapsed' },
  { id: 'reader-card-loading', title: 'reader-card-loading', help: 'reader-card-loading-help', keys: ['loading', 'loadingDelayMs'] },
  { id:'reader-card-header',title:'reader-card-header',help:'reader-autoHideHeader-help',keys:[],leading:'autoHideHeader' },
];

function FieldRow(props: { field: Field }): JSX.Element {
  const field = props.field;
  const setting = useSetting<string | number>(`reader.${field.key}`, field.fallback, (v): v is string | number => field.choices ? typeof v === 'string' && field.choices.includes(v) : typeof v === 'number' && Number.isFinite(v));
  const disabled = () => !setting.entry() || setting.locked() || setting.saving();
  const id = `reader-${field.key}`;
  return <SettingRow controlId={id} label={t(`reader-${field.key}`)} help={t(`reader-${field.key}-help`)} handle={setting}>{describedBy =>
    <Show when={field.steps === undefined} fallback={<select id={id} class={css.select} value={String(snapZoom(setting.value()))} disabled={disabled()} onChange={e => void setting.set(Number(e.currentTarget.value))} aria-describedby={describedBy()}><For each={field.steps}>{step => <option value={String(step)}>{t('reader-zoom-level', { level: step })}</option>}</For></select>}>
      <Show when={field.choices} fallback={<input id={id} class={css.input} type="number" min={field.min} max={field.max} step={field.step} value={setting.value()} disabled={disabled()} onChange={e => { if (e.currentTarget.checkValidity()) void setting.set(e.currentTarget.valueAsNumber); }} aria-describedby={describedBy()} />}>
        {choices => <select id={id} class={css.select} value={setting.value()} disabled={disabled()} onChange={e => void setting.set(e.currentTarget.value)} aria-describedby={describedBy()}><For each={choices()}>{choice => <option value={choice}>{t(`reader-option-${choice}`)}</option>}</For></select>}
      </Show>
    </Show>
  }</SettingRow>;
}

function AttachmentsCollapsedRow(): JSX.Element {
  const attachmentsCollapsed = useSetting('reader.attachmentsCollapsed', true);
  return <SettingRow controlId="reader-attachmentsCollapsed" label={t('reader-attachmentsCollapsed')} help={t('reader-attachmentsCollapsed-help')} handle={attachmentsCollapsed}>{describedBy =>
    <input id="reader-attachmentsCollapsed" type="checkbox" checked={attachmentsCollapsed.value()} aria-describedby={describedBy()} disabled={!attachmentsCollapsed.entry() || attachmentsCollapsed.locked() || attachmentsCollapsed.saving()} onChange={event => void attachmentsCollapsed.set(event.currentTarget.checked)} />
  }</SettingRow>;
}

export function ReaderAppearanceSettings(props:{senders?:JSX.Element;cache?:JSX.Element}): JSX.Element {
  const autoHideHeader=useSetting<boolean>('reader.autoHideHeader',false);
  onMount(() => { void loadCatalog('reader'); void loadCatalog('settings-policy'); });
  return <>
    <header><h3 class={css.heading}>{t('reader-settings-title')}</h3><p class={css.meta}>{t('reader-settings-help')}</p></header>
    <SettingsSubtabs section="reader" label="reader-settings-title" tabs={[
      {id:'appearance',label:'reader-card-text',cards:['reader-card-text','reader-card-header','reader-card-loading'],content:<>{cardsFor(READER_CARDS.filter(card=>card.id!=='reader-card-actions'))}</>},
      {id:'actions',label:'reader-actions-title',cards:['reader-card-actions',READER_ACTIONS_CARD.id],content:<>{cardsFor(READER_CARDS.filter(card=>card.id==='reader-card-actions'))}<ReaderActionPreferences/></>},
      {id:'attachments',label:ATTACHMENTS_BAR_CARD_SPEC.title,cards:[ATTACHMENTS_BAR_CARD_SPEC.id,ATTACHMENT_PREVIEW_CARD.id],content:<><AttachmentsBarSettings/><AttachmentPreviewPreferences/></>},
      {id:'badges',label:MESSAGE_BADGES_CARD.title,cards:[MESSAGE_BADGES_CARD.id],content:<MessageBadges/>},
      {id:'senders',label:'avatars-title',cards:['avatars-card','save-contacts-card'],content:props.senders},
      {id:'cache',label:'cache-card-store',cards:['cache-card-store','cache-card-prefetch'],content:props.cache},
    ]}/>
  </>;
  function cardsFor(cards:typeof READER_CARDS):JSX.Element{return <For each={cards}>{card => <SettingsCard id={card.id} title={t(card.title)} description={t(card.help)}>
      <Show when={card.leading}><Show when={card.leading==='autoHideHeader'} fallback={<AttachmentsCollapsedRow/>}><ToggleRow id="reader-autoHideHeader" label={t('reader-autoHideHeader')} help={t('reader-autoHideHeader-help')} handle={autoHideHeader}/></Show></Show>
      <For each={card.keys}>{key => <FieldRow field={fields[key]!} />}</For>
    </SettingsCard>}</For>;}
}

export const READER_CARD_SPECS: readonly CardSpec[] = [...READER_CARDS.map(card => ({
  id: card.id, section: 'reader', title: card.title, help: card.help,
  rows: [...(card.leading ? [card.leading] : []), ...card.keys].map(key => ({ id: `reader-${key}`, label: `reader-${key}`, help: `reader-${key}-help` })),
})), READER_ACTIONS_CARD, ATTACHMENTS_BAR_CARD_SPEC, ATTACHMENT_PREVIEW_CARD, MESSAGE_BADGES_CARD];
