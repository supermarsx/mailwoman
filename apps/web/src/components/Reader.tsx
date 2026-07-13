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
import { SweepDialog } from './SweepDialog.tsx';
import { ThumbnailStrip, type StripItem } from '../viewers/ThumbnailStrip.tsx';
import { AttachmentViewer } from '../viewers/AttachmentViewer.tsx';
import { buildDownloadUrl, fetchObjectUrl, type AttachmentPart } from '../viewers/attachments.ts';
import { SecurityPanel } from './SecurityPanel.tsx';
import type { SenderControlRequest, SenderControlResult } from './security/model.ts';
import { defaultSenderControl } from './security/model.ts';
import { MaxSecuritySwitch } from '../viewers/MaxSecuritySwitch.tsx';
import { createMaxSecurityStore } from '../viewers/max-security.ts';
import { bodyFrameDoc } from '../viewers/sandbox.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { createClient } from '../api/client.ts';
import { responseFor } from '../api/jmap.ts';
import { CAP_CORE } from '../api/jmap-types.ts';
import { CAP_CRYPTO, CAP_SECURITY } from '../api/crypto-types.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';
import type { SecurityVerdict, SignatureVerdict } from '../api/security-types.ts';

// The crypto/security JMAP surface (`SecurityVerdict/get`, `SenderControl/set`)
// is not exposed on `AppState`, so this component drives it over its own
// same-origin, cookie-authed client (stateless — hits the same session as the
// store's client). The max-security policy is an app-singleton (per-sender +
// global, persisted in localStorage) — plan §2.5.
const jmapClient = createClient();
const maxsec = createMaxSecurityStore();

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
    <div class="reader__toolbar" role="toolbar" aria-label="Message actions">
      <button
        type="button"
        class="btn btn--ghost"
        aria-pressed={pinned()}
        onClick={() => void app.pinMessage(id(), !pinned())}
      >
        {pinned() ? 'Unpin' : 'Pin'}
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.archiveMessage(id())}>
        Archive
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.trashMessage(id())}>
        Delete
      </button>
      <button type="button" class="btn btn--ghost" onClick={() => void app.markSpam(id())}>
        Spam
      </button>
      <button
        type="button"
        class="btn btn--ghost"
        data-testid="reader-export"
        onClick={() => void app.exportMessage()}
      >
        Export
      </button>
      <Show when={sender() !== ''}>
        <button type="button" class="btn btn--ghost" onClick={() => setSweeping(true)}>
          Sweep sender
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
        name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : '(unnamed)',
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
      <section class="reader__attachments" aria-label="Attachments" data-testid="reader-attachments">
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
              aria-label={`Attachment ${item().name}`}
              data-testid="attachment-viewer"
            >
              <div class="attachment-modal__bar">
                <span class="attachment-modal__name">{item().name}</span>
                <button
                  type="button"
                  class="btn btn--ghost"
                  aria-label="Close attachment"
                  onClick={() => setOpenItem(null)}
                >
                  ✕
                </button>
              </div>
              <Show when={blobUrl()} fallback={<p class="attachment-modal__loading">Loading attachment…</p>}>
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
  onDecrypted: (text: string, signature: SignatureVerdict) => void;
}): JSX.Element {
  const app = useApp();
  const [passphrase, setPassphrase] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Ensure own keys are loaded so we can find the private bundle to decrypt with.
  onMount(() => {
    if (app.ownKeys().length === 0) void app.loadKeys();
  });

  async function decryptNow(): Promise<void> {
    setError(null);
    setBusy(true);
    try {
      const own = app.ownKeys().find(
        (k) => k.kind === 'pgp' && k.encryptedPrivateBackup !== null,
      );
      const bundle = own?.encryptedPrivateBackup ?? null;
      if (bundle === null) throw new Error('No private key is available to decrypt this message.');
      const result = await getCryptoWorker().decrypt({
        kind: 'pgp',
        ciphertext: props.armor,
        encryptedPrivateBundle: bundle,
        passphrase: passphrase(),
      });
      props.onDecrypted(result.plaintextText ?? result.plaintextHtml ?? '', result.signature);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Decryption failed');
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class="reader__decrypt" data-testid="reader-decrypt" aria-label="Encrypted message">
      <p class="reader__decrypt-note">🔒 This message is end-to-end encrypted. Unlock it on this device to read it.</p>
      <form
        class="reader__decrypt-form"
        onSubmit={(e) => {
          e.preventDefault();
          void decryptNow();
        }}
      >
        <input
          type="password"
          class="reader__decrypt-pass"
          placeholder="Key passphrase"
          autocomplete="off"
          data-testid="decrypt-passphrase"
          value={passphrase()}
          onInput={(e) => setPassphrase(e.currentTarget.value)}
        />
        <button type="submit" class="btn btn--primary" data-testid="decrypt-submit" disabled={busy()}>
          {busy() ? 'Decrypting…' : 'Decrypt'}
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

export function Reader(): JSX.Element {
  const app = useApp();

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
  const [decryptedText, setDecryptedText] = createSignal<string | null>(null);
  createEffect(() => {
    emailId();
    setClientSig(null);
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
  const showDecrypt = (): boolean => armor() !== null && decryptedText() === null;

  // The message-body `srcdoc`, honoring the max-security mode + the decrypt path.
  // The DEFAULT (full-sanitized cleartext) path is the unchanged raw sanitized
  // fragment, so the existing sandbox/e2e contract is byte-identical.
  const bodySrcdoc = createMemo<string | null>(() => {
    const email = app.openEmail();
    if (email === null) return null;
    const dec = decryptedText();
    // Decrypted E2EE plaintext renders as ESCAPED TEXT in the sandbox — it never
    // round-trips to the server sanitizer (§1.3) and carries no HTML surface.
    if (dec !== null) return bodyFrameDoc('plain-text', { text: dec });
    const mode = maxsec.effectiveMode(sender());
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

  return (
    <section class="reader" aria-label="Message">
      <Show when={app.openEmail()} fallback={<p class="reader__empty">Select a message to read</p>}>
        {(email) => (
          <>
            <header class="reader__header">
              <button type="button" class="btn btn--ghost reader__close" onClick={() => app.closeMessage()}>
                ← Back
              </button>
              <h2 class="reader__subject">{email().subject ?? '(no subject)'}</h2>
              <div class="reader__meta">
                <span>From: {addressList(email().from)}</span>
                <span>To: {addressList(email().to)}</span>
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
            </header>
            <AttachmentsPane email={email()} />
            <Show
              when={showDecrypt()}
              fallback={
                <Show
                  when={bodySrcdoc() !== null}
                  fallback={<p class="reader__empty">{app.readLoading() ? 'Sanitizing…' : 'No content'}</p>}
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
                    title="Message body"
                    sandbox=""
                    srcdoc={bodySrcdoc() ?? ''}
                  />
                </Show>
              }
            >
              <DecryptPanel
                armor={armor() ?? ''}
                onDecrypted={(text, signature) => {
                  setDecryptedText(text);
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
