// Composer side-features (W9 Drafts drawer, W10 recall, W11 send-option toggles,
// W12 signature picker). Each is a small, prop-first component so it is unit-
// testable in isolation and Compose stays the single wiring point. No component
// here reaches into a store slice directly — Compose passes the data + callbacks.

import { For, Show, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n/index.ts';
import * as a11y from '../mailA11y.css.ts';
import type { EmailSubmission } from '../../api/jmap-types.ts';
import type { StoredDraft } from './drafts-store.ts';
import './compose-extras.css';

// ── W12: signature picker ────────────────────────────────────────────────────

/** A selectable signature. `html` is preferred for the rich body; `text` backs
 *  the plain-text path. Sourced from identities today; a signatures CRUD backend
 *  (e15) can supply the same shape later. */
export interface ComposeSignature {
  id: string;
  name: string;
  text: string;
  html: string | null;
}

export function SignaturePicker(props: {
  signatures: () => ComposeSignature[];
  onInsert: (sig: ComposeSignature) => void;
}): JSX.Element {
  return (
    <Show when={props.signatures().length > 0}>
      <label class="field compose-extra__signature" data-testid="compose-signature">
        <span>{t('mail-compose-signature')}</span>
        <select
          aria-label={t('mail-compose-signature')}
          onChange={(e) => {
            const sig = props.signatures().find((s) => s.id === e.currentTarget.value);
            if (sig !== undefined) props.onInsert(sig);
            e.currentTarget.value = '';
          }}
        >
          <option value="">{t('mail-compose-signature-none')}</option>
          <For each={props.signatures()}>
            {(sig) => <option value={sig.id}>{isolate(sig.name)}</option>}
          </For>
        </select>
      </label>
    </Show>
  );
}

// ── W11: send-option toggles (read receipt + open-tracking pixel) ────────────

export interface SendOptionsState {
  /** Ask the recipient's client for a read receipt (MDN). */
  requestReceipt: boolean;
  /** Embed a 1×1 open-tracking pixel in the HTML body. Off by default. */
  trackingPixel: boolean;
}

export const DEFAULT_SEND_OPTIONS: SendOptionsState = {
  requestReceipt: false,
  trackingPixel: false,
};

export function SendOptions(props: {
  state: () => SendOptionsState;
  onChange: (next: SendOptionsState) => void;
}): JSX.Element {
  return (
    <fieldset class="compose-extra__options" data-testid="compose-send-options">
      <legend>{t('mail-compose-options')}</legend>
      <label class="compose-extra__option">
        <input
          type="checkbox"
          data-testid="opt-receipt"
          checked={props.state().requestReceipt}
          onChange={(e) => props.onChange({ ...props.state(), requestReceipt: e.currentTarget.checked })}
        />
        <span>{t('mail-compose-receipt')}</span>
      </label>
      <label class="compose-extra__option">
        <input
          type="checkbox"
          data-testid="opt-tracking"
          checked={props.state().trackingPixel}
          onChange={(e) => props.onChange({ ...props.state(), trackingPixel: e.currentTarget.checked })}
        />
        <span>{t('mail-compose-tracking')}</span>
      </label>
      <p class="compose-extra__hint">{t('mail-compose-tracking-hint')}</p>
    </fieldset>
  );
}

// ── W10: recall panel (stop a still-holding / scheduled submission) ──────────

export function RecallPanel(props: {
  submissions: () => EmailSubmission[];
  onRecall: (id: string) => void;
}): JSX.Element {
  return (
    <Show when={props.submissions().length > 0}>
      <section class="compose-extra__recall" data-testid="compose-recall" aria-label={t('mail-compose-recall')}>
        <h3 class="compose-extra__recall-title">{t('mail-compose-recall')}</h3>
        <ul class="compose-extra__recall-list">
          <For each={props.submissions()}>
            {(sub) => (
              <li class="compose-extra__recall-row">
                <span>
                  {sub.sendAt !== null
                    ? t('mail-compose-recall-scheduled', { when: formatWhen(sub.sendAt) })
                    : t('mail-compose-recall-holding')}
                </span>
                <button
                  type="button"
                  class={`btn btn--ghost ${a11y.focusable}`}
                  data-testid={`recall-${sub.id}`}
                  onClick={() => props.onRecall(sub.id)}
                >
                  {t('mail-compose-recall-action')}
                </button>
              </li>
            )}
          </For>
        </ul>
      </section>
    </Show>
  );
}

function formatWhen(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}

// ── W9: universal Drafts drawer ──────────────────────────────────────────────

export function DraftsDrawer(props: {
  open: boolean;
  drafts: () => StoredDraft[];
  onResume: (draft: StoredDraft) => void;
  onDelete: (id: string) => void;
  onClose: () => void;
}): JSX.Element {
  return (
    <Show when={props.open}>
      <section class="compose-extra__drafts" data-testid="compose-drafts" aria-label={t('mail-compose-drafts')}>
        <header class="compose-extra__drafts-head">
          <h3>{t('mail-compose-drafts')}</h3>
          <button
            type="button"
            class={`btn btn--ghost ${a11y.iconButton}`}
            aria-label={t('mail-compose-close')}
            onClick={() => props.onClose()}
          >
            ✕
          </button>
        </header>
        <Show
          when={props.drafts().length > 0}
          fallback={<p class="compose-extra__drafts-empty">{t('mail-compose-drafts-empty')}</p>}
        >
          <ul class="compose-extra__drafts-list">
            <For each={props.drafts()}>
              {(draft) => (
                <li class="compose-extra__drafts-row">
                  <button
                    type="button"
                    class={`compose-extra__drafts-open ${a11y.focusable}`}
                    data-testid={`draft-resume-${draft.id}`}
                    onClick={() => props.onResume(draft)}
                  >
                    <span class="compose-extra__drafts-subject">
                      {draft.subject.trim() !== '' ? isolate(draft.subject) : t('mail-no-subject')}
                    </span>
                    <span class="compose-extra__drafts-preview">
                      {draft.to.trim() !== '' ? isolate(draft.to) : t('mail-compose-drafts-no-recipient')}
                    </span>
                  </button>
                  <button
                    type="button"
                    class={`btn btn--ghost ${a11y.iconButton}`}
                    aria-label={t('mail-compose-drafts-delete')}
                    data-testid={`draft-delete-${draft.id}`}
                    onClick={() => props.onDelete(draft.id)}
                  >
                    🗑
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </section>
    </Show>
  );
}
