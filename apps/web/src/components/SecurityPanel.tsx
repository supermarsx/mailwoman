// Reader Security panel (plan §3 e3 / SPEC §7.3) — a STANDALONE component that
// renders a FROZEN `SecurityVerdict` (`api/security-types.ts`) as a collapsed
// plain-language chip that expands into the full analysis: DKIM/SPF/DMARC/ARC
// verdicts, the Received chain (hops / delays / anomalies / optional ASN+country),
// the 3-state signature/cert analysis, per-attachment risk, and a sender-controls
// block. It owns no data-fetching: it takes the verdict + callbacks as props so
// e8 can mount it into the Reader toolbar and wire `SecurityVerdict/get` +
// `SenderControl/set` without touching this file. The `onSenderControl` prop
// defaults to a self-contained mock so the panel works + tests on its own.
//
// a11y (t8): every verdict badge carries a TEXT label (e.g. "DKIM passed") plus a
// per-tone glyph (`::before`, security-panel.css) so pass/fail is never conveyed by
// colour alone (WCAG 1.4.1). The chip is a real button (keyboard + aria-expanded);
// Escape collapses the panel and returns focus to it. Untrusted values (hop hosts,
// attachment names) get `dir="auto"` so a spoofed RTL run can't reorder the UI.
// i18n (t8): all labels come from `security.ftl` via `t()`.

import { For, Show, createMemo, createSignal, createUniqueId, onMount, type JSX } from 'solid-js';
import type {
  AttachmentRisk,
  AuthResult,
  ReceivedHop,
  SecurityVerdict,
  SignatureVerdict,
} from '../api/security-types.ts';
import {
  AUTH_RESULT_TONE,
  CHAIN_STATUS_TONE,
  REVOCATION_STATUS_TONE,
  SENDER_CONTROL_DANGER,
  attachmentTone,
  defaultSenderControl,
  formatDelay,
  overallTone,
  signatureTone,
  type SecurityTone,
  type SenderControlAction,
  type SenderControlRequest,
  type SenderControlResult,
} from './security/model.ts';
import { t, loadCatalog } from '../i18n';
import * as css from './security/security-panel.css.ts';

export interface SecurityPanelProps {
  /** The frozen verdict to render. */
  verdict: SecurityVerdict;
  /** Sender address, threaded to `SenderControl/set` (block/silence/report). */
  senderAddress?: string;
  /** Thread id, threaded to `SenderControl/set` (ignore-conversation). */
  threadId?: string;
  /** Start expanded (default collapsed to the chip). */
  initiallyExpanded?: boolean;
  /**
   * Dispatch a sender control. Defaults to a standalone mock (`defaultSenderControl`);
   * e8 passes a real `SenderControl/set` dispatch when it mounts the component.
   */
  onSenderControl?: (req: SenderControlRequest) => Promise<SenderControlResult> | void;
}

/** The sender controls rendered, in order. */
const SENDER_ACTIONS: SenderControlAction[] = [
  'block',
  'silence',
  'ignore-conversation',
  'report-phishing',
  'report-junk',
];

export function SecurityPanel(props: SecurityPanelProps): JSX.Element {
  onMount(() => void loadCatalog('security'));
  const [expanded, setExpanded] = createSignal(props.initiallyExpanded ?? false);
  const panelId = createUniqueId();
  const tone = createMemo(() => overallTone(props.verdict));
  let chipRef: HTMLButtonElement | undefined;

  // Escape collapses the expanded panel and returns focus to the chip (WCAG 2.1.2).
  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape' && expanded()) {
      e.stopPropagation();
      setExpanded(false);
      chipRef?.focus();
    }
  }

  return (
    <div class={css.root} data-tone={tone()} onKeyDown={onKeyDown}>
      <button
        ref={chipRef}
        type="button"
        class={`${css.chip} ${css.chipTone[tone()]}`}
        aria-expanded={expanded()}
        aria-controls={panelId}
        onClick={() => setExpanded((v) => !v)}
      >
        <span class={`${css.chipDot} ${css.chipDotTone[tone()]}`} aria-hidden="true" />
        <span class={css.chipLabel}>{props.verdict.plainLanguage}</span>
        <span class={css.chipCaret} aria-hidden="true">
          {expanded() ? '▲' : '▼'}
        </span>
      </button>

      <Show when={expanded()}>
        <div id={panelId} class={css.panel} role="region" aria-label={t('security-panel-region')}>
          <p class={css.summary}>{props.verdict.plainLanguage}</p>
          <AuthSection auth={props.verdict.auth} />
          <ReceivedSection hops={props.verdict.received} />
          <SignatureSection signature={props.verdict.signature} />
          <AttachmentsSection attachments={props.verdict.attachments} />
          <AnomaliesSection anomalies={props.verdict.anomalies} />
          <SenderControlsSection
            verdict={props.verdict}
            senderAddress={props.senderAddress}
            threadId={props.threadId}
            onSenderControl={props.onSenderControl}
          />
        </div>
      </Show>
    </div>
  );
}

