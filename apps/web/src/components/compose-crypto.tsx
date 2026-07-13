// Compose crypto + DLP subcomponents (plan §2.5, §3 e4). STANDALONE, self-
// contained pieces that take props + callbacks — e8 mounts them into `Compose.tsx`
// during its mount/wire pass (this file never imports or edits Compose, so the two
// executors don't collide). Three concerns, each independently testable:
//
//   • <EncryptSignToggles>  — per-message encrypt / sign switches + the
//                             encrypted-draft & protected-subject affordances.
//   • <CapabilityBanner>    — the live "this message will be E2EE / TLS / mixed"
//                             banner, computed from per-recipient `CryptoKey/lookup`.
//   • <DlpWarnings>         — inline warn / require-encryption / block verdicts from
//                             `Dlp/scan`; a `block` shows the rule message + gates send.
//
// <ComposeCrypto> wires all three: it probes recipient keys + DLP through the
// `lookupKeys` / `scanDlp` callbacks (mock in tests, `crypto-jmap.ts` factories in
// e8), calls the crypto-worker STUB to encrypt the draft when encryption is enabled
// (real WASM is e8), and reports {encrypt, sign, capability, canSend, …} up via
// `onChange` so the host Compose can encrypt-on-send + honor the DLP gate.

import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type JSX,
} from 'solid-js';
import type { DlpAction, DlpVerdict } from '../api/crypto-types.ts';
import type { EncryptResult } from '../contracts/crypto.ts';
import { getCryptoWorker, type CryptoWorkerApi } from '../crypto/index.ts';
import {
  chooseRecipientKey,
  computeCapability,
  normalizeRecipients,
  type RecipientCapability,
  type TransportCapability,
} from './compose/capability.ts';
import type { DlpAttachmentMeta, DlpScanFn, KeyLookupFn } from './compose/crypto-jmap.ts';
import './compose/compose-crypto.css';

export type { TransportCapability, RecipientCapability } from './compose/capability.ts';
export type { DlpAttachmentMeta, DlpScanFn, KeyLookupFn } from './compose/crypto-jmap.ts';

// ── Presentational: the E2EE / TLS / mixed banner ────────────────────────────

const BANNER_COPY: Record<TransportCapability, { label: string; detail: string }> = {
  e2ee: {
    label: 'End-to-end encrypted',
    detail: 'Every recipient has a key — the message body is encrypted on this device.',
  },
  tls: {
    label: 'Transport encryption (TLS)',
    detail: 'No recipient encryption keys were found; delivery is protected in transit only.',
  },
  mixed: {
    label: 'Mixed protection',
    detail: 'Some recipients can receive end-to-end encryption; others get TLS only.',
  },
};

export function CapabilityBanner(props: {
  capability: TransportCapability;
  recipients: RecipientCapability[];
  loading?: boolean;
}): JSX.Element {
  const copy = (): { label: string; detail: string } => BANNER_COPY[props.capability];
  const uncovered = (): RecipientCapability[] => props.recipients.filter((r) => !r.encryptable);
  return (
    <div
      class="cmpcrypto-banner"
      classList={{ [`cmpcrypto-banner--${props.capability}`]: true }}
      data-testid="compose-crypto-banner"
      data-capability={props.capability}
      role="status"
      aria-live="polite"
    >
      <span class="cmpcrypto-banner__badge" aria-hidden="true">
        {props.capability === 'e2ee' ? '🔒' : props.capability === 'mixed' ? '⚠️' : '🔓'}
      </span>
      <span class="cmpcrypto-banner__text">
        <strong class="cmpcrypto-banner__label">
          {copy().label}
          <Show when={props.loading}>
            <span class="cmpcrypto-banner__spin" aria-label="Checking recipient keys">
              {' '}
              · checking…
            </span>
          </Show>
        </strong>
        <span class="cmpcrypto-banner__detail">{copy().detail}</span>
        <Show when={props.capability === 'mixed' && uncovered().length > 0}>
          <span class="cmpcrypto-banner__uncovered">
            TLS only for:{' '}
            <For each={uncovered()}>
              {(r, i) => (
                <>
                  <Show when={i() > 0}>, </Show>
                  <span class="cmpcrypto-banner__addr">{r.address}</span>
                </>
              )}
            </For>
          </span>
        </Show>
      </span>
    </div>
  );
}

// ── Presentational: encrypt / sign toggles ───────────────────────────────────

