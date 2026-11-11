import {formatDate as regionalDate} from '../../../i18n/format.ts';
// "Message options...": the optional headers of one message (Sensitivity,
// Reply-By, Expires, Supersedes, legacy Return-Receipt-To) and an Advanced panel
// (Auto-Submitted, threading headers, custom headers). Every part is behind its
// `options.*` feature flag and the administrator's `compose.headers.*` gates; the
// server checks the same gates again. Read and delivery receipts and importance
// have their own controls in the Delivery tab; this dialog does not repeat them.
import { For, Show, createSignal, type JSX } from 'solid-js';
import { RibbonGroup, RibbonItem } from '../../bars/index.ts';
import { RibbonPopover } from '../ribbonParts.tsx';
import { t, loadCatalog } from '../../../i18n/index.ts';
import { useComposeFeature } from '../composeFeatures.ts';
import type { SentChoice } from '../../../api/sentMessages.ts';
import {
  MAX_CUSTOM_HEADERS, isMessageId, isMailbox, validateCustomHeader,
  type AutoSubmitted, type HeaderGates, type MessageHeaderOptions, type Sensitivity,
} from './messageHeaders.ts';
import './compose-layout.css';

export interface MessageOptionsProps {
  options: () => MessageHeaderOptions;
  update: (change: (current: MessageHeaderOptions) => MessageHeaderOptions) => void;
  gates: () => HeaderGates;
  /** The threading the message carries now (a reply or forward). */
  threading: () => { inReplyTo: string[]; references: string[] };
  /** Remove In-Reply-To and References ("Start a new thread"). */
  startNewThread: () => void;
  fromAddress: () => string;
  /** Recipients outside the organisation, for the confidential note. */
  externalCount: () => number;
  /** Search the user's Sent folder (the picker for "Replaces an earlier message"). */
  sentMessages?: (query: string) => Promise<SentChoice[]>;
  /** Whether the sent copy is kept, which a reminder needs. */
  remindAvailable?: () => boolean;
}

const SENSITIVITIES: readonly Sensitivity[] = ['normal', 'personal', 'private', 'company-confidential'];
const AUTO: readonly AutoSubmitted[] = ['no', 'auto-generated', 'auto-replied'];