// ── Tone badge ───────────────────────────────────────────────────────────────

function Badge(props: { tone: SecurityTone; children: JSX.Element }): JSX.Element {
  return <span class={`${css.badge} ${css.badgeTone[props.tone]}`}>{props.children}</span>;
}

// ── Email authentication ─────────────────────────────────────────────────────

function AuthSection(props: { auth: SecurityVerdict['auth'] }): JSX.Element {
  return (
    <section role="group" class={css.section} aria-labelledby="sec-auth-title">
      <h4 id="sec-auth-title" class={css.sectionTitle}>
        {t('security-section-auth')}
      </h4>
      <AuthRow name="DKIM" result={props.auth.dkim.result}>
        {authDetail([
          [t('security-detail-domain'), props.auth.dkim.domain],
          [t('security-detail-selector'), props.auth.dkim.selector],
        ])}
      </AuthRow>
      <AuthRow name="SPF" result={props.auth.spf.result}>
        {authDetail([[t('security-detail-domain'), props.auth.spf.domain]])}
      </AuthRow>
      <AuthRow name="DMARC" result={props.auth.dmarc.result}>
        {authDetail([
          [t('security-detail-policy'), props.auth.dmarc.policy],
          [
            t('security-detail-alignment'),
            props.auth.dmarc.aligned ? t('security-aligned') : t('security-not-aligned'),
          ],
        ])}
      </AuthRow>
      <AuthRow name="ARC" result={props.auth.arc.result}>
        {authDetail([[t('security-detail-chain-length'), String(props.auth.arc.chainLength)]])}
      </AuthRow>
    </section>
  );
}

function authDetail(fields: [string, string | null][]): string {
  const shown = fields.filter(([, v]) => v !== null && v !== '');
  if (shown.length === 0) return '';
  return shown.map(([k, v]) => `${k}: ${v}`).join(' · ');
}

function AuthRow(props: {
  name: string;
  result: AuthResult;
  children?: JSX.Element;
}): JSX.Element {
  return (
    <div class={css.authRow}>
      <span class={css.authName}>{props.name}</span>
      <span>
        <Badge tone={AUTH_RESULT_TONE[props.result]}>
          {props.name} {t(`security-auth-${props.result}`)}
        </Badge>
        <Show when={props.children}>
          <span class={css.authDetail}> {props.children}</span>
        </Show>
      </span>
    </div>
  );
}

// ── Received chain ───────────────────────────────────────────────────────────

function ReceivedSection(props: { hops: ReceivedHop[] }): JSX.Element {
  return (
    <section role="group" class={css.section} aria-labelledby="sec-received-title">
      <h4 id="sec-received-title" class={css.sectionTitle}>
        {t('security-section-received')}
      </h4>
      <Show
        when={props.hops.length > 0}
        fallback={<p class={css.empty}>{t('security-received-empty')}</p>}
      >
        <div class={css.hopScroll}>
          <ol class={css.hopList}>
            <For each={props.hops}>{(hop) => <HopRow hop={hop} />}</For>
          </ol>
        </div>
      </Show>
    </section>
  );
}

