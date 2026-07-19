import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  onMount,
  Show,
  type JSX,
} from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
// Reading-pane layout switch (W3): the globalStyle overrides keyed on
// `:root[data-reading-pane]`. Imported for its side effect so the rules ship.
import './readerPane.css.ts';
import { SweepDialog } from './SweepDialog.tsx';
import { ThumbnailStrip, type StripItem } from '../viewers/ThumbnailStrip.tsx';
import { AttachmentViewer } from '../viewers/AttachmentViewer.tsx';
import { buildDownloadUrl, fetchObjectUrl, type AttachmentPart } from '../viewers/attachments.ts';
import { SecurityPanel } from './SecurityPanel.tsx';
// V7 auto-tag (SPEC §14.3, e14b): model-suggested labels for the open message.
// Gated on the `auto-tag` Assist capability, so a Disabled gateway renders nothing.
import { AutoTag, type TagSuggestion } from '../modules/assist/index.ts';
import type { SenderControlRequest, SenderControlResult } from './security/model.ts';
import { defaultSenderControl } from './security/model.ts';
import { MaxSecuritySwitch } from '../viewers/MaxSecuritySwitch.tsx';
import { createMaxSecurityStore } from '../viewers/max-security.ts';
import { bodyFrameDoc } from '../viewers/sandbox.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { responseFor } from '../api/jmap.ts';
import { CAP_CORE } from '../api/jmap-types.ts';
import { CAP_CRYPTO, CAP_SECURITY, type CryptoKey } from '../api/crypto-types.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';
import type { SecurityVerdict, SignatureVerdict } from '../api/security-types.ts';
import {
  analyzeBlockedContent,
  coveringGrant,
  createRemoteImageApi,
  hasBlockedContent,
  type GrantScope,
} from '../api/remote-images.ts';
import { RemoteContentBar } from './RemoteContentBar.tsx';

// The crypto/security JMAP surface (`SecurityVerdict/get`, `SenderControl/set`)
// is not exposed on `AppState`, so this component drives it over its own client
// (stateless — hits the same session as the store's client; browser: same-origin
// cookie, native shell: configured base + bearer). The max-security policy is an
// app-singleton (per-sender + global, persisted in localStorage) — plan §2.5.
const jmapClient = createConfiguredClient();
const maxsec = createMaxSecurityStore();
// Remote-image grant surface (t16 §S8/S9): the reader drives grant/revoke/list
// over the same session client. The wire shapes are localized in
// `api/remote-images.ts` (the e6 seam); this reader only calls the interface.
const remoteImages = createRemoteImageApi(jmapClient);

const SECURITY_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

/** Regex-extract a PGP MESSAGE armor block from a body value (encrypted mail). */
const PGP_MESSAGE_RE = /-----BEGIN PGP MESSAGE-----[\s\S]*?-----END PGP MESSAGE-----/;

interface VerdictGetResponse {
  accountId: string;
  state: string;
  list: SecurityVerdict[];
  notFound: string[];
}

/** Fetch the server-computed security verdict for an email (public facets). */
async function fetchVerdict(accountId: string, emailId: string): Promise<SecurityVerdict | null> {
  try {
    const res = await jmapClient.jmap({
      using: SECURITY_USING,
      methodCalls: [['SecurityVerdict/get', { accountId, ids: [emailId] }, 'v']],
    });
    return responseFor<VerdictGetResponse>(res, 'v').list[0] ?? null;
  } catch {
    // A missing/failed verdict simply hides the chip — never breaks the reader.
    return null;
  }
}

/** Apply a sender control (block/silence/ignore/report) to the real engine. */
async function dispatchSenderControl(
  accountId: string,
  emailId: string,
  req: SenderControlRequest,
): Promise<SenderControlResult> {
  const res = await jmapClient.jmap({
    using: SECURITY_USING,
    methodCalls: [['SenderControl/set', { accountId, emailId, ...req }, 'sc']],
  });
  return responseFor<SenderControlResult>(res, 'sc');
}

/** The armored PGP MESSAGE in an email body, or `null` when it isn't encrypted. */
function extractPgpArmor(email: Email | null): string | null {
  if (email === null) return null;
  for (const v of Object.values(email.bodyValues ?? {})) {
    const m = PGP_MESSAGE_RE.exec(v.value);
    if (m !== null) return m[0];
  }
  return null;
}

