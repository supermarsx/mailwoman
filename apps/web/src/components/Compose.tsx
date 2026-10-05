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
import { AsyncBoundary } from './ErrorBoundary.tsx';
import type { RichTextApi } from './compose/RichTextEditor.tsx';
import {
  SignaturePicker,
  RecallPanel,
  DraftsDrawer,
  type ComposeSignature,
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
import type { ComposeInitial } from './compose/reply.ts';
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
import { parseRecipients, uploadBlob } from '../api/jmap.ts';
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

// Compose (plan §1.5, §2.1): grown with an identity/signature picker (multiple
// from-addresses, server-pulled allowed-froms) and send-later. The core To /
// Subject / Body fields + the Send button keep their exact labels so the mock +
// engine e2e specs still drive it. e10 adds contacts recipient autocomplete to
// the To field — a surgical addition over e7's `createContactAutocomplete`.

/** The recipient token currently being typed: the text after the last separator. */
function tokenBoundary(value: string): number {
  return Math.max(value.lastIndexOf(','), value.lastIndexOf(';'));
}

/** A `datetime-local` value (local wall clock, minute precision) for the first
 *  whole minute after `now` — the earliest time Send later accepts. */
function nextLocalMinute(now: Date): string {
  const d = new Date(Math.floor(now.getTime() / 60_000) * 60_000 + 60_000);
  const pad = (n: number): string => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** The dialog heading for each way a composer can be opened. */
const TITLE_KEY = {
  reply: 'mail-compose-title-reply',
  'reply-all': 'mail-compose-title-reply-all',
  forward: 'mail-compose-title-forward',
} as const;

export function Compose(props: {
  onClose: () => void;
  /** Reply / Reply all / Forward: what the composer starts with. Read once,
   *  when the composer mounts. */
  initial?: ComposeInitial;
}): JSX.Element {
  const app = useApp();
  const initial = props.initial;
  const [to, setTo] = createSignal(initial?.to ?? '');
  const [cc, setCc] = createSignal(initial?.cc ?? '');
  const [bcc, setBcc] = createSignal('');
  // The Cc and Bcc fields stay folded away until asked for, or until one of
  // them holds something (a reply-all, a resumed draft).
  const [ccBccOpen, setCcBccOpen] = createSignal((initial?.cc ?? '') !== '');
  // The ids a reply carries (`In-Reply-To`, `References`). Not editable; a
  // resumed draft brings its own.
  const [threading, setThreading] = createSignal<{ inReplyTo: string[]; references: string[] }>({
    inReplyTo: initial?.inReplyTo ?? [],
    references: initial?.references ?? [],
  });
  const [subject, setSubject] = createSignal(initial?.subject ?? '');
  // `body` stays the PLAIN-TEXT source of truth (crypto/DLP/dictation read it).
  // `bodyHtml` carries the rich-text HTML, and is what the normal send path
  // sends ONLY while the rich editor is mounted (`editorApi() !== null`); the
  // editor keeps both in sync. Whenever the editor is not there — plain-text
  // mode, its chunk still loading, its chunk failed — the send is built from
  // `body`. `richMode` toggles the ProseMirror editor vs a plain-text textarea;
  // the toggle round-trips the text.
  const [body, setBody] = createSignal(initial?.bodyText ?? '');
  const [bodyHtml, setBodyHtml] = createSignal(initial?.bodyHtml ?? '');

  /** The plain-text Body field. It is the fallback for all three ways the rich
   *  editor can be absent — plain-text mode, the chunk still loading, and the
   *  chunk failing to load at all — so it lives in one place rather than three. */
  const plainBody = (): JSX.Element => (
    <textarea
      aria-label={t('mail-compose-body')}
      rows="10"
      value={body()}
      onInput={(e) => {
        // Keep `bodyHtml` in step, so an editor that mounts later (its chunk
        // arrived after the user started typing here) is seeded with this text
        // instead of starting empty and overwriting it.
        setBody(e.currentTarget.value);
        setBodyHtml(plainToHtml(e.currentTarget.value));
      }}
    />
  );
  const [richMode, setRichMode] = createSignal(true);
  const [editorApi, setEditorApi] = createSignal<RichTextApi | null>(null);
  // W9 drafts drawer + W10 recall panel visibility, and the loaded draft list.
  const [draftsOpen, setDraftsOpen] = createSignal(false);
  const [recallOpen, setRecallOpen] = createSignal(false);
  const [drafts, setDrafts] = createSignal<StoredDraft[]>([]);
  // Stable id for THIS composer's auto-saved draft (W9).
  const draftId = newDraftId();
  const [identityId, setIdentityId] = createSignal<string>('');
  const [sendAt, setSendAt] = createSignal('');
  // Earliest time the Send-later field offers; refreshed when the field is
  // focused, since a composer can stay open for a long time.
  const [minSendAt, setMinSendAt] = createSignal(nextLocalMinute(new Date()));
  // Identities whose signature the picker has already put into the body, so
  // the send does not append the same signature a second time.
  const [insertedSignatures, setInsertedSignatures] = createSignal<ReadonlySet<string>>(new Set());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [acOpen, setAcOpen] = createSignal(false);
  // V7 GAL (plan §2.7): the in-progress recipient token also drives a directory
  // autocomplete as an ADDITIONAL source beside contacts. `pickedGroup` holds a
  // distribution group the sender may expand-before-send into its leaf recipients.
  const [galToken, setGalToken] = createSignal('');
  const [pickedGroup, setPickedGroup] = createSignal<GalEntry | null>(null);
  // V7 Nextcloud attach (plan §18.4): materialised attachments + the picker toggle.
  const [attachments, setAttachments] = createSignal<AttachedFile[]>(initial?.attachments ?? []);
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
  // A reply or forward that quotes the decrypted text of an encrypted message
  // is not sent unencrypted until the user has said so, in the dialog below,
  // for this composer.
  const quotesDecrypted = initial?.quotesDecrypted === true;
  const [plaintextConfirmOpen, setPlaintextConfirmOpen] = createSignal(false);
  const [plaintextConfirmed, setPlaintextConfirmed] = createSignal(false);

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
  let cryptoBoxEl: HTMLDivElement | undefined;
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

  // A reply is answered above the quote: once the editor is there, the caret
  // goes to its first (empty) paragraph. A forward still needs its recipient,
  // so focus stays in To.
  if (initial !== undefined && initial.mode !== 'forward') {
    let moved = false;
    createEffect(() => {
      const api = editorApi();
      if (api === null || moved) return;
      moved = true;
      api.focus();
    });
  }

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

  /** The signature the send appends by itself: the sending identity's, unless
   *  the picker already put it in the body, and never on an encrypted or signed
   *  send — there the body is ciphertext or a clear-signed block, and anything
   *  appended after it would sit outside the encryption or break the signature. */
  const autoSignature = createMemo<ComposeSignature | null>(() => {
    const id = identity();
    if (id === null) return null;
    const sig = signatures().find((s) => s.id === id.id) ?? null;
    if (sig === null || insertedSignatures().has(sig.id)) return null;
    const cs = cryptoState();
    if (cs !== null && (cs.encrypt || cs.sign)) return null;
    return sig;
  });

  /** A signature as HTML for the outgoing body. `signatureHtml` is whatever the
   *  server holds for the identity, so it is parsed into the composer's schema
   *  and re-serialised — the same reduction typed and pasted content gets — and
   *  only that result is sent. If the schema's chunk cannot be loaded, the
   *  signature goes out as escaped text instead of as unfiltered markup. */
  async function signatureBlock(sig: ComposeSignature): Promise<string> {
    if (sig.html !== null && sig.html !== '') {
      try {
        const rt = await import('./compose/richtext.ts');
        return rt.htmlFromDoc(rt.docFromHtml(sig.html));
      } catch {
        // Fall through to the text form.
      }
    }
    return plainToHtml(sig.text);
  }

  /** Insert a chosen signature (W12). In rich mode it appends as HTML through the
   *  editor handle (keeping existing formatting); otherwise it appends its text
   *  to the plain body. */
  function insertSignature(sig: ComposeSignature): void {
    setInsertedSignatures((prev) => new Set(prev).add(sig.id));
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
    setCc(d.cc ?? '');
    setBcc(d.bcc ?? '');
    setCcBccOpen((d.cc ?? '') !== '' || (d.bcc ?? '') !== '');
    setThreading({ inReplyTo: d.inReplyTo ?? [], references: d.references ?? [] });
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
  // (see `draftHasContent`). A composition quoting decrypted text is never
  // saved: local storage is plaintext, and the original was not.
  onMount(() => setDrafts(listDrafts()));
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(() => {
    if (quotesDecrypted) return;
    const snapshot: StoredDraft = {
      id: draftId,
      to: to(),
      ...(cc().trim() !== '' ? { cc: cc() } : {}),
      ...(bcc().trim() !== '' ? { bcc: bcc() } : {}),
      ...(threading().inReplyTo.length > 0 ? { inReplyTo: threading().inReplyTo } : {}),
      ...(threading().references.length > 0 ? { references: threading().references } : {}),
      subject: subject(),
      bodyHtml: richMode() && editorApi() !== null ? bodyHtml() : plainToHtml(body()),
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
    // Send later: a time that is not in the future is refused here, with the
    // field left as typed. It is never turned into an immediate send.
    let sendAtIso: string | null = null;
    if (sendAt() !== '') {
      // datetime-local yields a local wall-clock string; convert to a UTC ISO.
      const at = new Date(sendAt());
      if (Number.isNaN(at.getTime()) || at.getTime() <= Date.now()) {
        setError(t('mail-compose-send-later-past'));
        return;
      }
      sendAtIso = at.toISOString();
    }
    const willEncrypt = cs !== null && cs.encrypt && cs.encryptedDraft !== null;
    // An encrypted message names every key it is encrypted to, so a Bcc
    // recipient of one is visible to the others. Refused rather than sent.
    if (cs !== null && cs.encrypt && parseRecipients(bcc()).length > 0) {
      setError(t('mail-compose-bcc-encrypted'));
      return;
    }
    // Quoted plaintext of an encrypted original leaves unencrypted only after
    // an explicit confirmation (the dialog re-submits with it given).
    if (quotesDecrypted && !willEncrypt && !plaintextConfirmed()) {
      setPlaintextConfirmOpen(true);
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
      } else {
        // The rich editor's serialized HTML, but only while the editor is
        // actually mounted. In plain-text mode, and when the editor's chunk is
        // still loading or failed to load, the user typed into the textarea and
        // `bodyHtml` does not hold their text — the body is built from `body`.
        htmlBody = richMode() && editorApi() !== null ? bodyHtml() : plainToHtml(body());
        const sig = autoSignature();
        if (sig !== null) htmlBody += `<br>${await signatureBlock(sig)}`;
      }
      const subjectToSend =
        enc !== null && cs !== null && cs.protectSubject && enc.encryptedSubjectApplied
          ? t('mail-compose-encrypted-subject')
          : subject();
      const attached = attachments();
      await app.sendMessage({
        to: to(),
        cc: cc(),
        bcc: bcc(),
        inReplyTo: threading().inReplyTo,
        references: threading().references,
        subject: subjectToSend,
        htmlBody,
        identity: identity(),
        sendAt: sendAtIso,
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
          <h2>{t(initial !== undefined ? TITLE_KEY[initial.mode] : 'mail-compose-title')}</h2>
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

        <Show
          when={ccBccOpen()}
          fallback={
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="compose-show-cc-bcc"
              onClick={() => setCcBccOpen(true)}
            >
              {t('mail-compose-show-cc-bcc')}
            </button>
          }
        >
          <label class="field">
            <span>{t('mail-compose-cc')}</span>
            <input type="text" autocomplete="off" value={cc()} onInput={(e) => setCc(e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>{t('mail-compose-bcc')}</span>
            <input type="text" autocomplete="off" value={bcc()} onInput={(e) => setBcc(e.currentTarget.value)} />
          </label>
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
          <Show when={richMode()} fallback={plainBody()}>
            {/* The editor chunk can FAIL to arrive, not just be slow — offline, or
                a 404 against a tab open across a redeploy. `Suspense` covers only
                the PENDING case; a rejected `lazy()` import throws, and with no
                boundary here that throw escapes the dialog. The plain textarea
                was always the documented contract for "the editor is not here"
                (see the `lazy()` call above), so honour it when the import fails
                and not only while it is in flight — the same reasoning
                ErrorBoundary.tsx was written for.

                PROVEN to fix the `e2e-engine` `offline.spec.ts:18` failure
                ("Compose message" dialog never found), by controlled revert: on
                `9216888` plus a revert of this boundary and nothing else, that
                spec fails on all three retries with `element(s) not found` for
                the dialog (run 36255164856, 30 passed / 1 failed), while
                `9216888` itself is green. The signature is byte-identical to the
                historical failure at `b34b9d3`.

                Mechanism: `offline.spec.ts` opens Compose for the first time while
                the browser context is offline, so the ~286 kB ProseMirror chunk
                has never been fetched into the service-worker cache. The import
                rejects, and without this boundary the throw escapes and the dialog
                never renders — which is why the error was `element(s) not found`
                on the ROLE, never a name mismatch.

                Do not try to cover this with a unit test: under vitest a mocked
                rejecting import leaves Suspense pending forever, so the boundary is
                never exercised and such a test passes with OR without this wrapper.
                One was written, found to be vacuous, and deleted. The real coverage
                is `offline.spec.ts:18` itself — a real browser with the SW cache
                cold and the context offline. */}
            <AsyncBoundary fallback={() => plainBody()}>
              <Suspense fallback={plainBody()}>
                <RichTextEditor
                  initialHtml={bodyHtml()}
                  externalText={body}
                  ariaLabel={t('mail-compose-body')}
                  onChange={onEditorChange}
                  onReady={setEditorApi}
                />
              </Suspense>
            </AsyncBoundary>
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

        {/* Crypto + DLP (plan §2.5): encrypt/sign toggles, the live E2EE/TLS/mixed
            banner from real per-recipient CryptoKey/lookup, and the Dlp/scan
            pre-send warnings. Reports state up via onChange for the send path. */}
        {/* A boundary of its own, because the panel reads two resources (the
            per-recipient key lookup and the DLP scan) that go pending every time
            a recipient or the body changes. Without it the nearest boundary is
            the one around the whole signed-in shell (`LazyRoute` in App.tsx):
            each keystroke in the body detached the entire shell for the length
            of a DLP request, and the editor lost focus after one character.
            With it only this panel is held back while a request is out, and
            the placeholder keeps the panel's height so the fields below it do
            not move. */}
        <div ref={cryptoBoxEl}>
          <Suspense
            fallback={<div aria-hidden="true" style={{ height: `${cryptoBoxEl?.offsetHeight ?? 0}px` }} />}
          >
            <ComposeCrypto
              recipients={() => parseRecipients([to(), cc(), bcc()].join(',')).map((r) => r.email)}
              subject={() => subject()}
              bodyText={() => body()}
              lookupKeys={lookupKeys}
              scanDlp={scanDlp}
              signingKeyRef={() => signingSession()?.keyRef ?? null}
              onRequestSigningKey={() => setUnlockOpen(true)}
              onChange={setCryptoState}
            />
          </Suspense>
        </div>

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

        {/* Shown exactly when the send will append it (see `autoSignature`). */}
        <Show when={autoSignature()}>
          {(sig) => (
            <p class="compose__signature" data-testid="compose-auto-signature">
              — {isolate(sig().text)}
            </p>
          )}
        </Show>

        <label class="field">
          <span>{t('mail-compose-send-later')}</span>
          <input
            type="datetime-local"
            min={minSendAt()}
            value={sendAt()}
            onFocus={() => setMinSendAt(nextLocalMinute(new Date()))}
            onInput={(e) => setSendAt(e.currentTarget.value)}
            // The browser blocks the submit itself for a value under `min`;
            // say why in the dialog as well as in its own bubble.
            onInvalid={() => setError(t('mail-compose-send-later-past'))}
          />
        </label>

        {/* Asked once per composer, when a send would put the decrypted text of
            an encrypted original on the wire unencrypted. Confirming sends. */}
        <Show when={plaintextConfirmOpen()}>
          <section
            class="compose__sign-unlock"
            role="alertdialog"
            aria-labelledby="compose-plaintext-confirm-title"
            aria-describedby="compose-plaintext-confirm-body"
            data-testid="compose-plaintext-confirm"
          >
            <p id="compose-plaintext-confirm-title" class="compose__sign-unlock-note">
              <strong>{t('mail-compose-plaintext-confirm-title')}</strong>
            </p>
            <p id="compose-plaintext-confirm-body" class="compose__sign-unlock-note">
              {t('mail-compose-plaintext-confirm-body')}
            </p>
            <div class="compose__sign-unlock-row">
              <button
                type="button"
                class={`btn btn--ghost ${a11y.focusable}`}
                onClick={() => setPlaintextConfirmOpen(false)}
              >
                {t('mail-compose-plaintext-confirm-cancel')}
              </button>
              <button
                type="button"
                class={`btn btn--primary ${a11y.focusable}`}
                data-testid="compose-plaintext-confirm-send"
                onClick={(e) => {
                  setPlaintextConfirmed(true);
                  setPlaintextConfirmOpen(false);
                  void onSubmit(e);
                }}
              >
                {t('mail-compose-plaintext-confirm-send')}
              </button>
            </div>
          </section>
        </Show>

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