function HopRow(props: { hop: ReceivedHop }): JSX.Element {
  const hop = (): ReceivedHop => props.hop;
  const geo = (): string | null => {
    const h = hop();
    const parts: string[] = [];
    if (h.asn !== null) parts.push(`AS${h.asn}${h.asnOrg ? ` ${h.asnOrg}` : ''}`);
    if (h.country !== null) parts.push(h.country);
    return parts.length > 0 ? parts.join(' · ') : null;
  };
  return (
    <li class={css.hop}>
      <span class={css.hopIndex}>{hop().index}</span>
      <span class={css.hopBody}>
        {/* Untrusted host names — `dir="auto"` isolates their bidi run (SPEC §24). */}
        <span class={css.hopHost} dir="auto">
          {hop().fromHost ?? t('security-hop-unknown')} → {hop().byHost ?? t('security-hop-unknown')}
        </span>
        <span class={css.hopMeta}>
          <Show when={hop().protocol}>{(p) => <span>{p()}</span>}</Show>
          <Show when={hop().timestamp}>
            {(ts) => <time datetime={ts()}>{ts()}</time>}
          </Show>
          <span>+{formatDelay(hop().delayMs)}</span>
          <Show when={geo()}>{(g) => <span>{g()}</span>}</Show>
        </span>
      </span>
    </li>
  );
}

// ── Signature / certificate ──────────────────────────────────────────────────

function SignatureSection(props: { signature: SignatureVerdict | null }): JSX.Element {
  return (
    <section role="group" class={css.section} aria-labelledby="sec-signature-title">
      <h4 id="sec-signature-title" class={css.sectionTitle}>
        {t('security-section-signature')}
      </h4>
      <Show
        when={props.signature}
        fallback={<p class={css.empty}>{t('security-sig-none')}</p>}
      >
        {(sig) => <SignatureBody signature={sig()} />}
      </Show>
    </section>
  );
}

function SignatureBody(props: { signature: SignatureVerdict }): JSX.Element {
  const sig = (): SignatureVerdict => props.signature;
  return (
    <div>
      <Badge tone={signatureTone(sig())}>
        {sig().kind.toUpperCase()}: {t(`security-sig-${sig().status}`)}
      </Badge>
      <dl class={css.factGrid}>
        <Show when={sig().signerKeyId}>
          {(v) => (
            <Fact k={t('security-fact-signer-key')}>
              <span dir="auto">{v()}</span>
            </Fact>
          )}
        </Show>
        <Show when={sig().algorithm}>{(v) => <Fact k={t('security-fact-algorithm')}>{v()}</Fact>}</Show>
        <Show when={sig().keyCreatedAt}>
          {(v) => <Fact k={t('security-fact-key-created')}>{v()}</Fact>}
        </Show>
        <Show when={sig().keyExpiresAt}>
          {(v) => <Fact k={t('security-fact-key-expires')}>{v()}</Fact>}
        </Show>
        <Show when={sig().chainStatus}>
          {(v) => (
            <Fact k={t('security-fact-chain')}>
              <Badge tone={CHAIN_STATUS_TONE[v()]}>{t(`security-chain-${v()}`)}</Badge>
            </Fact>
          )}
        </Show>
        <Show when={sig().revocationStatus}>
          {(v) => (
            <Fact k={t('security-fact-revocation')}>
              <Badge tone={REVOCATION_STATUS_TONE[v()]}>{t(`security-revocation-${v()}`)}</Badge>
            </Fact>
          )}
        </Show>
        <Show when={sig().keyChanged}>
          <Fact k={t('security-fact-key-change')}>
            <Badge tone="warning">{t('security-key-changed')}</Badge>
          </Fact>
        </Show>
      </dl>
    </div>
  );
}

function Fact(props: { k: string; children: JSX.Element }): JSX.Element {
  return (
    <>
      <dt class={css.factKey}>{props.k}</dt>
      <dd class={css.factVal}>{props.children}</dd>
    </>
  );
}

// ── Attachments ──────────────────────────────────────────────────────────────