export function EncryptSignToggles(props: {
  encrypt: boolean;
  sign: boolean;
  protectSubject: boolean;
  /** Disable the encrypt switch (no recipient can receive E2EE). */
  encryptDisabled?: boolean;
  /** Why the encrypt switch is disabled (surfaced to AT + as a title). */
  encryptDisabledReason?: string;
  /** True once the crypto worker has produced an encrypted draft. */
  drafted?: boolean;
  onEncryptChange: (next: boolean) => void;
  onSignChange: (next: boolean) => void;
  onProtectSubjectChange: (next: boolean) => void;
}): JSX.Element {
  return (
    <fieldset class="cmpcrypto-toggles">
      <legend class="cmpcrypto-toggles__legend">Message security</legend>

      <label class="cmpcrypto-toggle" classList={{ 'cmpcrypto-toggle--disabled': props.encryptDisabled }}>
        <input
          type="checkbox"
          checked={props.encrypt}
          disabled={props.encryptDisabled}
          data-testid="encrypt-toggle"
          aria-describedby="cmpcrypto-encrypt-hint"
          onChange={(e) => props.onEncryptChange(e.currentTarget.checked)}
        />
        <span class="cmpcrypto-toggle__label">Encrypt (end-to-end)</span>
      </label>
      <span id="cmpcrypto-encrypt-hint" class="cmpcrypto-toggle__hint">
        <Show
          when={!props.encryptDisabled}
          fallback={props.encryptDisabledReason ?? 'No recipient encryption key available.'}
        >
          <Show when={props.encrypt && props.drafted} fallback="Encrypt the body on this device before sending.">
            <span class="cmpcrypto-toggle__drafted" data-testid="encrypted-draft-indicator">
              Draft encrypted on this device.
            </span>
          </Show>
        </Show>
      </span>

      <label class="cmpcrypto-toggle">
        <input
          type="checkbox"
          checked={props.sign}
          data-testid="sign-toggle"
          onChange={(e) => props.onSignChange(e.currentTarget.checked)}
        />
        <span class="cmpcrypto-toggle__label">Sign (verify it's from you)</span>
      </label>

      <Show when={props.encrypt && !props.encryptDisabled}>
        <label class="cmpcrypto-toggle cmpcrypto-toggle--sub">
          <input
            type="checkbox"
            checked={props.protectSubject}
            data-testid="protect-subject-toggle"
            onChange={(e) => props.onProtectSubjectChange(e.currentTarget.checked)}
          />
          <span class="cmpcrypto-toggle__label">Also encrypt the subject line</span>
        </label>
      </Show>
    </fieldset>
  );
}

// ── Presentational: DLP pre-send warnings ────────────────────────────────────

const DLP_ACTION_COPY: Record<DlpAction, { title: string; severity: 'block' | 'warn' }> = {
  block: { title: 'Sending blocked', severity: 'block' },
  'require-encryption': { title: 'Encryption required', severity: 'warn' },
  warn: { title: 'Heads up', severity: 'warn' },
  'notify-admin': { title: 'Administrator will be notified', severity: 'warn' },
};

export function DlpWarnings(props: { verdicts: DlpVerdict[]; loading?: boolean }): JSX.Element {
  const hasBlock = (): boolean => props.verdicts.some((v) => v.action === 'block');
  return (
    <Show when={props.verdicts.length > 0}>
      <ul
        class="cmpcrypto-dlp"
        data-testid="dlp-warnings"
        aria-label="Data-loss prevention warnings"
        // A blocking verdict is assertive; advisory ones are polite.
        role={hasBlock() ? 'alert' : 'status'}
        aria-live={hasBlock() ? 'assertive' : 'polite'}
      >
        <For each={props.verdicts}>
          {(v) => {
            const copy = DLP_ACTION_COPY[v.action];
            return (
              <li
                class="cmpcrypto-dlp__item"
                classList={{ [`cmpcrypto-dlp__item--${copy.severity}`]: true }}
                data-testid={v.action === 'block' ? 'dlp-block' : 'dlp-warn'}
                data-action={v.action}
              >
                <span class="cmpcrypto-dlp__title">{copy.title}</span>
                <span class="cmpcrypto-dlp__msg">{v.ruleName}</span>
                <Show when={v.excerptRedacted.length > 0}>
                  <span class="cmpcrypto-dlp__excerpt">{v.excerptRedacted}</span>
                </Show>
                <Show when={v.matchedDetectors.length > 0}>
                  <span class="cmpcrypto-dlp__detectors">
                    Matched: {v.matchedDetectors.join(', ')}
                  </span>
                </Show>
              </li>
            );
          }}
        </For>
      </ul>
    </Show>
  );
}

// ── Container: wires lookup + scan + the crypto worker ───────────────────────

/** The crypto/DLP state `ComposeCrypto` reports up for the host Compose (e8). */
export interface ComposeCryptoState {
  encrypt: boolean;
  sign: boolean;
  protectSubject: boolean;
  capability: TransportCapability;
  /** False when a `block` DLP verdict is present — the host must not send. */
  canSend: boolean;
  /** The worker-produced encrypted draft (stub until e8), or `null`. */
  encryptedDraft: EncryptResult | null;
  verdicts: DlpVerdict[];
  recipients: RecipientCapability[];
}

export interface ComposeCryptoProps {
  /** Current recipient addresses (accessor — the banner is live as you type). */
  recipients: () => string[];
  subject?: () => string;
  bodyText?: () => string;
  attachments?: () => DlpAttachmentMeta[];
  /** Resolve a recipient's keys (mock in tests; `createJmapKeyLookup` in e8). */
  lookupKeys: KeyLookupFn;
  /** DLP dry-run (mock in tests; `createJmapDlpScan` in e8). */
  scanDlp: DlpScanFn;
  /** Override the crypto worker (defaults to the process stub, swapped in e8). */
  cryptoWorker?: CryptoWorkerApi;
  /** Report crypto/DLP state to the host Compose after every change. */
  onChange?: (state: ComposeCryptoState) => void;
}

export function ComposeCrypto(props: ComposeCryptoProps): JSX.Element {
  const worker = (): CryptoWorkerApi => props.cryptoWorker ?? getCryptoWorker();

  const [encrypt, setEncrypt] = createSignal(false);
  const [sign, setSign] = createSignal(false);
  const [protectSubject, setProtectSubject] = createSignal(false);
  const [encryptedDraft, setEncryptedDraft] = createSignal<EncryptResult | null>(null);

  const normalized = createMemo(() => normalizeRecipients(props.recipients()));
  const recipientKey = createMemo(() => normalized().join(','));

  // Probe each recipient's keys via `lookupKeys`. Keyed on the joined address
  // list so it re-runs only when the recipient set actually changes; the
  // resource takes the latest result, so out-of-order lookups can't race.
  const [capsData] = createResource(recipientKey, async (key): Promise<RecipientCapability[]> => {
    if (key.length === 0) return [];
    const addrs = key.split(',');
    return Promise.all(
      addrs.map(async (addr) => chooseRecipientKey(addr, await props.lookupKeys(addr))),
    );
  });
  const recipientCaps = (): RecipientCapability[] => capsData() ?? [];
  const capability = createMemo<TransportCapability>(() => computeCapability(recipientCaps()));
  const encryptDisabled = (): boolean => normalized().length === 0 || capability() === 'tls';

  // DLP dry-run. Only scans when there is something to scan (a body or an
  // attachment); keyed on the draft so an unchanged draft isn't re-scanned.
  const draft = createMemo(() => ({
    recipients: normalized(),
    subject: props.subject?.() ?? '',
    bodyText: props.bodyText?.() ?? '',
    attachments: props.attachments?.() ?? [],
  }));
  const dlpKey = createMemo<string | false>(() => {
    const d = draft();
    if (d.bodyText.length === 0 && d.attachments.length === 0) return false;
    return JSON.stringify(d);
  });
  const [dlpData] = createResource(dlpKey, async (): Promise<DlpVerdict[]> => props.scanDlp(draft()));
  const verdicts = (): DlpVerdict[] => dlpData() ?? [];
  const canSend = createMemo(() => !verdicts().some((v) => v.action === 'block'));

  /** Encrypt the current draft body via the worker (stub until e8). */
  async function runEncrypt(): Promise<void> {
    const caps = recipientCaps().filter((r) => r.encryptable);
    const recipientPublicKeys = caps
      .map((r) => r.publicKey)
      .filter((k): k is string => k !== null);
    const kind = caps.find((r) => r.keyKind !== null)?.keyKind ?? 'pgp';
    // `exactOptionalPropertyTypes`: include `protectedSubject` only when set,
    // rather than passing an explicit `undefined`.
    const subject = props.subject?.();
    const subjectField =
      protectSubject() && subject !== undefined && subject.length > 0
        ? { protectedSubject: subject }
        : {};
    const result = await worker().encrypt({
      kind,
      plaintext: props.bodyText?.() ?? '',
      recipientPublicKeys,
      ...subjectField,
    });
    setEncryptedDraft(result);
  }

  function onEncryptChange(next: boolean): void {
    setEncrypt(next);
    if (next) {
      void runEncrypt();
    } else {
      setEncryptedDraft(null);
    }
  }

  // Re-encrypt when the protected-subject choice flips while encryption is on
  // (the affordance must reflect the new subject-protection state).
  function onProtectSubjectChange(next: boolean): void {
    setProtectSubject(next);
    if (encrypt()) void runEncrypt();
  }

  // Report the merged state to the host Compose after any change (e8 reads this
  // to encrypt-on-send, set the sign flag, and honor the DLP `canSend` gate).
  createEffect(() => {
    props.onChange?.({
      encrypt: encrypt(),
      sign: sign(),
      protectSubject: protectSubject(),
      capability: capability(),
      canSend: canSend(),
      encryptedDraft: encryptedDraft(),
      verdicts: verdicts(),
      recipients: recipientCaps(),
    });
  });

  return (
    <div class="cmpcrypto" data-testid="compose-crypto">
      <Show when={normalized().length > 0}>
        <CapabilityBanner
          capability={capability()}
          recipients={recipientCaps()}
          loading={capsData.loading}
        />
      </Show>

      <EncryptSignToggles
        encrypt={encrypt()}
        sign={sign()}
        protectSubject={protectSubject()}
        encryptDisabled={encryptDisabled()}
        encryptDisabledReason={
          normalized().length === 0
            ? 'Add a recipient to check for encryption keys.'
            : 'No recipient encryption key available — sending over TLS.'
        }
        drafted={encryptedDraft() !== null}
        onEncryptChange={onEncryptChange}
        onSignChange={setSign}
        onProtectSubjectChange={onProtectSubjectChange}
      />

      <DlpWarnings verdicts={verdicts()} loading={dlpData.loading} />
    </div>
  );
}