/** The plain-text body of an email (text parts joined; preview fallback). */
function plainTextOf(email: Email): string {
  const values = email.bodyValues ?? {};
  const text = (email.textBody ?? [])
    .map((p) => (p.partId !== null ? (values[p.partId]?.value ?? '') : ''))
    .join('\n');
  return text.length > 0 ? text : email.preview;
}

function addressList(addrs: EmailAddress[] | null): string {
  if (addrs === null || addrs.length === 0) return '';
  return addrs.map((a) => (a.name && a.name.length > 0 ? `${a.name} <${a.email}>` : a.email)).join(', ');
}

function ReaderToolbar(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [sweeping, setSweeping] = createSignal(false);
  const id = () => props.email.id;
  const sender = () => props.email.from?.[0]?.email ?? '';
  const pinned = () => props.email.pinned === true;

  return (
    <div class="reader__toolbar" role="toolbar" aria-label={t('mail-reader-actions')}>
      <button
        type="button"
        class={`btn btn--ghost ${a11y.focusable}`}
        aria-pressed={pinned()}
        onClick={() => void app.pinMessage(id(), !pinned())}
      >
        {pinned() ? t('mail-unpin') : t('mail-pin')}
      </button>
      <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => void app.archiveMessage(id())}>
        {t('mail-archive')}
      </button>
      <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => void app.trashMessage(id())}>
        {t('mail-delete')}
      </button>
      <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => void app.markSpam(id())}>
        {t('mail-spam')}
      </button>
      <button
        type="button"
        class={`btn btn--ghost ${a11y.focusable}`}
        data-testid="reader-export"
        onClick={() => void app.exportMessage()}
      >
        {t('mail-export')}
      </button>
      <Show when={sender() !== ''}>
        <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => setSweeping(true)}>
          {t('mail-sweep-sender')}
        </button>
      </Show>
      {/* Max-security opening switch (plan §7.2): drives the body render mode
          (full / no-media / plain-text) for this sender. Per-sender + global
          policy, admin floor clamps up. */}
      <MaxSecuritySwitch
        value={maxsec.effectiveMode(sender())}
        floor={maxsec.adminFloor()}
        onChange={(m) => maxsec.setSenderMode(sender(), m)}
      />
      <Show when={sweeping()}>
        <SweepDialog fromEmail={sender()} onClose={() => setSweeping(false)} />
      </Show>
    </div>
  );
}

/** Attachment thumbnails + the on-click viewer, rendered AROUND (never inside)
 *  the sandboxed message iframe. Each viewer keeps its own sandbox (§2.4). In a
 *  locked-down max-security mode attachments open ONLY through this re-encode
 *  preview jail (the AttachmentViewer sandbox), never as original bytes (§7.2). */
function AttachmentsPane(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [openItem, setOpenItem] = createSignal<StripItem | null>(null);

  const items = createMemo<StripItem[]>(() => {
    const parts = (props.email as { attachments?: AttachmentPart[] }).attachments ?? [];
    return parts
      .filter((a) => a.blobId !== null && a.blobId !== undefined && a.blobId !== '')
      .map((a) => ({
        blobId: a.blobId as string,
        name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : t('mail-attachment-unnamed'),
        mime: a.type.length > 0 ? a.type : 'application/octet-stream',
        size: a.size,
      }));
  });

  function blobUrlFor(item: StripItem): Promise<string> {
    const url = app.downloadUrl();
    const acct = app.accountId();
    if (url === null || acct === null) return Promise.resolve('');
    return fetchObjectUrl(
      buildDownloadUrl(url, { accountId: acct, blobId: item.blobId, name: item.name, mime: item.mime }),
    );
  }

  const [blobUrl] = createResource(openItem, (item) => blobUrlFor(item));

  return (
    <Show when={items().length > 0}>
      <section class="reader__attachments" aria-label={t('mail-attachments')} data-testid="reader-attachments">
        <ThumbnailStrip
          items={items()}
          selectedBlobId={openItem()?.blobId ?? ''}
          onSelect={(it) => setOpenItem(it)}
          resolveThumb={blobUrlFor}
        />
        <Show when={openItem()}>
          {(item) => (
            <div
              class="attachment-modal"
              role="dialog"
              aria-modal="true"
              aria-label={t('mail-attachment-open', { name: isolate(item().name) })}
              data-testid="attachment-viewer"
            >
              <div class="attachment-modal__bar">
                <span class="attachment-modal__name">{item().name}</span>
                <button
                  type="button"
                  class={`btn btn--ghost ${a11y.iconButton}`}
                  aria-label={t('mail-attachment-close')}
                  onClick={() => setOpenItem(null)}
                >
                  ✕
                </button>
              </div>
              <Show when={blobUrl()} fallback={<p class="attachment-modal__loading">{t('mail-attachment-loading')}</p>}>
                {(url) => (
                  <AttachmentViewer
                    part={{ partId: null, blobId: item().blobId, size: item().size, type: item().mime }}
                    blobUrl={url()}
                    mime={item().mime}
                    name={item().name}
                  />
                )}
              </Show>
            </div>
          )}
        </Show>
      </section>
    </Show>
  );
}

