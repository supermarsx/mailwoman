import {checkRecipientNames} from './compose/recipientNames.ts';
import {EditorCounts} from './compose/EditorCounts.tsx';
import {InsertAccountItem} from './compose/InsertAccountItem.tsx';
import {accountDraftApi,draftEditorState,localAccountDraft,type AccountDraftDocument} from '../api/accountDrafts.ts';
import {createDraftAutosave,type DraftSaveStatus,type DraftSaveMode} from './compose/accountDraftAutosave.ts';
import {toast} from './toast/toast.ts';
import {observePreferenceAccount,preferenceAccount, preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import { settings } from '../api/settings.ts';
import { supportsDeliveryReceipts } from '../api/deliveryReceipts.ts';
import { externalRecipients } from './compose/recipientTrust.ts';
import { Icon } from './Icon.tsx';
import {
  createEffect,
  createMemo,
  createSignal,
  createUniqueId,
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
import {ComposerHistory} from './compose/ComposerHistory.tsx';
import {
  SignaturePicker,
  RecallPanel,
  DraftsDrawer,
  type ComposeSignature,
} from './compose/ComposerExtras.tsx';
import {
  draftHasContent,
  listDrafts,
  saveDraft,
  deleteDraft,
  newDraftId,
  type StoredDraft,
} from './compose/drafts-store.ts';
import { SendOptions } from './compose/SendOptions.tsx';
import {
  inlineAttachments,
  inlineRefusal,
  newCid,
  referencedCids,
  storedImage,
  type InlineImage,
} from './compose/inlineImages.ts';
import { clientPolicy, loadClientPolicy } from '../api/policy.ts';
import { buildDownloadUrl, fetchObjectUrl } from '../viewers/attachments.ts';

// The rich-text editor pulls in ProseMirror (MIT, self-hosted). Loaded lazily so
// those libraries land in their own chunk and never inflate the login→inbox
// entry the size gate measures — the composer is user-triggered, so the small
// deferred load is invisible in practice. A plain textarea backs the Suspense
// fallback, so the Body field is usable (and its label present) before the
// chunk resolves.
const RichTextEditor = lazy(() => import('./compose/RichTextEditor.tsx'));
const HtmlSourceEditor = lazy(() => import('./compose/HtmlSourceEditor.tsx'));
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

// Advertised in the session only by this server's own engine (mw-engine
// `session_json`), which is what turns `mailwomanRequestReadReceipt` into a
// `Disposition-Notification-To` header. A proxied JMAP server does not know the
// property, so the receipt option is not offered there.
const ENGINE_CAP = 'urn:mailwoman:security';

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

type RecipientField = 'to' | 'cc' | 'bcc';

/** The dialog heading for each way a composer can be opened. */
const TITLE_KEY = {
  new: 'mail-compose-title',
  reply: 'mail-compose-title-reply',
  'reply-all': 'mail-compose-title-reply-all',
  forward: 'mail-compose-title-forward',
} as const;

export function Compose(props: {
  onClose: () => void;
  embedded?: boolean;
  active?: boolean;
  /** Workspace metadata contains only this account-owned local draft ID. */
  resumeDraftId?: string;
  onDraftIdChange?: (id:string)=>void;
  onSubjectChange?: (subject:string)=>void;
  registerBeforeClose?: (handler:()=>Promise<boolean>)=>void;
  /** Reply / Reply all / Forward: what the composer starts with. Read once,
   *  when the composer mounts. */
  initial?: ComposeInitial;
}): JSX.Element {
  const app = useApp();
  const [composerAuthority,setComposerAuthority]=createSignal(preferenceAccount());
  onCleanup(observePreferenceAccount(setComposerAuthority));
  function localDraftAuthority():string|undefined {const owner=composerAuthority();return owner?.id===app.accountId()?owner.storageAuthority:undefined;}
  const composeId=createUniqueId();
  const initial = props.initial;
  const [expanded, setExpanded] = createSignal(false);
  const compact = () => initial?.compact === true && !expanded();
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
    <><textarea
      aria-label={t('mail-compose-body')}
      rows={compact() ? 4 : 10}
      value={body()}
      onInput={(e) => {
        // Keep `bodyHtml` in step, so an editor that mounts later (its chunk
        // arrived after the user started typing here) is seeded with this text
        // instead of starting empty and overwriting it.
        setBody(e.currentTarget.value);
        setBodyHtml(plainToHtml(e.currentTarget.value));
      }}
    /><EditorCounts text={body()}/></>
  );
  const [richMode, setRichMode] = createSignal(true);
  const [sourceMode,setSourceMode]=createSignal(false);
  const [editorApi, setEditorApi] = createSignal<RichTextApi | null>(null);
  // W9 drafts drawer + W10 recall panel visibility, and the loaded draft list.
  const [draftsOpen, setDraftsOpen] = createSignal(false);
  const [recallOpen, setRecallOpen] = createSignal(false);
  const [recallError,setRecallError]=createSignal(false);
  const [recallBusy,setRecallBusy]=createSignal(false);
  const [drafts, setDrafts] = createSignal<StoredDraft[]>([]);
  // Stable id for THIS composer's auto-saved draft (W9).
  let draftId = newDraftId();
  const [remoteRevision,setRemoteRevision]=createSignal<string|null>(null);
  const [remoteReady,setRemoteReady]=createSignal(false);
  const [remoteStatus,setRemoteStatus]=createSignal<DraftSaveStatus>('idle');
  const [remoteDrafts,setRemoteDrafts]=createSignal<AccountDraftDocument[]>([]);
  const [remoteLoading,setRemoteLoading]=createSignal(false);
  const [discardConfirmation,setDiscardConfirmation]=createSignal<AccountDraftDocument|null>(null);
  let remoteWriter:ReturnType<typeof createDraftAutosave>|undefined;
  let remoteWriterId='';
  let resumedUncertain=false;
  let remoteAbort:AbortController|undefined;

  const [identityId, setIdentityId] = createSignal<string>('');
  const [fromOverride, setFromOverride] = createSignal('');
  const [importance,setImportance]=createSignal<'high'|'normal'|'low'>('normal');
  const [customFrom, setCustomFrom] = createSignal(false);
  const fromHelpId = createUniqueId();
  const [ribbonTab,setRibbonTab]=createSignal<'format'|'insert'|'table'|'security'|'delivery'>('format');
  createEffect(()=>{if(unlockOpen())setRibbonTab('security');});
  const [fromVisibleOverride,setFromVisibleOverride] = createSignal<boolean|null>(null);
  const showFrom=()=>customFrom() || (fromVisibleOverride() ?? settings()['mail.showFrom']?.value === true);
  const [sendAt, setSendAt] = createSignal('');
  // Earliest time the Send-later field offers; refreshed when the field is
  // focused, since a composer can stay open for a long time.
  const [minSendAt, setMinSendAt] = createSignal(nextLocalMinute(new Date()));
  // Identities whose signature the picker has already put into the body, so
  // the send does not append the same signature a second time.
  const [insertedSignatures, setInsertedSignatures] = createSignal<ReadonlySet<string>>(new Set());
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  // The recipient field whose in-progress token the contact suggestions are
  // for, or `null` when no list is open.
  const [acField, setAcField] = createSignal<RecipientField | null>(null);
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
  // Images placed in the body (t29-e12): uploaded blobs with a Content-ID each.
  // The body refers to them by `cid:`; the editor shows `previewUrl`.
  const [inlineImages, setInlineImages] = createSignal<InlineImage[]>([]);
  const [inlineError, setInlineError] = createSignal<string | null>(null);
  const ownedCids = createMemo<ReadonlySet<string>>(() => new Set(inlineImages().map((i) => i.cid)));
  const [downloadUrl, setDownloadUrl] = createSignal<string | null>(null);
  // Receipt preferences supply initial choices; drafts keep their own choices.
  // Read-receipt headers need the engine; DSN needs this account's capability.
  const [receiptAvailable, setReceiptAvailable] = createSignal(false);
  const [requestReceipt, setRequestReceiptValue] = createSignal(settings()['mail.requestReadReceipt']?.value===true);
  let receiptInitialized=settings()['mail.requestReadReceipt']!==undefined;
  const setRequestReceipt=(value:boolean):void=>{receiptInitialized=true;setRequestReceiptValue(value);};
  createEffect(()=>{const entry=settings()['mail.requestReadReceipt'];if(!receiptInitialized && entry){receiptInitialized=true;setRequestReceiptValue(entry.value===true);}});
  const [deliveryCapable,setDeliveryCapable]=createSignal(false);
  const deliveryAvailable=()=>deliveryCapable() && settings()['receipts.dsnAllowed']?.value===true;
  const [requestDelivery,setRequestDeliveryValue]=createSignal(false);
  let deliveryEdited=false;let deliveryDisposed=false;
  const setRequestDelivery=(value:boolean):void=>{deliveryEdited=true;setRequestDeliveryValue(value);};
  createEffect(()=>{const entry=settings()['mail.requestDeliveryReceipt'];if(!deliveryEdited&&entry&&deliveryAvailable())setRequestDeliveryValue(entry.value===true);});
  const [trackOpens, setTrackOpens] = createSignal(false);
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
  const protectedDraft=()=>quotesDecrypted||cryptoState()?.encrypt===true;
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
    if (props.embedded) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      void closeWithDraftSave();
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
    if (props.active !== false) toInputEl?.focus();
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
    // Whether this server records opens (`openTracking`); read once per session.
    void loadClientPolicy();
  });

  let sessionEpoch=0;
  createEffect(()=>{
    const deliveryAccount=app.accountId();
    const epoch=++sessionEpoch;
    setDeliveryCapable(false);setReceiptAvailable(false);setUploadUrl(null);setDownloadUrl(null);
    if(deliveryAccount===null)return;
    void jmapClient
      .session()
      .then((s) => {
        if(deliveryDisposed || epoch!==sessionEpoch || app.accountId()!==deliveryAccount)return;
        setDeliveryCapable(deliveryAccount!==null && supportsDeliveryReceipts(s,deliveryAccount));
        setUploadUrl(s.uploadUrl);
        setDownloadUrl(s.downloadUrl);
        setReceiptAvailable(s.capabilities[ENGINE_CAP] !== undefined);
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
  if (initial !== undefined && (initial.mode === 'reply' || initial.mode === 'reply-all')) {
    let moved = false;
    createEffect(() => {
      const api = editorApi();
      if (api === null || moved) return;
      moved = true;
      api.focus();
    });
  }

  onCleanup(() => {
    deliveryDisposed=true;
    if (!props.embedded) previouslyFocused?.focus();
    for (const img of inlineImages()) if (img.previewUrl !== null) URL.revokeObjectURL(img.previewUrl);
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
  const externalMode = () => settings()['mail.externalRecipientWarning']?.value ?? 'warn';
  const outside = createMemo(() => {
    const trusted=settings()['mail.trustedRecipientDomains']?.value;
    return externalRecipients([to(),cc(),bcc()],identity()?.email ?? app.identities()[0]?.email ?? app.me()?.username ?? '',Array.isArray(trusted)?trusted.filter((domain):domain is string=>typeof domain==='string'):[]);
  });
  const recipientReviewKey=()=>JSON.stringify([outside(),identity()?.email,externalMode()]);
  const [reviewedRecipients,setReviewedRecipients]=createSignal('');

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
  function changeFormat(mode:string):void {
    if(!richMode() && mode!=='plain')setBodyHtml(plainToHtml(body()));
    if(mode!=='rich'&&ribbonTab()==='table')setRibbonTab('format');
    setSourceMode(mode==='html');
    setRichMode(mode!=='plain');
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

  /** "Record when this message is opened" is offered only when the server has
   *  open tracking on and the message is neither encrypted nor signed (the
   *  image would sit outside the encryption or break the signature). */
  const trackingAvailable = createMemo(() => {
    if (!clientPolicy().openTracking || !richMode() || editorApi() === null) return false;
    const cs = cryptoState();
    return !(cs !== null && (cs.encrypt || cs.sign));
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

  async function signaturePlain(sig: ComposeSignature): Promise<string> {
    const source = app.identities().find(identity => identity.id === sig.id);
    if (source?.signatureText?.trim()) return source.signatureText;
    if (sig.html) {
      try { return (await import('./compose/richtext.ts')).htmlToText(sig.html); } catch { /* Plain fallback remains inert. */ }
    }
    return sig.text;
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
    const owner=preferenceAccount();
    if (d.accountId !== app.accountId() || !owner || owner.id!==d.accountId || !preferenceAccountIsCurrent(owner) || !owner.storageAuthority || d.storageAuthority!==owner.storageAuthority) return;
    remoteWriter?.dispose();remoteWriter=undefined;remoteWriterId='';
    resumedUncertain=d.serverStatus==='uncertain';
    draftId = d.id;
    setRemoteRevision(d.serverRevision??null);
    props.onDraftIdChange?.(draftId);
    setExpanded(true);
    setIdentityId(d.identityId ?? '');
    setFromOverride(d.fromOverride ?? '');
    setImportance(d.importance ?? 'normal');
    setCustomFrom(Boolean(d.fromOverride));
    setInsertedSignatures(new Set(d.insertedSignatures ?? []));
    setSourceMode(false);
    setRichMode(d.bodyFormat !== 'plain');
    setTo(d.to);
    setCc(d.cc ?? '');
    setBcc(d.bcc ?? '');
    setCcBccOpen((d.cc ?? '') !== '' || (d.bcc ?? '') !== '');
    setThreading({ inReplyTo: d.inReplyTo ?? [], references: d.references ?? [] });
    setSubject(d.subject);
    setRequestReceipt(d.requestReceipt === true);
    setRequestDelivery(d.requestDeliveryReceipt === true);
    setTrackOpens(d.openTracking === true);
    const images = (d.inline ?? []).map((i) => ({ ...i, previewUrl: null }));
    setInlineImages(images);
    setAttachments(d.attachments ?? []);
    setBody(d.bodyText);
    setBodyHtml(d.bodyHtml);
    const api = editorApi();
    if (richMode() && api !== null) api.setHtml(d.bodyHtml);
    setDraftsOpen(false);
    void loadPreviews(images);
  }

  /** Fetch a local preview for each image of a resumed draft (its uploaded
   *  blob, through the authenticated download route). An image whose blob is
   *  gone shows its name as alt text. */
  async function loadPreviews(images: InlineImage[]): Promise<void> {
    const template = downloadUrl();
    const acct = app.accountId();
    if (template === null || acct === null) return;
    for (const img of images) {
      try {
        const url = await fetchObjectUrl(
          buildDownloadUrl(template, { accountId: acct, blobId: img.blobId, name: img.name, mime: img.type }),
        );
        if (!ownsDraft() || app.accountId() !== acct) { URL.revokeObjectURL(url); return; }
        setInlineImages((cur) => cur.map((i) => (i.cid === img.cid ? { ...i, previewUrl: url } : i)));
      } catch {
        // The blob is no longer there; the image keeps its alt text.
      }
    }
    editorApi()?.refreshImages();
  }

  /** Upload image files for the body (button, paste or drop). Each refused or
   *  failed file is named; the others are placed. */
  async function uploadInline(files: File[]): Promise<{ cid: string; alt: string }[]> {
    const url = uploadUrl();
    const acct = app.accountId();
    if (url === null || acct === null) {
      setInlineError(t('mail-compose-upload-unavailable'));
      return [];
    }
    setInlineError(null);
    const max = maxUploadSize();
    const placed: { cid: string; alt: string }[] = [];
    for (const file of files) {
      const refusal = inlineRefusal(file, max);
      if (refusal === 'type') {
        setInlineError(t('mail-compose-image-type', { name: isolate(file.name) }));
        continue;
      }
      if (refusal === 'size') {
        setInlineError(
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
        if (!ownsDraft() || app.accountId() !== acct) return [];
        const cid = newCid();
        const previewUrl = URL.createObjectURL(file);
        setInlineImages((cur) => [
          ...cur,
          { cid, blobId: up.blobId, type: file.type, name: file.name, size: up.size, previewUrl },
        ]);
        placed.push({ cid, alt: file.name });
      } catch {
        setInlineError(t('mail-compose-upload-failed', { name: isolate(file.name) }));
      }
    }
    return placed;
  }

  /** Discard a stored draft and refresh the list (W9). */
  function discardDraft(id: string): void {
    deleteDraft(id, app.accountId(),localDraftAuthority());
    setDrafts(listDrafts(app.accountId(),localDraftAuthority()));
  }

  /** Open the Drafts drawer, refreshing the list from storage first (W9). */
  function openDrafts(): void {
    setDrafts(listDrafts(app.accountId(),localDraftAuthority()));
    setDraftsOpen(true);void refreshRemoteDrafts();
  }

  /** Open the recall panel, refreshing the server-held submission queue (W10). */
  function openRecall(): void {
    setRecallOpen(true);setRecallError(false);
    void app.refreshOutbox().catch(()=>{if(ownsDraft())setRecallError(true);});
  }

  /** Recall (cancel) a still-holding / scheduled submission before it dispatches. */
  function recallSubmission(id: string): void {
    if(recallBusy())return;
    setRecallBusy(true);setRecallError(false);
    void app.cancelOutbox(id).then(() => app.refreshOutbox()).catch(()=>{if(ownsDraft())setRecallError(true);}).finally(()=>{if(ownsDraft())setRecallBusy(false);});
  }

  function remoteFailure(code:string):void {
    if(!ownsDraft())return;
    toast.error(t(code==='draftSaveUncertain'?'mail-draft-uncertain':code==='draftConflict'?'mail-draft-conflict':code==='draftCleanupPending'?'mail-draft-cleanup-pending':code==='draftDiscardPending'?'mail-draft-discard-pending':code==='draftLimit'?'mail-draft-limit':'mail-draft-save-failed'));
  }
  async function refreshRemoteDrafts():Promise<void>{
    const owner=composerAuthority();if(!owner?.storageAuthority||!ownsDraft())return;
    setRemoteLoading(true);
    try{const result=await accountDraftApi(owner,remoteAbort?.signal).list();if(!ownsDraft())return;setRemoteReady(result.supported);setRemoteDrafts(result.items);if(!result.supported)setRemoteStatus('unsupported');}
    catch{if(ownsDraft())setRemoteStatus('error');}
    finally{if(ownsDraft())setRemoteLoading(false);}
  }
  async function resumeRemoteDraft(id:string):Promise<void>{
    const owner=composerAuthority();if(!owner?.storageAuthority||!ownsDraft()||remoteLoading())return;
    if(remoteWriter?.isBusy())return;
    if(remoteWriter?.hasPending()&&settings()['mail.draftSaveMode']?.value!=='manual'&&!await remoteWriter.flush())return;
    if(pendingDraft&&ownsDraft())saveDraft(pendingDraft);
    setRemoteLoading(true);
    try{const document=await accountDraftApi(owner,remoteAbort?.signal).load(id);if(!ownsDraft())return;
      const local=localAccountDraft(document);if(document.revision)local.serverRevision=document.revision;
      resumeDraft(local);ensureRemoteWriter();remoteWriter?.resume(document);
      setRemoteStatus(document.status==='uncertain'?'uncertain':'saved');
    }catch{remoteFailure('draftStateUnavailable');}finally{if(ownsDraft())setRemoteLoading(false);}
  }
  async function discardRemoteDraft(document:AccountDraftDocument):Promise<void>{
    const owner=composerAuthority();if(!owner?.storageAuthority||!ownsDraft()||remoteLoading())return;
    if(remoteWriterId===document.id){remoteWriter?.suspend(true);await remoteWriter?.settled();}
    setRemoteLoading(true);
    try{await accountDraftApi(owner,remoteAbort?.signal).discard(document.id,document.revision);if(!ownsDraft())return;deleteDraft(document.id,owner.id,owner.storageAuthority);setRemoteDrafts(rows=>rows.filter(row=>row.id!==document.id));void app.refreshFolders();
      if(draftId===document.id){allowRecoveryClose=true;pendingDraft=undefined;sent=true;props.onClose();}
    }catch{remoteFailure('draftDiscardPending');}finally{if(ownsDraft())setRemoteLoading(false);}
  }
  function ensureRemoteWriter():void{
    const owner=composerAuthority();if(!remoteReady()||protectedDraft()||!owner?.storageAuthority||!ownsDraft())return;
    if(remoteWriter&&remoteWriterId===draftId)return;
    remoteWriter?.dispose();remoteWriterId=draftId;
    remoteWriter=createDraftAutosave({id:draftId,transport:accountDraftApi(owner,remoteAbort?.signal),current:()=>ownsDraft()&&preferenceAccountIsCurrent(owner)&&!sent,status:setRemoteStatus,failure:remoteFailure,saved:(before,document)=>{
      setRemoteRevision(document.revision);
      void app.refreshFolders();
      const rebound=new Map<string,{blobId:string;size:number}>();
      before.attachments.forEach((file,index)=>{const saved=document.state?.attachments[index];if(saved)rebound.set(file.blobId,saved);});
      before.inline.forEach(file=>{const saved=document.state?.inline.find(value=>value.cid===file.cid);if(saved)rebound.set(file.blobId,saved);});
      setAttachments(files=>files.map(file=>{const saved=rebound.get(file.blobId);return saved?{...file,blobId:saved.blobId,size:saved.size}:file;}));
      setInlineImages(files=>files.map(file=>{const saved=rebound.get(file.blobId);return saved?{...file,blobId:saved.blobId,size:saved.size}:file;}));
    }});
    remoteWriter.seedRevision(remoteRevision());
    if(resumedUncertain)remoteWriter.seedUncertain();
  }
  let allowRecoveryClose=false;
  async function beforeDraftClose():Promise<boolean>{
    if(sent||allowRecoveryClose||!ownsDraft())return true;
    if(uploading()||remoteWriter?.isBusy())return false;
    if(settings()['mail.draftSaveMode']?.value!=='manual'&&remoteWriter?.hasPending()&&!await remoteWriter.flush())return false;
    return true;
  }
  async function closeWithDraftSave():Promise<void>{if(await beforeDraftClose())props.onClose();}
  onMount(()=>props.registerBeforeClose?.(beforeDraftClose));
  // W9 auto-save: debounce a snapshot of the composition to local storage so a
  // closed / refreshed composer can be resumed. Empty compositions are skipped
  // (see `draftHasContent`). A composition quoting decrypted text is never
  // saved: local storage is plaintext, and the original was not.
  onMount(() => setDrafts(listDrafts(app.accountId(),localDraftAuthority())));
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let pendingDraft:StoredDraft | undefined;
  let sent=false;
  let draftAccount=app.accountId();
  let draftAuthority=app.accountId()===composerAuthority()?.id?composerAuthority():null;
  let draftDisposed=false;
  const ownsDraft=()=>!draftDisposed && app.accountId()===draftAccount && (draftAuthority===null || preferenceAccountIsCurrent(draftAuthority));
  let resumedWorkspaceDraft=false;
  createEffect(()=>{
    const account=app.accountId(),owner=composerAuthority();
    if(resumedWorkspaceDraft||!account||!owner||owner.id!==account||!preferenceAccountIsCurrent(owner))return;
    resumedWorkspaceDraft=true;
    const saved=props.resumeDraftId?listDrafts(account,localDraftAuthority()).find(draft=>draft.id===props.resumeDraftId):undefined;
    if(saved)resumeDraft(saved);
    props.onDraftIdChange?.(draftId);
  });
  createEffect(()=>{const value=subject();if(ownsDraft()&&draftAccount)props.onSubjectChange?.(value);});
  createEffect(() => {
    const account = app.accountId(),owner=composerAuthority();
    if(!draftAuthority&&owner?.id===account){draftAuthority=owner;setDrafts(listDrafts(account,owner.storageAuthority));}
    if (draftAccount === null && account !== null) { draftAccount = account; draftAuthority=owner; setDrafts(listDrafts(account,localDraftAuthority())); }
    if (account !== draftAccount || (draftAuthority && !preferenceAccountIsCurrent(draftAuthority))) {
      remoteWriter?.dispose();remoteAbort?.abort();
      clearTimeout(saveTimer); pendingDraft=undefined;
      setTo('');setCc('');setBcc('');setSubject('');setBody('');setBodyHtml('');setIdentityId('');setFromOverride('');setCustomFrom(false);
      for (const image of inlineImages()) if (image.previewUrl) URL.revokeObjectURL(image.previewUrl);
      setInlineImages([]);setAttachments([]);setDrafts([]);props.onClose();
    }
  });
  createEffect(() => {
    const account = app.accountId();
    composerAuthority();
    if (protectedDraft() || account === null || account !== draftAccount || !draftAuthority?.storageAuthority) return;
    const snapshot: StoredDraft = {
      id: draftId,
      accountId: account,
      storageAuthority:draftAuthority.storageAuthority,
      ...(remoteRevision()?{serverRevision:remoteRevision()!}:{}),
      ...(remoteStatus()==='uncertain'?{serverStatus:'uncertain' as const}:{}),
      identityId: identityId(),
      importance:importance(),
      ...(customFrom() ? {fromOverride: fromOverride()} : {}),
      bodyFormat: richMode() ? 'html' : 'plain',
      insertedSignatures: [...insertedSignatures()],
      attachments: attachments().map(file => ({...file})),
      to: to(),
      ...(cc().trim() !== '' ? { cc: cc() } : {}),
      ...(bcc().trim() !== '' ? { bcc: bcc() } : {}),
      ...(threading().inReplyTo.length > 0 ? { inReplyTo: threading().inReplyTo } : {}),
      ...(threading().references.length > 0 ? { references: threading().references } : {}),
      subject: subject(),
      bodyHtml: richMode() && editorApi() !== null ? bodyHtml() : plainToHtml(body()),
      bodyText: body(),
      savedAt: Date.now(),
      ...(requestReceipt() ? { requestReceipt: true } : {}),
      ...(requestDelivery() ? { requestDeliveryReceipt: true } : {}),
      ...(trackOpens() ? { openTracking: true } : {}),
      ...(inlineImages().length > 0 ? { inline: inlineImages().map(storedImage) } : {}),
    };
    clearTimeout(saveTimer);
    pendingDraft=snapshot;
    ensureRemoteWriter();
    if(draftHasContent(snapshot))remoteWriter?.update(draftEditorState(snapshot));
    saveTimer = setTimeout(() => {if(!sent && ownsDraft() && app.accountId()===snapshot.accountId && app.me()!==null) saveDraft(snapshot);}, 800);
  });
  createEffect(()=>{
    const protectedNow=protectedDraft();remoteWriter?.suspend(protectedNow);
    if(protectedNow){clearTimeout(saveTimer);pendingDraft=undefined;}
  });
  let remoteOwnerStarted=false;
  createEffect(()=>{const owner=composerAuthority();if(!remoteOwnerStarted&&owner?.storageAuthority&&ownsDraft()){remoteOwnerStarted=true;remoteAbort=new AbortController();void refreshRemoteDrafts();}});
  createEffect(()=>{
    remoteReady();ensureRemoteWriter();
    const values=settings();
    const mode=values['mail.draftSaveMode']?.value;
    remoteWriter?.configure({mode:(mode==='periodic'||mode==='manual'?mode:'realtime') as DraftSaveMode,debounceMs:Number(values['mail.draftDebounceMs']?.value??1000),intervalSecs:Number(values['mail.draftIntervalSecs']?.value??30)});
    if(pendingDraft&&draftHasContent(pendingDraft))remoteWriter?.update(draftEditorState(pendingDraft));
  });
  onCleanup(()=>{remoteWriter?.dispose();remoteAbort?.abort();});
  onCleanup(() => {clearTimeout(saveTimer);if(!sent && pendingDraft && ownsDraft() && app.me()!==null) saveDraft(pendingDraft);draftDisposed=true;});

  const recipientFields = {
    to: [to, setTo],
    cc: [cc, setCc],
    bcc: [bcc, setBcc],
  } as const;

  function checkNames():void {
    const missing:string[]=[];let resolved=0;
    for(const field of ['to','cc','bcc'] as const){const [read,write]=recipientFields[field];const result=checkRecipientNames(read(),app.contacts());write(result.value);missing.push(...result.unresolved);resolved+=result.resolved;}
    if(missing.length)setError(t('mail-compose-names-unresolved',{names:isolate(missing.join(', '))}));
    else {setError(null);toast.info(t('mail-compose-names-checked',{count:resolved}));}
  }

  /** Typing in To, Cc or Bcc: the token after the last separator drives the
   *  contact suggestions for that field. The directory lookup stays on To. */
  function onRecipientInput(field: RecipientField, value: string): void {
    recipientFields[field][1](value);
    const token = value.slice(tokenBoundary(value) + 1).trim();
    contactAc.setQuery(token);
    if (field === 'to') setGalToken(token);
    setAcField(token.length > 0 ? field : null);
  }

  /** Replace the in-progress recipient token of `field` with a resolved
   *  address (`, `-joined). */
  function insertRecipient(field: RecipientField, address: string): void {
    const [value, setValue] = recipientFields[field];
    const cut = tokenBoundary(value());
    const head = cut >= 0 ? `${value().slice(0, cut + 1)} ` : '';
    setValue(`${head}${address}, `);
    contactAc.reset();
    if (field === 'to') setGalToken('');
    setAcField(null);
  }

  /** The contact suggestions under `field`, while it is the one being typed in. */
  const suggestionList = (field: RecipientField): JSX.Element => (
    <Show when={acField() === field && contactAc.suggestions().length > 0}>
      <ul class="compose__ac" role="listbox" aria-label={t('mail-compose-contact-suggestions')}>
        <For each={contactAc.suggestions()}>
          {(s: ContactSuggestion) => (
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
                  insertRecipient(field, s.display);
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
  );

  /** Pick a GAL entry (plan §2.7). A person is inserted as a recipient; a
   *  distribution group is inserted AND offered for expand-before-send. */
  function pickGalEntry(entry: GalEntry): void {
    insertRecipient('to', entry.mail);
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
  async function onFilesPicked(fileList: FileList | File[] | null): Promise<void> {
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
          if (!ownsDraft() || app.accountId() !== acct) return;
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
    if(busy()||uploading())return;
    setBusy(true);
    try {
    remoteWriter?.suspend(true);
    if(remoteWriter?.isBusy())await remoteWriter.settled();
    if(!ownsDraft())return;
    setError(null);
    if(externalMode()==='confirm' && outside().length && reviewedRecipients()!==recipientReviewKey()){
      setError(t('mail-external-confirm-required'));return;
    }
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
      let textBody: string | undefined;
      if (enc !== null) {
        htmlBody = enc.armoredCiphertext;
        textBody = enc.armoredCiphertext;
      } else if (signOnly && session !== null) {
        htmlBody = await clearSignBody(getCryptoWorker(), session, body());
        textBody = htmlBody;
      } else {
        // The rich editor's serialized HTML, but only while the editor is
        // actually mounted. In plain-text mode, and when the editor's chunk is
        // still loading or failed to load, the user typed into the textarea and
        // `bodyHtml` does not hold their text — the body is built from `body`.
        const rich = richMode() && editorApi() !== null;
        htmlBody = rich ? bodyHtml() : '';
        if (!rich) textBody = body();
        const sig = autoSignature();
        if (sig !== null) {
          if (rich) htmlBody += `<br>${await signatureBlock(sig)}`;
          else textBody += `\n\n-- \n${await signaturePlain(sig)}`;
        }
      }
      const subjectToSend =
        enc !== null && cs !== null && cs.protectSubject && enc.encryptedSubjectApplied
          ? t('mail-compose-encrypted-subject')
          : subject();
      const attached = attachments();
      // Only the images the body still shows go out (none for ciphertext or a
      // clear-signed body, which carry no `cid:` reference).
      const inline = inlineAttachments(inlineImages(), referencedCids(htmlBody));
      if(requestDelivery()&&!deliveryAvailable())throw new Error(t('mail-compose-delivery-unavailable'));
      if (!ownsDraft()) throw new Error(t('mail-compose-account-changed'));
      await app.sendMessage({
        ...(remoteRevision()&&remoteStatus()!=='uncertain'&&remoteStatus()!=='conflict'?{accountDraft:{id:draftId,revision:remoteRevision()!}}:{}),
        to: to(),
        cc: cc(),
        bcc: bcc(),
        inReplyTo: threading().inReplyTo,
        references: threading().references,
        ...(initial?.source !== undefined ? { source: initial.source } : {}),
        ...(initial?.sources !== undefined ? { sources: initial.sources } : {}),
        subject: subjectToSend,
        htmlBody,
        ...(textBody !== undefined ? {textBody} : {}),
        identity: identity(),
        ...(importance()!=='normal'?{importance:importance()}:{}),
        ...(customFrom() ? {fromOverride: fromOverride()} : {}),
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
        ...(inline.length > 0 ? { inline } : {}),
        ...(receiptAvailable() && requestReceipt() ? { requestReadReceipt: true } : {}),
        ...(requestDelivery() ? { requestDeliveryReceipt: true } : {}),
        ...(trackingAvailable() && trackOpens() ? { openTracking: true } : {}),
      });
      if (!ownsDraft()) return;
      // W9: the composition was sent — drop its auto-saved draft.
      sent=true;clearTimeout(saveTimer);pendingDraft=undefined;
      deleteDraft(draftId, draftAccount,draftAuthority?.storageAuthority);
      props.onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : t('mail-compose-send-failed'));
    } finally {
      setBusy(false);
    }
    } finally {setBusy(false);if(!sent)remoteWriter?.suspend(protectedDraft());else remoteWriter?.dispose();}
  }

  return (
    <div
      class={props.embedded ? "workspace-compose" : "compose__backdrop"}
      role={props.embedded ? "region" : "dialog"}
      aria-modal={props.embedded ? undefined : "true"}
      aria-label={t('mail-compose-label')}
      ref={backdropEl}
      onKeyDown={onDialogKeyDown}
    >
      <form class="compose" classList={{'compose--compact':compact()}} onSubmit={(e) => void onSubmit(e)}>
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
              <Icon name="file"/> {t('mail-compose-drafts')}
            </button>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="open-recall"
              data-tooltip={t('mail-compose-recall-help')}
              aria-expanded={recallOpen()}
              onClick={() => (recallOpen() ? setRecallOpen(false) : openRecall())}
            >
              <Icon name="clock"/> {t('mail-compose-recall')}
            </button>
            <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t('mail-compose-close')} onClick={() => void closeWithDraftSave()}>
              <Icon name="close" />
            </button>
          </div>
        </header>

        <p role="status" aria-live="polite" class="compose__sign-unlock-note" data-testid="account-draft-status">{t(protectedDraft()?'mail-draft-protected':`mail-draft-status-${remoteStatus()}`)}</p>
        <Show when={['error','uncertain','conflict'].includes(remoteStatus())&&localDraftAuthority()}><button type="button" class="btn btn--ghost" onClick={()=>{allowRecoveryClose=true;props.onClose();}}>{t('mail-draft-close-recovery')}</button></Show>
        <Show when={remoteReady()&&!protectedDraft()}><button type="button" class="btn btn--ghost" disabled={remoteStatus()==='saving'||remoteStatus()==='uncertain'||remoteStatus()==='conflict'} onClick={()=>void remoteWriter?.flush()}>{t('mail-draft-save-now')}</button></Show>
        <Show when={draftsOpen()}><section aria-label={t('mail-draft-account-list')}><h3>{t('mail-draft-account-list')}</h3><p>{t('mail-draft-quota-help')}</p><Show when={remoteLoading()}><p role="status">{t('mail-draft-loading')}</p></Show><For each={remoteDrafts()}>{document=><div><button type="button" class="btn btn--ghost" disabled={remoteLoading()} onClick={()=>void resumeRemoteDraft(document.id)}>{document.summary.subject||t('mail-no-subject')}</button><button type="button" class="btn btn--ghost" disabled={remoteLoading()||document.status==='uncertain'} onClick={()=>setDiscardConfirmation(document)}>{t('mail-draft-discard-account')}</button></div>}</For></section></Show>
        <Show when={discardConfirmation()}>{document=><section role="alertdialog" aria-label={t('mail-draft-discard-confirm')}><p>{t('mail-draft-discard-confirm-detail',{subject:document().summary.subject||t('mail-no-subject')})}</p><button type="button" class="btn btn--ghost" disabled={remoteLoading()} onClick={()=>setDiscardConfirmation(null)}>{t('mail-compose-cancel')}</button><button type="button" class="btn" disabled={remoteLoading()} onClick={()=>{const row=document();setDiscardConfirmation(null);void discardRemoteDraft(row);}}>{t('mail-draft-discard-confirm')}</button></section>}</Show>
        <DraftsDrawer
          open={draftsOpen()}
          drafts={drafts}
          onResume={resumeDraft}
          onDelete={discardDraft}
          onClose={() => setDraftsOpen(false)}
        />
        <Show when={recallOpen()}>
          <RecallPanel submissions={app.cancelableOutbox} onRecall={recallSubmission} loading={app.outboxLoading() || recallBusy()} failed={recallError()} />
        </Show>

        <Show when={!localDraftAuthority()}><p role="status" class="compose__sign-unlock-note">{t('mail-compose-private-storage-unavailable')}</p></Show>
        <Show when={compact()}><section class="compose__quick-summary" aria-label={t('mail-quick-reply')}><p><strong>{t('mail-compose-to')}:</strong> {to()}<Show when={cc()}> · Cc: {cc()}</Show><Show when={bcc()}> · Bcc: {bcc()}</Show></p><p>{subject()}</p><button type="button" class="btn btn--ghost" onClick={() => setExpanded(true)}>{t('mail-quick-reply-expand')}</button></section></Show>
        <Show when={!compact()}>
          <Show when={showFrom()}><label class="field compose__from">
            <span>{t('mail-compose-from')}</span>
            <select value={customFrom() ? "__custom_from__" : identityId()} onChange={(e) => { const value=e.currentTarget.value; setCustomFrom(value === "__custom_from__"); if(value !== "__custom_from__") setIdentityId(value); }}>
              <option value="">{isolate(app.me()?.username ?? t('mail-compose-from-default'))}</option>
              <option value="__custom_from__">{t('mail-compose-from-custom')}</option>
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
          <Show when={customFrom()}><label class="field compose__from"><span>{t('mail-compose-from-alias')}</span><input type="email" required maxlength={254} autocomplete="off" value={fromOverride()} onInput={event=>setFromOverride(event.currentTarget.value)} aria-describedby={fromHelpId}/><small id={fromHelpId}>{t('mail-compose-from-alias-help')}</small></label></Show>

        <label class="field compose__to">
          <span>{t('mail-compose-to')}</span>
          <input
            type="text"
            ref={toInputEl}
            placeholder={t('mail-compose-to-placeholder')}
            autocomplete="off"
            value={to()}
            onInput={(e) => onRecipientInput('to', e.currentTarget.value)}
            onBlur={() => setAcField(null)}
          />
          {suggestionList('to')}
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

        <Show when={ccBccOpen()}>
          <label class="field compose__to">
            <span>{t('mail-compose-cc')}</span>
            <input
              type="text"
              autocomplete="off"
              value={cc()}
              onInput={(e) => onRecipientInput('cc', e.currentTarget.value)}
              onBlur={() => setAcField(null)}
            />
            {suggestionList('cc')}
          </label>
          <label class="field compose__to">
            <span>{t('mail-compose-bcc')}</span>
            <input
              type="text"
              autocomplete="off"
              value={bcc()}
              onInput={(e) => onRecipientInput('bcc', e.currentTarget.value)}
              onBlur={() => setAcField(null)}
            />
            {suggestionList('bcc')}
          </label>
        </Show>

        <label class="field">
          <span>{t('mail-compose-subject')}</span>
          <input type="text" value={subject()} onInput={(e) => setSubject(e.currentTarget.value)} />
        </label>
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
                    <Icon name="close" />
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>

        <Show when={externalMode()!=='off' && outside().length>0}><section class="compose-external" aria-label={t('mail-external-recipients')}><details><summary><Icon name="alert"/> {t('mail-external-count',{count:outside().length})}</summary><p>{t('mail-external-help')}</p><ul><For each={outside()}>{address=><li>{address}</li>}</For></ul></details><Show when={externalMode()==='confirm'}><label><input type="checkbox" checked={reviewedRecipients()===recipientReviewKey()} onChange={event=>setReviewedRecipients(event.currentTarget.checked?recipientReviewKey():'')}/>{t('mail-external-reviewed')}</label></Show></section></Show>
        <section class="compose-ribbon" aria-label={t('mail-compose-ribbon')}>
          <Show when={richMode()&&!sourceMode()}><ComposerHistory api={editorApi()}/></Show>
          <div class="compose-rt__tabs" role="tablist"><For each={richMode()&&!sourceMode()?['format','insert','table','security','delivery'] as const:['format','insert','security','delivery'] as const}>{tab=><button type="button" role="tab" aria-selected={ribbonTab()===tab} onClick={()=>setRibbonTab(tab)}>{t(`mail-compose-ribbon-${tab}`)}</button>}</For></div>
          <div class="compose-ribbon__panel" hidden={ribbonTab()!=='format'} inert={ribbonTab()!=='format'}>
            <div class="compose-ribbon__group"><span>{t('mail-compose-ribbon-recipients')}</span><button type="button" class="btn btn--ghost" onClick={checkNames} data-tooltip={t('mail-compose-check-names-help')}><Icon name="check"/>{t('mail-compose-check-names')}</button><button type="button" class="btn btn--ghost" data-testid="compose-show-cc-bcc" aria-pressed={ccBccOpen()} onClick={()=>setCcBccOpen(!ccBccOpen())}>{t('mail-compose-show-cc-bcc')}</button><button type="button" class="btn btn--ghost" aria-pressed={showFrom()} onClick={()=>setFromVisibleOverride(!showFrom())}>{t('mail-compose-from')}</button></div>
            <label class="compose-ribbon__group"><span>{t('mail-compose-ribbon-message-type')}</span><select data-testid="format-toggle" value={!richMode()?'plain':sourceMode()?'html':'rich'} onChange={event=>changeFormat(event.currentTarget.value)}><option value="plain">{t('mail-compose-format-plain')}</option><option value="rich">{t('mail-compose-format-rich')}</option><option value="html">{t('mail-compose-html-source')}</option></select></label>
          </div>
          <div class="compose-ribbon__panel" hidden={ribbonTab()!=='insert'} inert={ribbonTab()!=='insert'}>        <InsertAccountItem onFiles={onFilesPicked} disabled={uploading()||busy()}/><SignaturePicker signatures={signatures} onInsert={insertSignature} />

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

</div>
          <div class="compose-ribbon__panel" hidden={ribbonTab()!=='security'} inert={ribbonTab()!=='security'}>        {/* Crypto + DLP (plan §2.5): encrypt/sign toggles, the live E2EE/TLS/mixed
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
              onRequestSigningKey={() => {setRibbonTab('security');setUnlockOpen(true);}}
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

</div>
          <div class="compose-ribbon__panel" hidden={ribbonTab()!=='delivery'} inert={ribbonTab()!=='delivery'}>
            <label class="field"><span>{t('mail-compose-importance')}</span><select value={importance()} onChange={event=>setImportance(event.currentTarget.value as 'high'|'normal'|'low')}><For each={['high','normal','low'] as const}>{value=><option value={value}>{t(`mail-compose-importance-${value}`)}</option>}</For></select></label>        <SendOptions
          deliveryAvailable={deliveryAvailable()}
          requestDelivery={requestDelivery()}
          onRequestDelivery={setRequestDelivery}
          receiptAvailable={receiptAvailable()}
          requestReceipt={requestReceipt()}
          onRequestReceipt={setRequestReceipt}
          trackingAvailable={trackingAvailable()}
          tracking={trackOpens()}
          onTracking={setTrackOpens}
        />

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

</div>
        </section>
        <div class="field field--grow">
          <div class="compose__body-head">

            <span>{t('mail-compose-body')}</span>

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
                <Show when={!sourceMode()} fallback={<HtmlSourceEditor initialHtml={bodyHtml()} externalText={body} owned={ownedCids} onChange={onEditorChange} onReady={setEditorApi}/>}><RichTextEditor
                  externalHistory
                  ribbonTab={()=>ribbonTab() === 'format' || ribbonTab() === 'insert' || ribbonTab() === 'table' ? ribbonTab() as 'format'|'insert'|'table' : null}
                  initialHtml={bodyHtml()}
                  externalText={body}
                  ariaLabel={t('mail-compose-body')}
                  onChange={onEditorChange}
                  onReady={setEditorApi}
                  inline={{
                    owned: ownedCids,
                    preview: (cid) => inlineImages().find((i) => i.cid === cid)?.previewUrl ?? null,
                    upload: uploadInline,
                  }}
                /></Show>
              </Suspense>
            </AsyncBoundary>
          </Show>
          <Show when={inlineError()}>
            <p class="login__error" role="alert">
              {inlineError()}
            </p>
          </Show>
        </div>

        {/* Shown exactly when the send will append it (see `autoSignature`). */}
        <Show when={autoSignature()}>
          {(sig) => (
            <p class="compose__signature" data-testid="compose-auto-signature">
              — {isolate(sig().text)}
            </p>
          )}
        </Show>

        {/* Asked once per composer, when a send would put the decrypted text of
            an encrypted original on the wire unencrypted. Confirming sends. */}
        <Show when={plaintextConfirmOpen()}>
          <section
            class="compose__sign-unlock"
            role="alertdialog"
            aria-labelledby={`${composeId}-plaintext-title`}
            aria-describedby={`${composeId}-plaintext-body`}
            data-testid="compose-plaintext-confirm"
          >
            <p id={`${composeId}-plaintext-title`} class="compose__sign-unlock-note">
              <strong>{t('mail-compose-plaintext-confirm-title')}</strong>
            </p>
            <p id={`${composeId}-plaintext-body`} class="compose__sign-unlock-note">
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
          <button type="button" class={`btn btn--ghost ${a11y.focusable}`} onClick={() => void closeWithDraftSave()}>
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
