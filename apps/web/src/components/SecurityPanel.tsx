// Reader Security panel (plan §3 e3 / SPEC §7.3) — a STANDALONE component that
// renders a FROZEN `SecurityVerdict` (`api/security-types.ts`) as a collapsed
// plain-language chip that expands into the full analysis: DKIM/SPF/DMARC/ARC
// verdicts, the Received chain (hops / delays / anomalies / optional ASN+country),
// the 3-state signature/cert analysis, per-attachment risk, and a sender-controls
// block. It owns no data-fetching: it takes the verdict + callbacks as props so
// e8 can mount it into the Reader toolbar and wire `SecurityVerdict/get` +
// `SenderControl/set` without touching this file. The `onSenderControl` prop
// defaults to a self-contained mock so the panel works + tests on its own.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import type {
  AttachmentRisk,
  AuthResult,
  ReceivedHop,
  SecurityVerdict,
  SignatureVerdict,
} from '../api/security-types.ts';
import {
  ANOMALY_LABEL,
  ATTACHMENT_RISK_LABEL,
  AUTH_RESULT_LABEL,
  AUTH_RESULT_TONE,
  CHAIN_STATUS_LABEL,
  CHAIN_STATUS_TONE,
  REVOCATION_STATUS_LABEL,
  REVOCATION_STATUS_TONE,
  SENDER_CONTROL_DANGER,
  SENDER_CONTROL_DONE,
  SENDER_CONTROL_LABEL,
  SIGNATURE_STATUS_LABEL,
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
  const [expanded, setExpanded] = createSignal(props.initiallyExpanded ?? false);
  const panelId = createUniqueId();
  const tone = createMemo(() => overallTone(props.verdict));

  return (
    <div class={css.root} data-tone={tone()}>
      <button
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
        <div id={panelId} class={css.panel} role="region" aria-label="Message security details">
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
        Authentication
      </h4>
      <AuthRow name="DKIM" result={props.auth.dkim.result}>
        {authDetail([
          ['domain', props.auth.dkim.domain],
          ['selector', props.auth.dkim.selector],
        ])}
      </AuthRow>
      <AuthRow name="SPF" result={props.auth.spf.result}>
        {authDetail([['domain', props.auth.spf.domain]])}
      </AuthRow>
      <AuthRow name="DMARC" result={props.auth.dmarc.result}>
        {authDetail([
          ['policy', props.auth.dmarc.policy],
          ['alignment', props.auth.dmarc.aligned ? 'aligned' : 'not aligned'],
        ])}
      </AuthRow>
      <AuthRow name="ARC" result={props.auth.arc.result}>
        {authDetail([['chain length', String(props.auth.arc.chainLength)]])}
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
          {props.name} {AUTH_RESULT_LABEL[props.result]}
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
        Delivery path
      </h4>
      <Show
        when={props.hops.length > 0}
        fallback={<p class={css.empty}>No Received chain available.</p>}
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
        <span class={css.hopHost}>
          {hop().fromHost ?? 'unknown'} → {hop().byHost ?? 'unknown'}
        </span>
        <span class={css.hopMeta}>
          <Show when={hop().protocol}>{(p) => <span>{p()}</span>}</Show>
          <Show when={hop().timestamp}>
            {(t) => <time datetime={t()}>{t()}</time>}
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
        Signature
      </h4>
      <Show
        when={props.signature}
        fallback={<p class={css.empty}>{SIGNATURE_STATUS_LABEL.none}</p>}
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
        {sig().kind.toUpperCase()}: {SIGNATURE_STATUS_LABEL[sig().status]}
      </Badge>
      <dl class={css.factGrid}>
        <Show when={sig().signerKeyId}>
          {(v) => <Fact k="Signer key">{v()}</Fact>}
        </Show>
        <Show when={sig().algorithm}>{(v) => <Fact k="Algorithm">{v()}</Fact>}</Show>
        <Show when={sig().keyCreatedAt}>
          {(v) => <Fact k="Key created">{v()}</Fact>}
        </Show>
        <Show when={sig().keyExpiresAt}>
          {(v) => <Fact k="Key expires">{v()}</Fact>}
        </Show>
        <Show when={sig().chainStatus}>
          {(v) => (
            <Fact k="Chain">
              <Badge tone={CHAIN_STATUS_TONE[v()]}>{CHAIN_STATUS_LABEL[v()]}</Badge>
            </Fact>
          )}
        </Show>
        <Show when={sig().revocationStatus}>
          {(v) => (
            <Fact k="Revocation">
              <Badge tone={REVOCATION_STATUS_TONE[v()]}>{REVOCATION_STATUS_LABEL[v()]}</Badge>
            </Fact>
          )}
        </Show>
        <Show when={sig().keyChanged}>
          <Fact k="Key change">
            <Badge tone="warning">Signer key changed since last seen</Badge>
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
        Attachments
      </h4>
      <Show
        when={props.attachments.length > 0}
        fallback={<p class={css.empty}>No attachments.</p>}
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
      <span class={css.attachName}>{a().name}</span>
      <Badge tone={attachmentTone(a())}>{ATTACHMENT_RISK_LABEL[a().risk]}</Badge>
      <Show when={a().mismatch}>
        <span class={css.mismatchNote}>
          type mismatch (declared {a().declaredType ?? 'unknown'}, detected{' '}
          {a().detectedType ?? 'unknown'})
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
          Warnings
        </h4>
        <ul class={css.list}>
          <For each={props.anomalies}>
            {(token) => (
              <li class={css.anomalyItem} data-anomaly={token}>
                <span class={css.anomalyMark} aria-hidden="true">
                  ⚠
                </span>
                <span>{ANOMALY_LABEL[token]}</span>
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
      setStatus(SENDER_CONTROL_DONE[action]);
    } finally {
      setPending(null);
    }
  }

  return (
    <section role="group" class={css.section} aria-labelledby="sec-controls-title">
      <h4 id="sec-controls-title" class={css.sectionTitle}>
        Sender controls
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
              {SENDER_CONTROL_LABEL[action]}
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
