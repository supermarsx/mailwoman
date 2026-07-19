import {
  createEffect,
  createMemo,
  createSignal,
  lazy,
  For,
  Show,
  Suspense,
  onMount,
  onCleanup,
  type JSX,
} from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import type { RichTextApi } from './compose/RichTextEditor.tsx';
import {
  SignaturePicker,
  SendOptions,
  RecallPanel,
  DraftsDrawer,
  DEFAULT_SEND_OPTIONS,
  type ComposeSignature,
  type SendOptionsState,
} from './compose/ComposerExtras.tsx';
import {
  listDrafts,
  saveDraft,
  deleteDraft,
  newDraftId,
  type StoredDraft,
} from './compose/drafts-store.ts';

// The rich-text editor pulls in ProseMirror (MIT, self-hosted). Loaded lazily so
// those libraries land in their own chunk and never inflate the login→inbox
// entry the size gate measures — the composer is user-triggered, so the small
// deferred load is invisible in practice. A plain textarea backs the Suspense
// fallback, so the Body field is usable (and its label present) before the
// chunk resolves.
const RichTextEditor = lazy(() => import('./compose/RichTextEditor.tsx'));
import {
  createContactAutocomplete,
  type ContactSuggestion,
} from '../modules/contacts/autocomplete.ts';
import { ComposeCrypto, type ComposeCryptoState } from './compose-crypto.tsx';
import {
  clearSignBody,
  createJmapDlpScan,
  createJmapKeyLookup,
  type DlpScanFn,
  type KeyLookupFn,
  type SigningSession,
} from './compose/crypto-jmap.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { uploadBlob } from '../api/jmap.ts';
import { CAP_CORE } from '../api/jmap-types.ts';
// V7 last-mile mailbox integration (plan §2.7/§14, e14b). All ADDITIVE: each block
// is gated so a deployment with no directory / disabled Assist / no Nextcloud sees
// the exact same composer as before.
import { DirectorySearch, GroupExpand, type GalEntry } from '../modules/directory/index.ts';
import { ComposerTools, Dictation } from '../modules/assist/index.ts';
import { NextcloudAttach, type AttachedFile } from '../modules/nextcloud/index.ts';

// The crypto/DLP JMAP surface (`CryptoKey/lookup`, `Dlp/scan`) is not on
// `AppState`; drive it over a dedicated client that hits the same session as the
// store's client (browser: same-origin cookie; native shell: configured base +
// bearer — plan §2.2/§2.5).
const jmapClient = createConfiguredClient();