function AttachmentsSection(props: { attachments: AttachmentRisk[] }): JSX.Element {
  return (
    <section role="group" class={css.section} aria-labelledby="sec-attachments-title">
      <h4 id="sec-attachments-title" class={css.sectionTitle}>
        {t('security-section-attachments')}
      </h4>
      <Show
        when={props.attachments.length > 0}
        fallback={<p class={css.empty}>{t('security-attachments-empty')}</p>}
      >
        <ul class={css.list}>
          <For each={props.attachments}>{(a) => <AttachmentRow attachment={a} />}</For>
        </ul>
      </Show>
    </section>
  );
}

function AttachmentRow(props: { attachment: AttachmentRisk }): JSX.Element {
  const a = (): AttachmentRisk => props.attachment;
  return (
    <li class={css.attachItem}>
      {/* Untrusted file name — `dir="auto"` defeats the exe.png↔gnp.exe spoof. */}
      <span class={css.attachName} dir="auto">
        {a().name}
      </span>
      <Badge tone={attachmentTone(a())}>{t(`security-attach-${a().risk}`)}</Badge>
      <Show when={a().mismatch}>
        <span class={css.mismatchNote}>
          {t('security-attach-mismatch', {
            declared: a().declaredType ?? t('security-unknown'),
            detected: a().detectedType ?? t('security-unknown'),
          })}
        </span>
      </Show>
    </li>
  );
}

// ── Anomalies ────────────────────────────────────────────────────────────────

function AnomaliesSection(props: {
  anomalies: SecurityVerdict['anomalies'];
}): JSX.Element {
  return (
    <Show when={props.anomalies.length > 0}>
      <section role="group" class={css.section} aria-labelledby="sec-anomalies-title">
        <h4 id="sec-anomalies-title" class={css.sectionTitle}>
          {t('security-section-warnings')}
        </h4>
        <ul class={css.list}>
          <For each={props.anomalies}>
            {(token) => (
              <li class={css.anomalyItem} data-anomaly={token}>
                <span class={css.anomalyMark} aria-hidden="true">
                  ⚠
                </span>
                <span>{t(`security-anomaly-${token}`)}</span>
              </li>
            )}
          </For>
        </ul>
      </section>
    </Show>
  );
}

// ── Sender controls ──────────────────────────────────────────────────────────

function SenderControlsSection(props: {
  verdict: SecurityVerdict;
  senderAddress: string | undefined;
  threadId: string | undefined;
  onSenderControl: ((req: SenderControlRequest) => Promise<SenderControlResult> | void) | undefined;
}): JSX.Element {
  const [status, setStatus] = createSignal('');
  const [pending, setPending] = createSignal<SenderControlAction | null>(null);

  function requestFor(action: SenderControlAction): SenderControlRequest {
    const req: SenderControlRequest = { action, emailId: props.verdict.emailId };
    if (action === 'ignore-conversation') {
      if (props.threadId !== undefined) req.threadId = props.threadId;
    } else if (props.senderAddress !== undefined) {
      req.address = props.senderAddress;
    }
    if (action === 'report-phishing' || action === 'report-junk') req.abuseReport = true;
    return req;
  }

  async function run(action: SenderControlAction): Promise<void> {
    setPending(action);
    try {
      const handler = props.onSenderControl ?? defaultSenderControl;
      await handler(requestFor(action));
      setStatus(t(`security-control-done-${action}`));
    } finally {
      setPending(null);
    }
  }

  return (
    <section role="group" class={css.section} aria-labelledby="sec-controls-title">
      <h4 id="sec-controls-title" class={css.sectionTitle}>
        {t('security-section-controls')}
      </h4>
      <div class={css.controls}>
        <For each={SENDER_ACTIONS}>
          {(action) => (
            <button
              type="button"
              class={`${css.controlBtn} ${
                SENDER_CONTROL_DANGER.has(action) ? css.controlBtnDanger : ''
              }`}
              disabled={pending() !== null}
              onClick={() => void run(action)}
            >
              {t(`security-control-${action}`)}
            </button>
          )}
        </For>
      </div>
      <p class={css.statusLive} role="status" aria-live="polite">
        {status()}
      </p>
    </section>
  );
}