/** The unlock affordance for an encrypted (PGP) message: takes the passphrase,
 *  runs the client-side WASM decrypt in the crypto worker (§1.2/§1.3 — private
 *  material + plaintext never leave the client), and hands the plaintext + the
 *  signature verdict back to the reader. */
function DecryptPanel(props: {
  armor: string;
  senderAddress: string;
  onDecrypted: (content: { html?: string; text?: string }, signature: SignatureVerdict) => void;
}): JSX.Element {
  const app = useApp();
  const [passphrase, setPassphrase] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Ensure own keys are loaded so we can find the private bundle to decrypt with.
  onMount(() => {
    if (app.ownKeys().length === 0) void app.loadKeys();
  });

  /** Resolve the sender's armored PGP public key so the worker can VERIFY the
   *  embedded signature — mirrors how compose resolves recipient keys: first the
   *  already-loaded keyring (own + harvested/looked-up), then the same
   *  `CryptoKey/lookup` discovery (harvested/autocrypt/WKD/VKS).
   *
   *  This only picks a CANDIDATE key by the From address; the actual verdict is
   *  decided by real cryptography in the worker (`pgp::decrypt` → "verified" only
   *  for a genuine signature match, "invalid" for a wrong key, "none" when no key
   *  is supplied). So a false "verified" is impossible regardless of how the
   *  candidate is chosen — a wrong candidate yields an honest "invalid", and no
   *  candidate yields an honest "none". Matching therefore tolerates a From that
   *  is a bare local part (no domain — some transports, incl. the engine loopback,
   *  present addresses that way) by also comparing local parts. Returns undefined
   *  when the keyring holds no key relatable to the sender. */
  async function resolveSignerPublicKey(address: string): Promise<string | undefined> {
    if (address === '') return undefined;
    const norm = address.toLowerCase();
    const localOf = (a: string): string => a.split('@')[0] ?? '';
    const senderLocal = localOf(norm);
    const relatesToSender = (a: string): boolean => {
      const al = a.toLowerCase();
      return al === norm || (senderLocal !== '' && localOf(al) === senderLocal);
    };
    const usableArmor = (k: CryptoKey): string | undefined =>
      k.kind === 'pgp' && k.publicKeyArmored !== null && k.addresses.some(relatesToSender)
        ? (k.publicKeyArmored ?? undefined)
        : undefined;
    for (const k of app.keys()) {
      const armored = usableArmor(k);
      if (armored !== undefined) return armored;
    }
    try {
      const found = await app.lookupContactKey(address, ['harvested', 'autocrypt', 'wkd', 'vks']);
      for (const k of found) {
        const armored = usableArmor(k);
        if (armored !== undefined) return armored;
      }
    } catch {
      // Key discovery failure → no signer key → honest unverified verdict (never
      // a hard error, and never a false "verified").
    }
    return undefined;
  }

  async function decryptNow(): Promise<void> {
    setError(null);
    setBusy(true);
    try {
      const own = app.ownKeys().find(
        (k) => k.kind === 'pgp' && k.encryptedPrivateBackup !== null,
      );
      const bundle = own?.encryptedPrivateBackup ?? null;
      if (bundle === null) throw new Error(t('mail-decrypt-no-key'));
      const signerPublicKey = await resolveSignerPublicKey(props.senderAddress);
      const result = await getCryptoWorker().decrypt({
        kind: 'pgp',
        ciphertext: props.armor,
        encryptedPrivateBundle: bundle,
        passphrase: passphrase(),
        ...(signerPublicKey !== undefined ? { signerPublicKey } : {}),
      });
      // The worker sanitized any HTML plaintext IN-WORKER (§1.3): `plaintextHtml` is
      // already safe to render as HTML; `plaintextText` renders escaped.
      const content =
        result.plaintextHtml !== undefined
          ? { html: result.plaintextHtml }
          : { text: result.plaintextText ?? '' };
      props.onDecrypted(content, result.signature);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('mail-decrypt-failed'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class="reader__decrypt" data-testid="reader-decrypt" aria-label={t('mail-encrypted-region')}>
      <p class="reader__decrypt-note">{t('mail-encrypted-note')}</p>
      <form
        class="reader__decrypt-form"
        onSubmit={(e) => {
          e.preventDefault();
          void decryptNow();
        }}
      >
        <input
          type="password"
          class={`reader__decrypt-pass ${a11y.focusable}`}
          placeholder={t('mail-key-passphrase')}
          autocomplete="off"
          data-testid="decrypt-passphrase"
          value={passphrase()}
          onInput={(e) => setPassphrase(e.currentTarget.value)}
        />
        <button type="submit" class={`btn btn--primary ${a11y.focusable}`} data-testid="decrypt-submit" disabled={busy()}>
          {busy() ? t('mail-decrypting') : t('mail-decrypt')}
        </button>
      </form>
      <Show when={error()}>
        <p class="reader__decrypt-error" role="alert">
          {error()}
        </p>
      </Show>
    </section>
  );
}

/** Parse a model reply (comma/newline/semicolon-separated labels) into keyword
 *  suggestions for auto-tag. Defensive: bounded, de-duplicated, slug-safe. */
function parseTagSuggestions(text: string): TagSuggestion[] {
  const seen = new Set<string>();
  const out: TagSuggestion[] = [];
  for (const raw of text.split(/[,\n;]+/)) {
    const label = raw.trim().replace(/^[#\-*]\s*/, '');
    if (label.length === 0 || label.length > 40) continue;
    const keyword = label.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
    if (keyword.length === 0 || seen.has(keyword)) continue;
    seen.add(keyword);
    out.push({ keyword, label, confidence: 0.8 });
    if (out.length >= 6) break;
  }
  return out;
}

/** V7 auto-tag (§14.3): fetch model-suggested labels for the open message and render
 *  <AutoTag>. The invoke only fires when the `auto-tag` capability is granted; the
 *  component itself renders nothing without suggestions, so the reader is unchanged
 *  when Assist is disabled. Apply/revert route through the mail slice's keyword path. */
function AutoTagSection(props: { email: Email }): JSX.Element {
  const app = useApp();
  const [suggestions] = createResource(
    () => (app.assist.can('auto-tag') ? props.email.id : null),
    async (): Promise<TagSuggestion[]> => {
      const acct = app.accountId();
      if (acct === null) return [];
      const boxId = Object.keys(props.email.mailboxIds ?? {})[0] ?? '';
      const box = app.mailboxes().find((m) => m.id === boxId);
      const text = [props.email.subject ?? '', props.email.preview ?? '']
        .filter((s) => s.length > 0)
        .join('\n');
      try {
        const res = await app.assist.service.invoke({
          capability: 'auto-tag',
          prompt:
            'Suggest up to 6 short single-word labels for this message. Reply with a comma-separated list only.',
          context: [{ account: acct, folder: box?.name ?? 'Mail', text, kind: 'plain' }],
        });
        app.assist.recordDisclosure('auto-tag', res.disclosure);
        return parseTagSuggestions(res.text);
      } catch {
        return [];
      }
    },
  );

  return (
    <AutoTag
      config={app.assist.config()}
      messageId={props.email.id}
      suggestions={suggestions() ?? []}
      mode={app.assist.autoTagMode()}
      onModeChange={app.assist.setAutoTagMode}
      onApply={(kw) => void app.applyTag(props.email.id, kw)}
      onRevert={(kw) => void app.removeTag(props.email.id, kw)}
      onAudit={app.assist.recordTagAudit}
    />
  );
}

export function Reader(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('remote-images'));

  const emailId = (): string | null => app.openEmail()?.id ?? null;
  const sender = (): string => app.openEmail()?.from?.[0]?.email ?? '';
  const threadId = (): string | undefined => app.openEmail()?.threadId;

  // Server verdict for the open message (public facets: auth/received/attachments/
  // anomalies + a signature/encryption first pass). Re-fetched per message.
  const [serverVerdict] = createResource(
    (): { acct: string; id: string } | null => {
      const id = emailId();
      const acct = app.accountId();
      return id !== null && acct !== null ? { acct, id } : null;
    },
    (k) => fetchVerdict(k.acct, k.id),
  );

  // Client decrypt/verify results, merged over the server verdict (§1.2). Reset
  // whenever the open message changes.
  const [clientSig, setClientSig] = createSignal<SignatureVerdict | null>(null);
  // Decrypted body: `decryptedHtml` = HTML already sanitized IN-WORKER (§1.3),
  // rendered as sanitized HTML in the sandbox; `decryptedText` = non-HTML plaintext,
  // rendered escaped. Reset whenever the open message changes.
  const [decryptedHtml, setDecryptedHtml] = createSignal<string | null>(null);
  const [decryptedText, setDecryptedText] = createSignal<string | null>(null);
  createEffect(() => {
    emailId();
    setClientSig(null);
    setDecryptedHtml(null);
    setDecryptedText(null);
  });

  const verdict = createMemo<SecurityVerdict | null>(() => {
    const v = serverVerdict();
    if (v === undefined || v === null) return null;
    const sig = clientSig();
    if (sig === null) return v;
    // Overlay the client-computed signature + flag the client-decrypt path.
    return { ...v, signature: sig, encryption: { ...v.encryption, decryptsClientSide: true } };
  });

  const armor = (): string | null => extractPgpArmor(app.openEmail());
  const showDecrypt = (): boolean =>
    armor() !== null && decryptedHtml() === null && decryptedText() === null;

  // The message-body `srcdoc`, honoring the max-security mode + the decrypt path.
  // The DEFAULT (full-sanitized cleartext) path is the unchanged raw sanitized
  // fragment, so the existing sandbox/e2e contract is byte-identical.
  const bodySrcdoc = createMemo<string | null>(() => {
    const email = app.openEmail();
    if (email === null) return null;
    const mode = maxsec.effectiveMode(sender());
    // Decrypted E2EE body (§1.3): HTML was sanitized IN-WORKER (never round-trips to
    // the server sanitizer) and renders as sanitized HTML in the sandbox, honoring
    // the max-security mode; non-HTML plaintext renders escaped. Same no-scripts /
    // no-same-origin iframe as cleartext mail.
    const decHtml = decryptedHtml();
    if (decHtml !== null) {
      if (mode === 'plain-text') return bodyFrameDoc('plain-text', { text: decHtml });
      if (mode === 'sanitized-no-media') return bodyFrameDoc('sanitized-no-media', { html: decHtml });
      return bodyFrameDoc('full-sanitized', { html: decHtml });
    }
    const decText = decryptedText();
    if (decText !== null) return bodyFrameDoc('plain-text', { text: decText });
    if (mode === 'plain-text') return bodyFrameDoc('plain-text', { text: plainTextOf(email) });
    const html = app.sanitizedHtml();
    if (html === null) return null;
    // no-media: reuse the server-sanitized HTML but pin a media-free CSP so no
    // image/media loads (belt-and-braces). full: unchanged raw fragment.
    if (mode === 'sanitized-no-media') return bodyFrameDoc('sanitized-no-media', { html });
    return html;
  });

  async function onSenderControl(req: SenderControlRequest): Promise<SenderControlResult> {
    const acct = app.accountId();
    const id = emailId();
    if (acct === null || id === null) return defaultSenderControl(req);
    try {
      return await dispatchSenderControl(acct, id, req);
    } catch {
      return { updated: false };
    }
  }

  // ── Remote-content (image-grant) bar (t16 §S8/S9) ──────────────────────────
  // What the sanitizer blocked in the CURRENT body (derived from the sanitized
  // string, no round-trip). Only meaningful in the full-sanitized mode — in the
  // no-media / plain-text modes images are stripped regardless of any grant, so
  // the bar is hidden there rather than offering an action that can't take effect.
  const blockedReport = createMemo(() => analyzeBlockedContent(app.sanitizedHtml()));
  const fullMode = (): boolean => maxsec.effectiveMode(sender()) === 'full-sanitized';

  // Active grants for the account, so the bar can show "turn off" (revoke) once a
  // grant covers the open message. Resilient: a missing/failing endpoint (e.g. e6
  // not yet deployed) yields no grants, so the blocked state still renders.
  const [grants, { refetch: refetchGrants }] = createResource(
    (): { acct: string; id: string } | null => {
      const id = emailId();
      const acct = app.accountId();
      return id !== null && acct !== null ? { acct, id } : null;
    },
    async (k) => {
      try {
        return await remoteImages.listGrants(k.acct);
      } catch {
        return [];
      }
    },
  );

  const activeGrant = createMemo(() => {
    const id = emailId();
    if (id === null) return null;
    return coveringGrant(grants() ?? [], { emailId: id, sender: sender() });
  });

  const showRemoteBar = (): boolean =>
    fullMode() && (hasBlockedContent(blockedReport()) || activeGrant() !== null);

  async function reloadOpenMessage(): Promise<void> {
    const id = emailId();
    if (id !== null) await app.openMessage(id);
  }

  async function onRemoteGrant(scope: GrantScope): Promise<void> {
    const acct = app.accountId();
    if (acct === null) return;
    await remoteImages.grant(acct, scope);
    void refetchGrants();
    // Re-fetch + re-sanitize so the now-permitted images load through the proxy.
    await reloadOpenMessage();
  }

  async function onRemoteRevoke(scope: GrantScope): Promise<void> {
    const acct = app.accountId();
    if (acct === null) return;
    await remoteImages.revoke(acct, scope);
    void refetchGrants();
    await reloadOpenMessage();
  }

  return (
    <section
      class="reader"
      classList={{ 'reader--open': app.openEmail() !== null }}
      aria-label={t('mail-reader-label')}
    >
      <Show when={app.openEmail()} fallback={<p class="reader__empty">{t('mail-reader-empty')}</p>}>
        {(email) => (
          <>
            <header class="reader__header">
              <button type="button" class={`btn btn--ghost reader__close ${a11y.focusable}`} onClick={() => app.closeMessage()}>
                ← {t('mail-back')}
              </button>
              <h2 class="reader__subject">{email().subject ?? t('mail-no-subject')}</h2>
              <div class="reader__meta">
                <span>{t('mail-reader-from', { addr: isolate(addressList(email().from)) })}</span>
                <span>{t('mail-reader-to', { addr: isolate(addressList(email().to)) })}</span>
              </div>
              {/* Security chip → expandable panel (plan §7.3): server verdict merged
                  with the client decrypt/verify result. */}
              <Show when={verdict()}>
                {(v) => {
                  const tid = threadId();
                  return (
                    <SecurityPanel
                      verdict={v()}
                      senderAddress={sender()}
                      {...(tid !== undefined ? { threadId: tid } : {})}
                      onSenderControl={onSenderControl}
                    />
                  );
                }}
              </Show>
              <ReaderToolbar email={email()} />
              <AutoTagSection email={email()} />
            </header>
            <AttachmentsPane email={email()} />
            {/* Remote-content bar (§S8/S9): blocked-image/tracker count + the
                4 grant scopes, above the body it governs. */}
            <Show when={showRemoteBar()}>
              <RemoteContentBar
                emailId={email().id}
                sender={sender()}
                report={blockedReport()}
                activeGrant={activeGrant()}
                onGrant={onRemoteGrant}
                onRevoke={onRemoteRevoke}
              />
            </Show>
            <Show
              when={showDecrypt()}
              fallback={
                <Show
                  when={bodySrcdoc() !== null}
                  fallback={<p class="reader__empty">{app.readLoading() ? t('mail-sanitizing') : t('mail-no-content')}</p>}
                >
                  {/*
                    Security-critical (SPEC §7.2): the message body renders in a
                    sandboxed iframe whose sandbox attribute intentionally omits
                    allow-scripts AND allow-same-origin, so even if the sanitizer
                    missed something, scripts cannot run and the frame has an
                    opaque origin with no access to the parent.
                  */}
                  <iframe
                    class="reader__frame"
                    title={t('mail-message-body')}
                    sandbox=""
                    srcdoc={bodySrcdoc() ?? ''}
                  />
                </Show>
              }
            >
              <DecryptPanel
                armor={armor() ?? ''}
                senderAddress={sender()}
                onDecrypted={(content, signature) => {
                  if (content.html !== undefined) setDecryptedHtml(content.html);
                  else setDecryptedText(content.text ?? '');
                  setClientSig(signature);
                }}
              />
            </Show>
          </>
        )}
      </Show>
    </section>
  );
}