/** Split the raw To field into recipient tokens (the banner is live as you type). */
function splitRecipients(raw: string): string[] {
  return raw
    .split(/[,;]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

// Compose (plan §1.5, §2.1): grown with an identity/signature picker (multiple
// from-addresses, server-pulled allowed-froms) and send-later. The core To /
// Subject / Body fields + the Send button keep their exact labels so the mock +
// engine e2e specs still drive it. e10 adds contacts recipient autocomplete to
// the To field — a surgical addition over e7's `createContactAutocomplete`.

/** The recipient token currently being typed: the text after the last separator. */
function tokenBoundary(value: string): number {
  return Math.max(value.lastIndexOf(','), value.lastIndexOf(';'));
}

export function Compose(props: { onClose: () => void }): JSX.Element {
  const app = useApp();
  const [to, setTo] = createSignal('');
  const [subject, setSubject] = createSignal('');
  // `body` stays the PLAIN-TEXT source of truth (crypto/DLP/dictation read it).
  // `bodyHtml` carries the rich-text HTML for the normal send path; the rich
  // editor keeps both in sync. `richMode` toggles the ProseMirror editor vs a
  // plain-text / format=flowed textarea; the toggle round-trips the text.
  const [body, setBody] = createSignal('');
  const [bodyHtml, setBodyHtml] = createSignal('');
  const [richMode, setRichMode] = createSignal(true);
  const [editorApi, setEditorApi] = createSignal<RichTextApi | null>(null);
  // W11 send-option toggles (read receipt + open-tracking pixel).
  const [sendOptions, setSendOptions] = createSignal<SendOptionsState>(DEFAULT_SEND_OPTIONS);
  // W9 drafts drawer + W10 recall panel visibility, and the loaded draft list.
  const [draftsOpen, setDraftsOpen] = createSignal(false);
  const [recallOpen, setRecallOpen] = createSignal(false);
  const [drafts, setDrafts] = createSignal<StoredDraft[]>([]);
  // Stable id for THIS composer's auto-saved draft (W9).
  const draftId = newDraftId();
  const [identityId, setIdentityId] = createSignal<string>('');
  const [sendAt, setSendAt] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [acOpen, setAcOpen] = createSignal(false);
  // V7 GAL (plan §2.7): the in-progress recipient token also drives a directory
  // autocomplete as an ADDITIONAL source beside contacts. `pickedGroup` holds a
  // distribution group the sender may expand-before-send into its leaf recipients.
  const [galToken, setGalToken] = createSignal('');
  const [pickedGroup, setPickedGroup] = createSignal<GalEntry | null>(null);
  // V7 Nextcloud attach (plan §18.4): materialised attachments + the picker toggle.
  const [attachments, setAttachments] = createSignal<AttachedFile[]>([]);
  const [ncOpen, setNcOpen] = createSignal(false);
  // New-file blob upload (26.15 §1): the per-account upload endpoint + size limit
  // are pulled from the JMAP session; a local file is POSTed to `uploadUrl` and
  // the returned blob folds into the SAME `attachments` list as the Nextcloud
  // path. `uploadUrl` null ⇒ the session probe hasn't landed (picker disabled).
  const [uploadUrl, setUploadUrl] = createSignal<string | null>(null);
  const [maxUploadSize, setMaxUploadSize] = createSignal<number>(50_000_000);
  const [uploading, setUploading] = createSignal(false);
  const [attachError, setAttachError] = createSignal<string | null>(null);
  // Crypto/DLP state reported up by <ComposeCrypto> (encrypt/sign toggles, the
  // E2EE/TLS/mixed capability, the DLP `canSend` gate, and the WASM-encrypted
  // draft) — plan §2.5.
  const [cryptoState, setCryptoState] = createSignal<ComposeCryptoState | null>(null);
  // Signing session (plan §2.5, decision flag 2): the signing key is unlocked
  // ONCE per composer via the passphrase prompt below (mirroring
  // Reader.tsx::decryptNow's unlock), then cached so encrypt+sign and sign-only
  // sends reuse it without re-prompting. `signingKeyRef` is handed to
  // <ComposeCrypto> to fold a signature into its encrypt call; the panel opens
  // on demand when `sign` is switched on while still locked.
  const [signingSession, setSigningSession] = createSignal<SigningSession | null>(null);
  const [unlockOpen, setUnlockOpen] = createSignal(false);
  const [unlockPass, setUnlockPass] = createSignal('');
  const [unlockError, setUnlockError] = createSignal<string | null>(null);
  const [unlocking, setUnlocking] = createSignal(false);

  // Client-backed key lookup + DLP scan for <ComposeCrypto> (real engine). Read
  // the account id at call time (it is null until the session loads). A lookup /
  // scan failure (offline, or no crypto capability) degrades gracefully — the
  // banner falls back to TLS and no DLP verdict blocks — rather than crashing
  // compose.
  const lookupKeys: KeyLookupFn = async (address) => {
    const acct = app.accountId();
    if (acct === null) return [];
    try {
      return await createJmapKeyLookup(jmapClient, acct)(address);
    } catch {
      return [];
    }
  };
  const scanDlp: DlpScanFn = async (draft) => {
    const acct = app.accountId();
    if (acct === null) return [];
    try {
      return await createJmapDlpScan(jmapClient, acct)(draft);
    } catch {
      return [];
    }
  };

  // Recipient autocomplete over the loaded contacts (plan §2.2 / e7 seam). The
  // ranking is client-side over `app.contacts()`; we load contacts on open so a
  // fresh session can still complete. Load failures are non-fatal (empty list).
  const contactAc = createContactAutocomplete(() => app.contacts());

  // Dialog focus management (self-contained per t8-e1; no import from the
  // e3-owned a11y primitives). On open: pull the mail catalog, remember the
  // trigger, and move focus into the composer. On close: restore focus. Escape
  // closes; Tab is trapped inside the dialog.
  let backdropEl: HTMLDivElement | undefined;
  let toInputEl: HTMLInputElement | undefined;
  let previouslyFocused: HTMLElement | null = null;

  function focusableIn(root: HTMLElement): HTMLElement[] {
    return Array.from(
      root.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((el) => el.offsetParent !== null || el === document.activeElement);
  }

  function onDialogKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      props.onClose();
      return;
    }
    if (e.key !== 'Tab' || backdropEl === undefined) return;
    const items = focusableIn(backdropEl);
    if (items.length === 0) return;
    const first = items[0]!;
    const last = items[items.length - 1]!;
    const activeEl = document.activeElement as HTMLElement | null;
    if (e.shiftKey && activeEl === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && activeEl === last) {
      e.preventDefault();
      first.focus();
    }
  }

  onMount(() => {
    void loadCatalog('mail');
    previouslyFocused = document.activeElement as HTMLElement | null;
    toInputEl?.focus();
    void app.loadIdentities();
    void app.loadContacts().catch(() => undefined);
    // Probe the optional V7 backends ONCE (idempotent, silent on failure): a
    // NotConfigured directory / absent Nextcloud leaves `enabled` false so their
    // affordances never mount and the composer is byte-unchanged.
    void app.directory.ensureEnabled();
    void app.nextcloud.ensureEnabled();
    // Pull the session's upload contract (uploadUrl template + maxSizeUpload) so
    // the local-file picker can POST bytes to the per-account endpoint and guard
    // the size client-side. A failed probe (offline / no session) simply leaves
    // the picker disabled; the rest of the composer is unchanged.
    void jmapClient
      .session()
      .then((s) => {
        setUploadUrl(s.uploadUrl);
        const core = s.capabilities[CAP_CORE] as { maxSizeUpload?: number } | undefined;
        if (core?.maxSizeUpload !== undefined && core.maxSizeUpload > 0) {
          setMaxUploadSize(core.maxSizeUpload);
        }
      })
      .catch(() => undefined);
  });

  onCleanup(() => {
    previouslyFocused?.focus();
    // Drop the cached signing key from the worker session when the composer closes
    // (the worker zeroizes the unlocked private key for this ref).
    const s = signingSession();
    if (s !== null) void getCryptoWorker().lockKey({ keyRef: s.keyRef });
  });

  /** Unlock the sending key ONCE per composer session (decision flag 2): find the
   *  own PGP private bundle (as Reader.tsx::decryptNow does), unlock it in the
   *  worker to get a session keyRef, and cache the ref + bundle + passphrase so
   *  signed sends reuse it without re-prompting. */
  async function unlockSigningKey(e: Event): Promise<void> {
    e.preventDefault();
    setUnlockError(null);
    setUnlocking(true);
    try {
      if (app.ownKeys().length === 0) await app.loadKeys();
      const own = app.ownKeys().find((k) => k.kind === 'pgp' && k.encryptedPrivateBackup !== null);
      const bundle = own?.encryptedPrivateBackup ?? null;
      if (bundle === null) throw new Error(t('mail-compose-sign-no-key'));
      const keyRef = await getCryptoWorker().unlockKey({
        encryptedPrivateBundle: bundle,
        passphrase: unlockPass(),
      });
      setSigningSession({ keyRef, bundle, passphrase: unlockPass() });
      setUnlockPass('');
      setUnlockOpen(false);
    } catch (err) {
      setUnlockError(err instanceof Error ? err.message : t('mail-compose-sign-unlock-failed'));
    } finally {
      setUnlocking(false);
    }
  }

  const identity = createMemo(() => app.identities().find((i) => i.id === identityId()) ?? null);

  /** Plain text → the same minimal HTML the plain-text send path has always
   *  produced (escaped, newlines as `<br>`). Used to seed the rich editor when
   *  switching plain → rich, so the typed text carries over. No ProseMirror here
   *  (that would drag the lazy editor's libraries onto the entry chunk). */
  function plainToHtml(text: string): string {
    return `<p>${escapeHtml(text).replace(/\n/g, '<br>')}</p>`;
  }

  /** The rich editor reports HTML (for the send) + a plain-text projection (for
   *  crypto/DLP/dictation, which read `body()`). */
  function onEditorChange(html: string, text: string): void {
    setBodyHtml(html);
    setBody(text);
  }

  /** Toggle the body between the rich editor and a plain-text / format=flowed
   *  textarea. Rich → plain just reveals the already-synced text; plain → rich
   *  re-seeds the editor from that text so the content round-trips. */
  function toggleFormat(): void {
    if (richMode()) {
      setRichMode(false);
    } else {
      setBodyHtml(plainToHtml(body()));
      setRichMode(true);
    }
  }

  // W12: signatures the picker offers, derived from the sending identities that
  // carry one. A signatures CRUD backend (e15) can supply the same shape later.
  const signatures = createMemo<ComposeSignature[]>(() =>
    app
      .identities()
      .map((id) => {
        const text = (id.signatureText ?? '').trim();
        const htmlText = (id.signatureHtml ?? '').replace(/<[^>]*>/g, '').trim();
        const plain = text !== '' ? text : htmlText;
        return { id: id.id, name: id.name, text: plain, html: id.signatureHtml };
      })
      .filter((s) => s.text !== '' || (s.html ?? '') !== ''),
  );

  /** Insert a chosen signature (W12). In rich mode it appends as HTML through the
   *  editor handle (keeping existing formatting); otherwise it appends its text
   *  to the plain body. */
  function insertSignature(sig: ComposeSignature): void {
    const api = editorApi();
    if (richMode() && api !== null) {
      api.appendHtml(sig.html !== null && sig.html !== '' ? sig.html : plainToHtml(sig.text));
    } else {
      setBody((cur) => (cur.trim() !== '' ? `${cur}\n\n-- \n${sig.text}` : sig.text));
    }
  }

  /** Resume a locally auto-saved draft (W9) into this composer. */
  function resumeDraft(d: StoredDraft): void {
    setTo(d.to);
    setSubject(d.subject);
    setBody(d.bodyText);
    setBodyHtml(d.bodyHtml);
    const api = editorApi();
    if (richMode() && api !== null) api.setHtml(d.bodyHtml);
    setDraftsOpen(false);
  }

  /** Discard a stored draft and refresh the list (W9). */
  function discardDraft(id: string): void {
    deleteDraft(id);
    setDrafts(listDrafts());
  }

  /** Open the Drafts drawer, refreshing the list from storage first (W9). */
  function openDrafts(): void {
    setDrafts(listDrafts());
    setDraftsOpen(true);
  }

  /** Open the recall panel, refreshing the server-held submission queue (W10). */
  function openRecall(): void {
    void app.refreshOutbox();
    setRecallOpen(true);
  }

  /** Recall (cancel) a still-holding / scheduled submission before it dispatches. */
  function recallSubmission(id: string): void {
    void app.cancelOutbox(id).then(() => app.refreshOutbox());
  }

  // W9 auto-save: debounce a snapshot of the composition to local storage so a
  // closed / refreshed composer can be resumed. Empty compositions are skipped
  // (see `draftHasContent`).
  onMount(() => setDrafts(listDrafts()));
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(() => {
    const snapshot: StoredDraft = {
      id: draftId,
      to: to(),
      subject: subject(),
      bodyHtml: richMode() ? bodyHtml() : plainToHtml(body()),
      bodyText: body(),
      savedAt: Date.now(),
    };
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => saveDraft(snapshot), 800);
  });
  onCleanup(() => clearTimeout(saveTimer));

  function onToInput(value: string): void {
    setTo(value);
    const token = value.slice(tokenBoundary(value) + 1).trim();
    contactAc.setQuery(token);
    setGalToken(token);
    setAcOpen(token.length > 0);
  }

  /** Replace the in-progress recipient token with a resolved address (`, `-joined). */
  function insertRecipient(address: string): void {
    const value = to();
    const cut = tokenBoundary(value);
    const head = cut >= 0 ? `${value.slice(0, cut + 1)} ` : '';
    setTo(`${head}${address}, `);
    contactAc.reset();
    setGalToken('');
    setAcOpen(false);
  }

  /** Replace the in-progress recipient token with the picked contact. */
  function pickSuggestion(s: ContactSuggestion): void {
    insertRecipient(s.display);
  }

  /** Pick a GAL entry (plan §2.7). A person is inserted as a recipient; a
   *  distribution group is inserted AND offered for expand-before-send. */
  function pickGalEntry(entry: GalEntry): void {
    insertRecipient(entry.mail);
    setPickedGroup(entry.isGroup ? entry : null);
  }

  /** Expand-before-send: swap the group's address for its concrete leaf members. */
  function expandGroupInTo(group: GalEntry, members: GalEntry[]): void {
    const leaves = members.map((m) => m.mail).join(', ');
    // Replace the group's own address token with the flattened leaves.
    setTo((cur) => cur.replace(group.mail, leaves));
    setPickedGroup(null);
  }

  /** Upload one or more locally-picked files to the account's JMAP upload
   *  endpoint and fold each returned blob into the SAME attachment list the
   *  Nextcloud path uses (so the send payload carries `{blobId,name,type,size}`
   *  unchanged). An over-`maxSizeUpload` file is refused BEFORE upload with a
   *  concrete size message; a failed upload reports the file by name and leaves
   *  the rest of the selection intact. */
  async function onFilesPicked(fileList: FileList | null): Promise<void> {
    if (fileList === null || fileList.length === 0) return;
    const url = uploadUrl();
    const acct = app.accountId();
    if (url === null || acct === null) {
      setAttachError(t('mail-compose-upload-unavailable'));
      return;
    }
    setAttachError(null);
    const max = maxUploadSize();
    const files = Array.from(fileList);
    setUploading(true);
    try {
      for (const file of files) {
        if (file.size > max) {
          setAttachError(
            t('mail-compose-upload-too-large', {
              name: isolate(file.name),
              size: megabytes(file.size),
              max: megabytes(max),
            }),
          );
          continue;
        }
        try {
          const up = await uploadBlob(url, acct, file);
          setAttachments((cur) => [
            ...cur,
            { name: file.name, blobId: up.blobId, size: up.size, contentType: up.type },
          ]);
        } catch {
          setAttachError(t('mail-compose-upload-failed', { name: isolate(file.name) }));
        }
      }
    } finally {
      setUploading(false);
    }
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    const cs = cryptoState();
    // DLP gate (plan §1.8 / §2.2): a `block` verdict stops the send before it
    // reaches the engine. The blocking rule is already surfaced inline by
    // <ComposeCrypto>; here we enforce the send gate.
    if (cs !== null && !cs.canSend) {
      setError(t('mail-compose-dlp-blocked'));
      return;
    }
    // Signing gate (plan §2.5): a signed send — whether folded into encrypt or a
    // clear-signed sign-only send — needs the signing key unlocked first. Prompt
    // for the passphrase once per composer session; never send silently unsigned.
    if (cs !== null && cs.sign && signingSession() === null) {
      setUnlockOpen(true);
      setError(t('mail-compose-sign-unlock-required'));
      return;
    }
    setBusy(true);
    try {
      // Encrypt-on-send (plan §2.5): when encryption is on the worker has already
      // produced the armored ciphertext (signed in-place when `sign` is on, via
      // `signWithKeyRef`); send it as the body so the recipient decrypts it
      // client-side. Protected-subject replaces the visible subject with a
      // placeholder.
      const enc =
        cs !== null && cs.encrypt && cs.encryptedDraft !== null ? cs.encryptedDraft : null;
      // Sign-only (plan §2.5): a signature requested WITHOUT encryption → clear-sign
      // the body (inline PGP SIGNED MESSAGE) so the recipient can verify it's from
      // us while the content stays readable.
      const session = signingSession();
      const signOnly = enc === null && cs !== null && cs.sign && session !== null;
      let htmlBody: string;
      if (enc !== null) {
        htmlBody = enc.armoredCiphertext;
      } else if (signOnly && session !== null) {
        htmlBody = await clearSignBody(getCryptoWorker(), session, body());
      } else if (richMode()) {
        // W1: the rich editor's serialized HTML feeds the SAME send payload.
        htmlBody = bodyHtml();
      } else {
        // Plain-text / format=flowed: the original escaped-body behavior.
        htmlBody = `<p>${escapeHtml(body()).replace(/\n/g, '<br>')}</p>`;
      }
      // W11: an opt-in open-tracking pixel, only on a normal (not encrypted,
      // not clear-signed) send. Off by default; the toggle copy is explicit
      // that it embeds a remote image.
      if (enc === null && !signOnly && sendOptions().trackingPixel) {
        htmlBody += `<img src="/api/track/open/${encodeURIComponent(draftId)}.gif" width="1" height="1" alt="">`;
      }
      const subjectToSend =
        enc !== null && cs !== null && cs.protectSubject && enc.encryptedSubjectApplied
          ? t('mail-compose-encrypted-subject')
          : subject();
      const attached = attachments();
      await app.sendMessage({
        to: to(),
        subject: subjectToSend,
        htmlBody,
        identity: identity(),
        // datetime-local yields a local wall-clock string; convert to a UTC ISO.
        sendAt: sendAt() !== '' ? new Date(sendAt()).toISOString() : null,
        // V7 (§18.4): Nextcloud-materialised blob attachments (empty ⇒ omitted).
        ...(attached.length > 0
          ? {
              attachments: attached.map((a) => ({
                blobId: a.blobId,
                name: a.name,
                type: a.contentType ?? 'application/octet-stream',
                ...(a.size > 0 ? { size: a.size } : {}),
              })),
            }
          : {}),
      });
      // W9: the composition was sent — drop its auto-saved draft.
      deleteDraft(draftId);
      props.onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('mail-compose-send-failed'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div
      class="compose__backdrop"
      role="dialog"
      aria-modal="true"
      aria-label={t('mail-compose-label')}
      ref={backdropEl}
      onKeyDown={onDialogKeyDown}
    >
      <form class="compose" onSubmit={(e) => void onSubmit(e)}>
        <header class="compose__header">
          <h2>{t('mail-compose-title')}</h2>
          <div class="compose__header-actions">
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="open-drafts"
              aria-expanded={draftsOpen()}
              onClick={() => (draftsOpen() ? setDraftsOpen(false) : openDrafts())}
            >
              {t('mail-compose-drafts')}
            </button>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="open-recall"
              aria-expanded={recallOpen()}
              onClick={() => (recallOpen() ? setRecallOpen(false) : openRecall())}
            >
              {t('mail-compose-recall')}
            </button>
            <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t('mail-compose-close')} onClick={() => props.onClose()}>
              ✕
            </button>
          </div>
        </header>

        <DraftsDrawer
          open={draftsOpen()}
          drafts={drafts}
          onResume={resumeDraft}
          onDelete={discardDraft}
          onClose={() => setDraftsOpen(false)}
        />
        <Show when={recallOpen()}>
          <RecallPanel submissions={app.cancelableOutbox} onRecall={recallSubmission} />
        </Show>

        <Show when={app.identities().length > 0}>
          <label class="field">
            <span>{t('mail-compose-from')}</span>
            <select value={identityId()} onChange={(e) => setIdentityId(e.currentTarget.value)}>
              <option value="">{t('mail-compose-from-default')}</option>
              <For each={app.identities()}>
                {(id) => (
                  <option value={id.id}>
                    {isolate(id.name)} &lt;{id.email}&gt;
                  </option>
                )}
              </For>
            </select>
          </label>
        </Show>

        <label class="field compose__to">
          <span>{t('mail-compose-to')}</span>
          <input
            type="text"
            required
            ref={toInputEl}
            placeholder={t('mail-compose-to-placeholder')}
            autocomplete="off"
            value={to()}
            onInput={(e) => onToInput(e.currentTarget.value)}
            onBlur={() => setAcOpen(false)}
          />
          <Show when={acOpen() && contactAc.suggestions().length > 0}>
            <ul class="compose__ac" role="listbox" aria-label={t('mail-compose-contact-suggestions')}>
              <For each={contactAc.suggestions()}>
                {(s) => (
                  <li>
                    <button
                      type="button"
                      role="option"
                      aria-selected={false}
                      class="compose__ac-item"
                      data-testid="contact-suggestion"
                      // mousedown (not click) so the pick lands before the input's blur.
                      onMouseDown={(e) => {
                        e.preventDefault();
                        pickSuggestion(s);
                      }}
                    >
                      <span class="compose__ac-name">{s.name.length > 0 ? s.name : s.email}</span>
                      <Show when={s.name.length > 0}>
                        <span class="compose__ac-email">{s.email}</span>
                      </Show>
                    </button>
                  </li>
                )}
              </For>
            </ul>
          </Show>
        </label>

        {/* V7 GAL autocomplete (plan §2.7): an ADDITIONAL recipient source beside
            contacts. Mounted only when a directory is configured, so an unconfigured
            deployment's To field is unchanged. Picking a distribution group also
            offers expand-before-send below. */}
        <Show when={app.directory.enabled() && galToken().length > 0}>
          <div class="compose__gal" data-testid="compose-gal">
            <DirectorySearch
              query={galToken()}
              onPick={pickGalEntry}
              service={app.directory.service}
              debounceMs={120}
            />
          </div>
        </Show>
        <Show when={pickedGroup()}>
          {(group) => (
            <GroupExpand
              group={group()}
              service={app.directory.service}
              onExpand={(members) => expandGroupInTo(group(), members)}
            />
          )}
        </Show>

        <label class="field">
          <span>{t('mail-compose-subject')}</span>
          <input type="text" value={subject()} onInput={(e) => setSubject(e.currentTarget.value)} />
        </label>
        <div class="field field--grow">
          <div class="compose__body-head">
            <span>{t('mail-compose-body')}</span>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="format-toggle"
              aria-pressed={!richMode()}
              onClick={() => toggleFormat()}
            >
              {richMode() ? t('mail-compose-format-plain') : t('mail-compose-format-rich')}
            </button>
          </div>
          <Show
            when={richMode()}
            fallback={
              <textarea
                aria-label={t('mail-compose-body')}
                rows="10"
                value={body()}
                onInput={(e) => setBody(e.currentTarget.value)}
              />
            }
          >
            <Suspense
              fallback={
                <textarea
                  aria-label={t('mail-compose-body')}
                  rows="10"
                  value={body()}
                  onInput={(e) => setBody(e.currentTarget.value)}
                />
              }
            >
              <RichTextEditor
                initialHtml={bodyHtml()}
                externalText={body}
                ariaLabel={t('mail-compose-body')}
                onChange={onEditorChange}
                onReady={setEditorApi}
              />
            </Suspense>
          </Show>
        </div>

        <SignaturePicker signatures={signatures} onInsert={insertSignature} />

        {/* V7 inline Assist composer tools + dictation (plan §14.3). Each component
            self-hides on the capabilities it lacks; the whole block is additionally
            gated on the gateway being enabled, so a Disabled Assist gateway renders
            NOTHING here and the composer is unchanged. Nothing is auto-applied or sent. */}
        <Show when={app.assist.enabled()}>
          <div class="compose__assist" data-testid="compose-assist">
            <Dictation
              config={app.assist.config()}
              service={app.assist.service}
              onTranscript={(t) => setBody((cur) => (cur.length > 0 ? `${cur} ${t}` : t))}
            />
            <ComposerTools
              config={app.assist.config()}
              service={app.assist.service}
              text={body()}
              account={app.accountId() ?? ''}
              onApply={setBody}
              onDisclosure={(d) => app.assist.recordDisclosure('draft', d)}
            />
          </div>
        </Show>

        {/* New-file attach (26.15 §1): pick a local file, upload its bytes to the
            account's JMAP upload endpoint, and fold the returned blob into the
            shared attachment list below. Always available (core compose); the
            input is disabled until the session's upload contract has loaded. The
            file input is visually hidden behind the styled label so the composer
            keeps its own button look rather than the native file control. */}
        <div class="compose__attach" data-testid="compose-attach">
          <label class={`btn btn--ghost ${a11y.focusable}`}>
            {uploading() ? t('mail-compose-uploading') : t('mail-compose-attach-file')}
            <input
              type="file"
              multiple
              class={a11y.srOnly}
              aria-label={t('mail-compose-attach-file')}
              disabled={uploading() || uploadUrl() === null || app.accountId() === null}
              onChange={(e) => {
                const input = e.currentTarget;
                void onFilesPicked(input.files).finally(() => {
                  // Reset so re-selecting the same file fires another change.
                  input.value = '';
                });
              }}
            />
          </label>
          <Show when={attachError()}>
            <p class="login__error" role="alert">
              {attachError()}
            </p>
          </Show>
        </div>

        {/* V7 Nextcloud attach (plan §18.4): mounted only when a Nextcloud account is
            linked. Large files are best shared as links (ShareLinkComposer) — here we
            attach materialised blobs; the picker opens on demand. */}
        <Show when={app.nextcloud.enabled()}>
          <div class="compose__nextcloud" data-testid="compose-nextcloud">
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              aria-expanded={ncOpen()}
              onClick={() => setNcOpen((v) => !v)}
            >
              {ncOpen() ? t('mail-compose-close-nextcloud') : t('mail-compose-attach-nextcloud')}
            </button>
            <Show when={ncOpen()}>
              <NextcloudAttach
                service={app.nextcloud.service}
                {...(app.accountId() !== null ? { accountId: app.accountId()! } : {})}
                onAttached={(files) => {
                  setAttachments((cur) => [...cur, ...files]);
                  setNcOpen(false);
                }}
              />
            </Show>
          </div>
        </Show>

        {/* Shared attachment list (Nextcloud + local-file uploads). Rendered
            independent of any backend gate so a local-file attach shows even when
            no Nextcloud account is linked. */}
        <Show when={attachments().length > 0}>
          <ul class="compose__attachments" aria-label={t('mail-compose-attachments')} data-testid="compose-attachments">
            <For each={attachments()}>
              {(a) => (
                <li>
                  <span>{a.name}</span>
                  <button
                    type="button"
                    class={`btn btn--ghost ${a11y.iconButton}`}
                    aria-label={t('mail-compose-remove-attachment', { name: isolate(a.name) })}
                    onClick={() => setAttachments((cur) => cur.filter((x) => x.blobId !== a.blobId))}
                  >
                    ✕
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>

        {/* W11: read-receipt request + open-tracking pixel toggles. Both off by
            default; the tracking-pixel copy states plainly that it embeds a
            remote image. */}
        <SendOptions state={sendOptions} onChange={setSendOptions} />

        {/* Crypto + DLP (plan §2.5): encrypt/sign toggles, the live E2EE/TLS/mixed
            banner from real per-recipient CryptoKey/lookup, and the Dlp/scan
            pre-send warnings. Reports state up via onChange for the send path. */}
        <ComposeCrypto
          recipients={() => splitRecipients(to())}
          subject={() => subject()}
          bodyText={() => body()}
          lookupKeys={lookupKeys}
          scanDlp={scanDlp}
          signingKeyRef={() => signingSession()?.keyRef ?? null}
          onRequestSigningKey={() => setUnlockOpen(true)}
          onChange={setCryptoState}
        />

        {/* Signing-key unlock (plan §2.5, decision flag 2): opens when the sign
            toggle is switched on while the key is locked, or on a signed send with
            no cached session. Unlocks the sending key ONCE — subsequent signed
            sends this session reuse the cached keyRef with no further prompt. */}
        <Show when={unlockOpen()}>
          <section
            class="compose__sign-unlock"
            data-testid="compose-sign-unlock"
            aria-label={t('mail-compose-sign-unlock-title')}
          >
            <p class="compose__sign-unlock-note">{t('mail-compose-sign-unlock-note')}</p>
            <div class="compose__sign-unlock-row">
              <label class="field">
                <span>{t('mail-key-passphrase')}</span>
                <input
                  type="password"
                  class={a11y.focusable}
                  autocomplete="off"
                  data-testid="sign-passphrase"
                  value={unlockPass()}
                  onInput={(e) => setUnlockPass(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    // Enter unlocks without submitting the outer compose form.
                    if (e.key === 'Enter') void unlockSigningKey(e);
                  }}
                />
              </label>
              <button
                type="button"
                class={`btn btn--primary ${a11y.focusable}`}
                data-testid="sign-unlock-submit"
                disabled={unlocking()}
                onClick={(e) => void unlockSigningKey(e)}
              >
                {unlocking() ? t('mail-compose-sign-unlocking') : t('mail-compose-sign-unlock')}
              </button>
            </div>
            <Show when={unlockError()}>
              <p class="login__error" role="alert">
                {unlockError()}
              </p>
            </Show>
          </section>
        </Show>

        <Show when={identity()?.signatureText}>
          {(sig) => <p class="compose__signature">— {isolate(sig())}</p>}
        </Show>

        <label class="field">
          <span>{t('mail-compose-send-later')}</span>
          <input
            type="datetime-local"
            value={sendAt()}
            onInput={(e) => setSendAt(e.currentTarget.value)}
          />
        </label>

        <Show when={error()}>
          <p class="login__error" role="alert">
            {error()}
          </p>
        </Show>
        <footer class="compose__footer">
          <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => props.onClose()}>
            {t('mail-compose-cancel')}
          </button>
          <button type="submit" class={`btn btn--primary ${a11y.focusable}`} disabled={busy()}>
            {busy() ? t('mail-compose-sending') : sendAt() !== '' ? t('mail-compose-schedule') : t('mail-compose-send')}
          </button>
        </footer>
      </form>
    </div>
  );
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** Bytes → megabytes (decimal, 1 MB = 1,000,000 B, matching `maxSizeUpload`),
 *  rounded to one decimal place for a concise, honest size in the UI copy. */
function megabytes(bytes: number): number {
  return Math.round((bytes / 1_000_000) * 10) / 10;
}
