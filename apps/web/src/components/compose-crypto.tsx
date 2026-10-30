import { Icon } from './Icon.tsx';
import { RibbonItem } from './bars/index.ts';
import { RibbonPopover } from './compose/ribbonParts.tsx';
// Compose crypto + DLP subcomponents (plan §2.5). STANDALONE, self-contained
// pieces that take props + callbacks so each is independently testable; the host
// `Compose.tsx` mounts them and reads the reported state on send. Three concerns:
//
//   • <EncryptSignToggles>  — per-message encrypt / sign switches + the
//                             encrypted-draft & protected-subject affordances.
//   • <CapabilityBanner>    — the live "this message will be E2EE / TLS / mixed"
//                             banner, computed from per-recipient `CryptoKey/lookup`.
//   • <DlpWarnings>         — inline warn / require-encryption / block verdicts from
//                             `Dlp/scan`; a `block` shows the rule message + gates send.
//
// <ComposeCrypto> wires all three: it probes recipient keys + DLP through the
// `lookupKeys` / `scanDlp` callbacks (mock in tests, `crypto-jmap.ts` factories at
// runtime), calls the crypto worker to encrypt the draft when encryption is enabled
// (the real WASM worker in the browser; a deterministic stub under vitest), folds a
// signature into that encrypt via `signWithKeyRef` when `sign` is on, and reports
// {encrypt, sign, capability, canSend, …} up via `onChange` so the host Compose can
// encrypt-on-send, clear-sign a sign-only send, and honor the DLP gate.

import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  on,
  onMount,
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
import { t, loadCatalog } from '../i18n';
import './compose/compose-crypto.css';

export type { TransportCapability, RecipientCapability } from './compose/capability.ts';
export type { DlpAttachmentMeta, DlpScanFn, KeyLookupFn } from './compose/crypto-jmap.ts';

// ── Presentational: the E2EE / TLS / mixed banner ────────────────────────────