/** `datetime-local` text to a Date in the user's time zone, or null. */
export function fromLocalInput(v: string): Date | null {
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(v)) return null;
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? null : d;
}
export function toLocalInput(d: Date | null | undefined): string {
  if (!d) return '';
  const p = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

/** The quick choices for a Reply-By or Expires time, relative to `now`. */
export function datePresets(now: Date): { id: string; at: Date }[] {
  const at = (days: number, hour: number): Date => { const d = new Date(now); d.setDate(d.getDate() + days); d.setHours(hour, 0, 0, 0); return d; };
  const nextMonday = at(((8 - now.getDay()) % 7) || 7, 9);
  const today = at(0, 17);
  return [
    ...(today.getTime() > now.getTime() ? [{ id: 'today', at: today }] : []),
    { id: 'tomorrow', at: at(1, 9) },
    { id: 'in3days', at: at(3, 9) },
    { id: 'nextWeek', at: nextMonday },
  ];
}

function DateRow(props: { id: string; label: string; help: string; value: () => Date | null | undefined; onChange: (d: Date | null) => void; testId: string }): JSX.Element {
  return <div class="mo-row">
    <label for={props.id}>{props.label}</label>
    <div class="mo-controls">
      <input id={props.id} type="datetime-local" value={toLocalInput(props.value())} data-testid={props.testId} aria-describedby={`${props.id}-help`} onInput={(e) => props.onChange(fromLocalInput(e.currentTarget.value))} />
      <select aria-label={t('mo-presets')} value="" data-testid={`${props.testId}-preset`} onChange={(e) => { const p = datePresets(new Date()).find((x) => x.id === e.currentTarget.value); if (p) props.onChange(p.at); e.currentTarget.value = ''; }}>
        <option value="">{t('mo-presets')}</option>
        <For each={datePresets(new Date())}>{(p) => <option value={p.id}>{t(`mo-preset-${p.id}`)}</option>}</For>
      </select>
      <button type="button" class="compose-rt__btn" disabled={!props.value()} onClick={() => props.onChange(null)}>{t('mo-clear')}</button>
    </div>
    <small id={`${props.id}-help`}>{props.help}</small>
  </div>;
}

export function MessageOptions(props: MessageOptionsProps): JSX.Element {
  void loadCatalog('compose-layout');
  const fSens = useComposeFeature('options.sensitivity');
  const fReplyBy = useComposeFeature('options.replyBy');
  const fExpires = useComposeFeature('options.expires');
  const fSupersedes = useComposeFeature('options.supersedes');
  const fReturn = useComposeFeature('options.returnReceipt');
  const fAuto = useComposeFeature('options.autoSubmitted');
  const fThread = useComposeFeature('options.inReplyTo');
  const fCustom = useComposeFeature('options.customHeaders');
  const [draftName, setDraftName] = createSignal('');
  const [draftValue, setDraftValue] = createSignal('');
  const [customError, setCustomError] = createSignal('');
  const [picking, setPicking] = createSignal(false);
  const [pickQuery, setPickQuery] = createSignal('');
  const [choices, setChoices] = createSignal<SentChoice[] | null>(null);
  const [pickFailed, setPickFailed] = createSignal(false);
  const search = async (query: string): Promise<void> => {
    setPickQuery(query);
    try { setChoices(await props.sentMessages!(query)); setPickFailed(false); } catch { setChoices([]); setPickFailed(true); }
  };
  const o = props.options;
  const set = (patch: MessageHeaderOptions): void => props.update((cur) => ({ ...cur, ...patch }));
  const any = (): boolean => fSens() || fReplyBy() || fExpires() || fSupersedes() || (fReturn() && props.gates().allowLegacyReturnReceipt) || (fAuto() && props.gates().allowAutoSubmitted) || fThread() || (fCustom() && props.gates().allowCustom);
  const supersedesBad = (): boolean => (o().supersedes ?? '') !== '' && !isMessageId(o().supersedes!);
  const addCustom = (): void => {
    const h = { name: draftName().trim(), value: draftValue().trim() };
    const err = validateCustomHeader(h);
    if (err) { setCustomError(t(`mo-custom-error-${err}`)); return; }
    if ((o().custom ?? []).length >= MAX_CUSTOM_HEADERS) { setCustomError(t('mo-custom-error-tooMany')); return; }
    if ((o().custom ?? []).some((c) => c.name.toLowerCase() === h.name.toLowerCase())) { setCustomError(t('mo-custom-error-duplicate')); return; }
    setCustomError('');
    set({ custom: [...(o().custom ?? []), h] });
    setDraftName(''); setDraftValue('');
  };

  return <Show when={any()}>
    <RibbonGroup label={t('mo-group')} priority={10} icon="preferences" testId="compose-delivery-message-options">
      <RibbonItem>
        <RibbonPopover label={t('mo-open')} icon="preferences" testId="compose-message-options" buttonTestId="compose-message-options-button">
          <div class="mo" role="group" aria-label={t('mo-group')}>
            <Show when={fSens()}>
              <div class="mo-row">
                <label for="mo-sensitivity">{t('mo-sensitivity')}</label>
                <select id="mo-sensitivity" value={o().sensitivity ?? 'normal'} data-testid="mo-sensitivity" aria-describedby="mo-sensitivity-help" onChange={(e) => set({ sensitivity: e.currentTarget.value as Sensitivity })}>
                  <For each={SENSITIVITIES}>{(s) => <option value={s}>{t(`mo-sensitivity-${s}`)}</option>}</For>
                </select>
                <small id="mo-sensitivity-help">{t('mo-sensitivity-help')}</small>
                <Show when={o().sensitivity === 'company-confidential' && props.externalCount() > 0}><small role="alert" data-testid="mo-confidential-external">{t('mo-confidential-external', { count: props.externalCount() })}</small></Show>
              </div>
            </Show>
            <Show when={fReplyBy()}><DateRow id="mo-reply-by" testId="mo-reply-by" label={t('mo-reply-by')} help={t('mo-reply-by-help')} value={() => o().replyBy} onChange={(d) => set(d ? { replyBy: d } : { replyBy: null, remindIfNoReply: false })} />
              <div class="mo-row">
                <label class="mo-check"><input type="checkbox" data-testid="mo-remind" checked={o().remindIfNoReply === true} disabled={!o().replyBy || props.remindAvailable?.() === false} aria-describedby="mo-remind-help" onChange={(e) => set({ remindIfNoReply: e.currentTarget.checked })} /> {t('mo-remind')}</label>
                <small id="mo-remind-help">{props.remindAvailable?.() === false ? t('mo-remind-nocopy') : t('mo-remind-help')}</small>
              </div></Show>
            <Show when={fExpires()}><DateRow id="mo-expires" testId="mo-expires" label={t('mo-expires')} help={t('mo-expires-help')} value={() => o().expires} onChange={(d) => set({ expires: d })} /></Show>
            <Show when={fSupersedes()}>
              <div class="mo-row">
                <label for="mo-supersedes">{t('mo-supersedes')}</label>
                <input id="mo-supersedes" type="text" placeholder="<id@example.com>" value={o().supersedes ?? ''} data-testid="mo-supersedes" aria-invalid={supersedesBad()} aria-describedby="mo-supersedes-help" onInput={(e) => set({ supersedes: e.currentTarget.value.trim() })} />
                <small id="mo-supersedes-help">{supersedesBad() ? t('mo-supersedes-invalid') : t('mo-supersedes-help')}</small>
                <Show when={props.sentMessages}>
                  <button type="button" class="compose-rt__btn" data-testid="mo-supersedes-pick" aria-expanded={picking()} onClick={() => { const next = !picking(); setPicking(next); if (next && choices() === null) void search(''); }}>{t('mo-supersedes-pick')}</button>
                  <Show when={picking()}>
                    <div class="mo-picker" data-testid="mo-supersedes-picker">
                      <input type="search" value={pickQuery()} placeholder={t('mo-supersedes-search')} aria-label={t('mo-supersedes-search')} data-testid="mo-supersedes-search" onInput={(e) => void search(e.currentTarget.value)} />
                      <Show when={!pickFailed()} fallback={<small role="alert">{t('mo-supersedes-failed')}</small>}>
                        <Show when={(choices() ?? []).length > 0} fallback={<small>{choices() === null ? t('mo-supersedes-loading') : t('mo-supersedes-none')}</small>}>
                          <ul role="listbox" aria-label={t('mo-supersedes-pick')}>
                            <For each={choices() ?? []}>{(c) => <li role="option" aria-selected={o().supersedes === c.messageId}><button type="button" class="compose-rt__btn" data-testid="mo-supersedes-choice" onClick={() => { set({ supersedes: c.messageId }); setPicking(false); }}><span>{c.subject || t('mo-supersedes-no-subject')}</span> <small>{c.date ? regionalDate(c.date) : ''}</small></button></li>}</For>
                          </ul>
                        </Show>
                      </Show>
                    </div>
                  </Show>
                </Show>
              </div>
            </Show>
            <Show when={fReturn() && props.gates().allowLegacyReturnReceipt}>
              <div class="mo-row">
                <label><input type="checkbox" data-testid="mo-return-receipt" checked={o().returnReceiptTo !== undefined} onChange={(e) => set({ returnReceiptTo: e.currentTarget.checked ? props.fromAddress() : undefined })} />{t('mo-return-receipt')}</label>
                <small>{t('mo-return-receipt-help')}</small>
                <Show when={o().returnReceiptTo !== undefined && !isMailbox(o().returnReceiptTo!)}><small role="alert">{t('mo-return-receipt-invalid')}</small></Show>
              </div>
            </Show>
            <Show when={(fAuto() && props.gates().allowAutoSubmitted) || fThread() || (fCustom() && props.gates().allowCustom)}>
              <details class="mo-advanced" data-testid="mo-advanced">
                <summary>{t('mo-advanced')}</summary>
                <Show when={fAuto() && props.gates().allowAutoSubmitted}>
                  <div class="mo-row">
                    <label for="mo-auto">{t('mo-auto-submitted')}</label>
                    <select id="mo-auto" value={o().autoSubmitted ?? ''} data-testid="mo-auto" onChange={(e) => set({ autoSubmitted: (e.currentTarget.value || undefined) as AutoSubmitted | undefined })}>
                      <option value="">{t('mo-auto-absent')}</option>
                      <For each={AUTO}>{(a) => <option value={a}>{a}</option>}</For>
                    </select>
                    <small>{t('mo-auto-help')}</small>
                  </div>
                </Show>
                <Show when={fThread()}>
                  <div class="mo-row" data-testid="mo-threading">
                    <span class="mo-label">{t('mo-threading')}</span>
                    <Show when={props.threading().inReplyTo.length > 0} fallback={<small>{t('mo-threading-none')}</small>}>
                      <code data-testid="mo-in-reply-to">In-Reply-To: {props.threading().inReplyTo.map((i) => `<${i}>`).join(' ')}</code>
                      <code data-testid="mo-references">References: {props.threading().references.map((i) => `<${i}>`).join(' ')}</code>
                      <button type="button" class="compose-rt__btn" data-testid="mo-new-thread" disabled={!props.gates().allowThreadingEdit} title={props.gates().allowThreadingEdit ? undefined : t('mo-threading-locked')} onClick={() => props.startNewThread()}>{t('mo-new-thread')}</button>
                      <Show when={!props.gates().allowThreadingEdit}><small>{t('mo-threading-locked')}</small></Show>
                    </Show>
                  </div>
                </Show>
                <Show when={fCustom() && props.gates().allowCustom}>
                  <div class="mo-row" data-testid="mo-custom">
                    <span class="mo-label">{t('mo-custom')}</span>
                    <ul class="mo-custom-list">
                      <For each={o().custom ?? []}>{(h, i) => <li><code>{h.name}: {h.value}</code> <button type="button" class="compose-rt__btn" aria-label={t('mo-custom-remove', { name: h.name })} onClick={() => set({ custom: (o().custom ?? []).filter((_, j) => j !== i()) })}>{t('mo-clear')}</button></li>}</For>
                    </ul>
                    <div class="mo-controls">
                      <input aria-label={t('mo-custom-name')} placeholder="X-Ticket" value={draftName()} data-testid="mo-custom-name" onInput={(e) => setDraftName(e.currentTarget.value)} />
                      <input aria-label={t('mo-custom-value')} placeholder="42" value={draftValue()} data-testid="mo-custom-value" onInput={(e) => setDraftValue(e.currentTarget.value)} />
                      <button type="button" class="compose-rt__btn" data-testid="mo-custom-add" onClick={addCustom}>{t('mo-custom-add')}</button>
                    </div>
                    <Show when={customError()}><small role="alert" data-testid="mo-custom-error">{customError()}</small></Show>
                    <small>{t('mo-custom-help', { max: MAX_CUSTOM_HEADERS })}</small>
                  </div>
                </Show>
              </details>
            </Show>
          </div>
        </RibbonPopover>
      </RibbonItem>
    </RibbonGroup>
  </Show>;
}