export function CapabilityBanner(props: {
  capability: TransportCapability;
  recipients: RecipientCapability[];
  loading?: boolean;
  /** The detail under the composer's status chip (t38): no testid or live role, the chip has them. */
  detailOnly?: boolean;
}): JSX.Element {
  const copy = (): { label: string; detail: string } => ({
    label: t(`crypto-banner-${props.capability}-label`),
    detail: t(`crypto-banner-${props.capability}-detail`),
  });
  const uncovered = (): RecipientCapability[] => props.recipients.filter((r) => !r.encryptable);
  return (
    <div
      class="cmpcrypto-banner"
      classList={{ [`cmpcrypto-banner--${props.capability}`]: true }}
      data-testid={props.detailOnly ? undefined : 'compose-crypto-banner'}
      data-capability={props.capability}
      role={props.detailOnly ? undefined : 'status'}
      aria-live={props.detailOnly ? undefined : 'polite'}
    >
      <span class="cmpcrypto-banner__badge" aria-hidden="true">
        <Icon name={props.capability === 'e2ee' ? 'lock' : props.capability === 'mixed' ? 'alert' : 'unlock'} />
      </span>
      <span class="cmpcrypto-banner__text">
        <strong class="cmpcrypto-banner__label">
          {copy().label}
          <Show when={props.loading}>
            <span class="cmpcrypto-banner__spin" aria-label={t('crypto-banner-checking-label')}>
              {' '}
              {t('crypto-banner-checking')}
            </span>
          </Show>
        </strong>
        <span class="cmpcrypto-banner__detail">{copy().detail}</span>
        <Show when={props.capability === 'mixed' && uncovered().length > 0}>
          <span class="cmpcrypto-banner__uncovered">
            {t('crypto-banner-tls-only')}{' '}
            <For each={uncovered()}>
              {(r, i) => (
                <>
                  <Show when={i() > 0}>, </Show>
                  {/* Untrusted recipient address — `dir="auto"` isolates its bidi run. */}
                  <span class="cmpcrypto-banner__addr" dir="auto">
                    {r.address}
                  </span>
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
      <legend class="cmpcrypto-toggles__legend">{t('crypto-toggles-legend')}</legend>

      <label class="cmpcrypto-toggle" classList={{ 'cmpcrypto-toggle--disabled': props.encryptDisabled }}>
        <input
          type="checkbox"
          checked={props.encrypt}
          disabled={props.encryptDisabled}
          data-testid="encrypt-toggle"
          aria-describedby="cmpcrypto-encrypt-hint"
          onChange={(e) => props.onEncryptChange(e.currentTarget.checked)}
        />
        <span class="cmpcrypto-toggle__label">{t('crypto-encrypt-label')}</span>
      </label>
      <span id="cmpcrypto-encrypt-hint" class="cmpcrypto-toggle__hint">
        <Show
          when={!props.encryptDisabled}
          fallback={props.encryptDisabledReason ?? t('crypto-encrypt-no-key')}
        >
          <Show when={props.encrypt && props.drafted} fallback={t('crypto-encrypt-hint-default')}>
            <span class="cmpcrypto-toggle__drafted" data-testid="encrypted-draft-indicator">
              {t('crypto-encrypt-drafted')}
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
        <span class="cmpcrypto-toggle__label">{t('crypto-sign-label')}</span>
      </label>

      <Show when={props.encrypt && !props.encryptDisabled}>
        <label class="cmpcrypto-toggle cmpcrypto-toggle--sub">
          <input
            type="checkbox"
            checked={props.protectSubject}
            data-testid="protect-subject-toggle"
            onChange={(e) => props.onProtectSubjectChange(e.currentTarget.checked)}
          />
          <span class="cmpcrypto-toggle__label">{t('crypto-protect-subject')}</span>
        </label>
      </Show>
    </fieldset>
  );
}

// ── Presentational: DLP pre-send warnings ────────────────────────────────────

const DLP_ACTION_SEVERITY: Record<DlpAction, 'block' | 'warn'> = {
  block: 'block',
  'require-encryption': 'warn',
  warn: 'warn',
  'notify-admin': 'warn',
};

const DLP_ACTION_TITLE_ID: Record<DlpAction, string> = {
  block: 'crypto-dlp-block',
  'require-encryption': 'crypto-dlp-require',
  warn: 'crypto-dlp-warn',
  'notify-admin': 'crypto-dlp-notify',
};

export function DlpWarnings(props: { verdicts: DlpVerdict[]; loading?: boolean }): JSX.Element {
  const hasBlock = (): boolean => props.verdicts.some((v) => v.action === 'block');
  return (
    <Show when={props.verdicts.length > 0}>
      <ul
        class="cmpcrypto-dlp"
        data-testid="dlp-warnings"
        aria-label={t('crypto-dlp-aria')}
        // A blocking verdict is assertive; advisory ones are polite.
        role={hasBlock() ? 'alert' : 'status'}
        aria-live={hasBlock() ? 'assertive' : 'polite'}
      >
        <For each={props.verdicts}>
          {(v) => {
            const severity = DLP_ACTION_SEVERITY[v.action];
            return (
              <li
                class="cmpcrypto-dlp__item"
                classList={{ [`cmpcrypto-dlp__item--${severity}`]: true }}
                data-testid={v.action === 'block' ? 'dlp-block' : 'dlp-warn'}
                data-action={v.action}
              >
                <span class="cmpcrypto-dlp__title">{t(DLP_ACTION_TITLE_ID[v.action])}</span>
                {/* Admin-authored rule name — `dir="auto"` isolates its bidi run. */}
                <span class="cmpcrypto-dlp__msg" dir="auto">
                  {v.ruleName}
                </span>
                <Show when={v.excerptRedacted.length > 0}>
                  {/* Redacted message excerpt (user content) — isolate its direction. */}
                  <span class="cmpcrypto-dlp__excerpt" dir="auto">
                    {v.excerptRedacted}
                  </span>
                </Show>
                <Show when={v.matchedDetectors.length > 0}>
                  <span class="cmpcrypto-dlp__detectors">
                    {t('crypto-dlp-matched', { list: v.matchedDetectors.join(', ') })}
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
  /** The worker-produced encrypted draft (signed in-place when `sign` is on), or `null`. */
  encryptedDraft: EncryptResult | null;
  /** The signing keyRef the encrypted draft was signed with, or `null` when it
   *  carries no signature (t87 D11: the host checks it is its current key). */
  draftSignedWith: string | null;
  draftPlaintext?: string | null;
  draftSource?: string | null;
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
  /** Override the crypto worker (defaults to the process worker: real WASM in the
   *  browser, a deterministic stub under vitest). */
  cryptoWorker?: CryptoWorkerApi;
  /** The session-cached signing keyRef (from `unlockKey`), or `null` when the
   *  signing key is locked. When `sign` is on and this is non-null, the encrypt
   *  call folds in a signature via `signWithKeyRef`. */
  signingKeyRef?: () => string | null;
  /** Ask the host to unlock the signing key (prompt for the passphrase) — called
   *  when `sign` is switched on while the key is still locked. */
  onRequestSigningKey?: () => void;
  /** Report crypto/DLP state to the host Compose after every change. */
  onChange?: (state: ComposeCryptoState) => void;
}

export function cryptoDraftSource(body:string,recipients:readonly string[],subject:string,protect:boolean,signingKey:string|null):string {return JSON.stringify([body,normalizeRecipients([...recipients]),subject,protect,signingKey]);}

export function ComposeCrypto(props: ComposeCryptoProps): JSX.Element {
  onMount(() => void loadCatalog('crypto'));
  const worker = (): CryptoWorkerApi => props.cryptoWorker ?? getCryptoWorker();

  const [encrypt, setEncrypt] = createSignal(false);
  const [sign, setSign] = createSignal(false);
  const [protectSubject, setProtectSubject] = createSignal(false);
  const [encryptedDraft, setEncryptedDraft] = createSignal<EncryptResult | null>(null);
  const [draftSignedWith, setDraftSignedWith] = createSignal<string | null>(null);
  // Only the newest encrypt call may set the draft: one started with a signing
  // key that has since been dropped must not land after the switch.
  let encryptRun = 0;
  const [draftSource,setDraftSource]=createSignal<string|null>(null);
  const source=()=>cryptoDraftSource(props.bodyText?.()??'',props.recipients(),props.subject?.()??'',protectSubject(),sign()?props.signingKeyRef?.()??null:null);
  const [draftPlaintext,setDraftPlaintext]=createSignal<string|null>(null);

  const normalized = createMemo(() => normalizeRecipients(props.recipients()));
  const recipientKey = createMemo(() => normalized().join(','));

  // Probe each recipient's keys via `lookupKeys`. Keyed on the joined address
  // list so it re-runs only when the recipient set actually changes; the
  // resource takes the latest result, so out-of-order lookups can't race.
  //
  // Both resources here start with a value and are read through `.latest`, so
  // reading them never suspends: a lookup or a scan in flight shows the result
  // before it (and `loading`), instead of taking whatever Suspense boundary is
  // above the composer out of the document on every keystroke.
  const [capsData] = createResource(
    recipientKey,
    async (key): Promise<RecipientCapability[]> => {
      if (key.length === 0) return [];
      const addrs = key.split(',');
      return Promise.all(
        addrs.map(async (addr) => chooseRecipientKey(addr, await props.lookupKeys(addr))),
      );
    },
    { initialValue: [] },
  );
  const recipientCaps = (): RecipientCapability[] => capsData.latest;
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
  const [dlpData] = createResource(dlpKey, async (): Promise<DlpVerdict[]> => props.scanDlp(draft()), {
    initialValue: [],
  });
  const verdicts = (): DlpVerdict[] => dlpData.latest;
  const canSend = createMemo(() => !verdicts().some((v) => v.action === 'block'));

  /** Encrypt the current draft body via the worker; folds in a signature via
   *  `signWithKeyRef` when `sign` is on and the signing key is unlocked, so
   *  encrypt+sign produces one signed-and-encrypted message. */
  async function runEncrypt(): Promise<void> {
    const run = ++encryptRun;
    setEncryptedDraft(null);setDraftPlaintext(null);
    const plaintext=props.bodyText?.()??'';
    const sourceKey=source();setDraftSource(null);
    if(JSON.stringify(normalizeRecipients(recipientCaps().map(cap=>cap.address)))!==JSON.stringify(normalized()))return;
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
    // Sign-on-send: when the sign toggle is on and the host has unlocked the
    // signing key, hand the worker the session keyRef so the ciphertext is
    // signed as well as encrypted (omit it when locked → an unsigned draft that
    // the effect below re-signs once the key is unlocked).
    const signRef = sign() ? props.signingKeyRef?.() ?? null : null;
    const signField = signRef !== null ? { signWithKeyRef: signRef } : {};
    const result = await worker().encrypt({
      kind,
      plaintext,
      recipientPublicKeys,
      ...subjectField,
      ...signField,
    }).catch(()=>null);
    if(result===null)return;
    if (run !== encryptRun || signRef !== (sign() ? props.signingKeyRef?.() ?? null : null)) return;
    if(sourceKey!==source())return;
    setDraftSource(sourceKey);
    setDraftPlaintext(plaintext);
    setEncryptedDraft(result);
    setDraftSignedWith(signRef);
  }

  function onEncryptChange(next: boolean): void {
    setEncrypt(next);
    if (next) {
      void runEncrypt();
    } else {
      encryptRun += 1;
      setEncryptedDraft(null);
      setDraftSignedWith(null);
    }
  }

  // Re-encrypt when the protected-subject choice flips while encryption is on
  // (the affordance must reflect the new subject-protection state).
  function onProtectSubjectChange(next: boolean): void {
    setProtectSubject(next);
    if (encrypt()) void runEncrypt();
  }

  // Toggling `sign`: when turning it on while the signing key is still locked,
  // ask the host to unlock it. Re-encrypt whenever encryption is on so the draft
  // reflects the new sign state (turning sign OFF re-encrypts without a
  // signature; turning it ON with the key already unlocked folds one in).
  function onSignChange(next: boolean): void {
    setSign(next);
    if (next && (props.signingKeyRef?.() ?? null) === null) props.onRequestSigningKey?.();
    if (encrypt()) void runEncrypt();
  }

  // When the signing key finishes unlocking (the host's `signingKeyRef` changes
  // to a non-null ref) while sign + encrypt are both on, re-encrypt so the
  // ciphertext carries the signature. `defer` skips the initial run; `on` tracks
  // only the ref, so a sign-toggle handled above doesn't double-encrypt here.
  createEffect(
    on(
      () => props.signingKeyRef?.() ?? null,
      (ref) => {
        // A dropped key (null, after a mailbox switch) re-encrypts unsigned so
        // no ciphertext signed by it stays in the draft.
        if (encrypt() && (sign() || ref === null)) void runEncrypt();
      },
      { defer: true },
    ),
  );

  createEffect(on(()=>JSON.stringify([draft(),recipientCaps()]),()=>{if(encrypt())void runEncrypt();},{defer:true}));

  // Report the merged state to the host Compose after any change (the host reads
  // this to encrypt-on-send, clear-sign a sign-only send, and honor the DLP
  // `canSend` gate).
  createEffect(() => {
    props.onChange?.({
      encrypt: encrypt(),
      sign: sign(),
      protectSubject: protectSubject(),
      capability: capability(),
      canSend: canSend(),
      encryptedDraft: encryptedDraft(),
      draftSignedWith: draftSignedWith(),
      draftPlaintext:draftPlaintext(),
      draftSource:draftSource(),
      verdicts: verdicts(),
      recipients: recipientCaps(),
    });
  });

  // The Security group's controls, three rows (t38 §2.2): encrypt and sign,
  // then the live status as a one-line chip whose detail opens in a popover;
  // the protected-subject option and a policy (DLP) chip take the next column.
  // Always mounted (the ribbon keeps hidden tabs), so the evaluation is live.
  const encryptHint = (): string =>
    !encryptDisabled()
      ? encrypt() && encryptedDraft() !== null
        ? t('crypto-encrypt-drafted')
        : t('crypto-encrypt-hint-default')
      : normalized().length === 0
        ? t('crypto-reason-add-recipient')
        : t('crypto-reason-tls');
  return (
    <>
      <RibbonItem><div class="compose-rib__row">
        <label
          class="cmpcrypto-toggle"
          classList={{ 'cmpcrypto-toggle--disabled': encryptDisabled() }}
          data-tooltip={encryptHint()}
        >
          <input
            type="checkbox"
            checked={encrypt()}
            disabled={encryptDisabled()}
            data-testid="encrypt-toggle"
            aria-describedby="cmpcrypto-encrypt-hint"
            onChange={(e) => onEncryptChange(e.currentTarget.checked)}
          />
          <span class="cmpcrypto-toggle__label">{t('crypto-encrypt-label')}</span>
          <Show when={encrypt() && encryptedDraft() !== null}>
            <span class="cmpcrypto-toggle__drafted" aria-hidden="true">
              <Icon name="check" />
            </span>
          </Show>
        </label>
        <span id="cmpcrypto-encrypt-hint" hidden>
          <Show when={encrypt() && encryptedDraft() !== null} fallback={encryptHint()}>
            <span data-testid="encrypted-draft-indicator">{t('crypto-encrypt-drafted')}</span>
          </Show>
        </span>
      </div></RibbonItem>
      <RibbonItem>
        <label class="cmpcrypto-toggle">
          <input
            type="checkbox"
            checked={sign()}
            data-testid="sign-toggle"
            onChange={(e) => onSignChange(e.currentTarget.checked)}
          />
          <span class="cmpcrypto-toggle__label">{t('crypto-sign-label')}</span>
        </label>
      </RibbonItem>
      <RibbonItem>
        <Show when={normalized().length > 0}>
          <CapabilityChip capability={capability()} recipients={recipientCaps()} loading={capsData.loading} />
        </Show>
      </RibbonItem>
      <Show when={encrypt() && !encryptDisabled()}>
        <RibbonItem>
          <label class="cmpcrypto-toggle">
            <input
              type="checkbox"
              checked={protectSubject()}
              data-testid="protect-subject-toggle"
              onChange={(e) => onProtectSubjectChange(e.currentTarget.checked)}
            />
            <span class="cmpcrypto-toggle__label">{t('crypto-protect-subject')}</span>
          </label>
        </RibbonItem>
      </Show>
      <Show when={verdicts().length > 0}>
        <RibbonItem>
          <DlpChip verdicts={verdicts()} loading={dlpData.loading} />
        </RibbonItem>
      </Show>
    </>
  );
}

/** The transport status on one line (the Security group's third row); the detail opens below it. */
export function CapabilityChip(props: {
  capability: TransportCapability;
  recipients: RecipientCapability[];
  loading?: boolean;
}): JSX.Element {
  const label = (): string => t(`crypto-banner-${props.capability}-label`);
  return (
    <RibbonPopover
      label={label()}
      popoverLabel={t('crypto-toggles-legend')}
      buttonClass={`compose-rt__btn cmpcrypto-chip cmpcrypto-chip--${props.capability}`}
      content={
        <span
          class="cmpcrypto-chip__status"
          data-testid="compose-crypto-banner"
          data-capability={props.capability}
          role="status"
          aria-live="polite"
        >
          <Icon name={props.capability === 'e2ee' ? 'lock' : props.capability === 'mixed' ? 'alert' : 'unlock'} />
          <span class="compose-rib__text">
            {label()}
            <Show when={props.loading}>
              <span class="cmpcrypto-banner__spin" aria-label={t('crypto-banner-checking-label')}>
                {' '}
                {t('crypto-banner-checking')}
              </span>
            </Show>
          </span>
        </span>
      }
    >
      <CapabilityBanner capability={props.capability} recipients={props.recipients} detailOnly />
    </RibbonPopover>
  );
}

/** The policy (DLP) result on one line: the most severe verdict; the list opens below it. */
export function DlpChip(props: { verdicts: DlpVerdict[]; loading?: boolean }): JSX.Element {
  const worst = (): DlpVerdict | undefined =>
    props.verdicts.find((v) => v.action === 'block') ?? props.verdicts[0];
  const blocked = (): boolean => worst()?.action === 'block';
  const title = (): string => {
    const v = worst();
    return v ? t(DLP_ACTION_TITLE_ID[v.action]) : '';
  };
  return (
    <RibbonPopover
      label={title()}
      popoverLabel={t('crypto-dlp-aria')}
      buttonClass={`compose-rt__btn cmpcrypto-chip cmpcrypto-chip--${blocked() ? 'block' : 'warn'}`}
      content={
        <span
          class="cmpcrypto-chip__status"
          role={blocked() ? 'alert' : 'status'}
          aria-live={blocked() ? 'assertive' : 'polite'}
        >
          <Icon name="alert" />
          <span class="compose-rib__text">{title()}</span>
        </span>
      }
    >
      <DlpWarnings verdicts={props.verdicts} {...(props.loading !== undefined ? { loading: props.loading } : {})} />
    </RibbonPopover>
  );
}
