import {ResponseTemplates} from './compose/ResponseTemplates.tsx';
import {assignedSignature,signatureVariants} from './compose/signatureVariants.ts';
import {registerLiveReplyDraft} from './compose/liveReplyDrafts.ts';
import {htmlMessageDocument} from './compose/htmlMessage.ts';
import {attachmentHashManifest,publicIdentityFile,liveIdentityKey} from './compose/outgoingExtras.ts';
import {checkRecipientNames} from './compose/recipientNames.ts';
import {EditorCounts} from './compose/EditorCounts.tsx';
import {InsertAccountItem} from './compose/InsertAccountItem.tsx';
import {DraftsPopup} from './compose/DraftsPopup.tsx';
import {SendLaterMenu} from './compose/SendLaterMenu.tsx';
import {accountDraftApi,draftEditorState,localAccountDraft,type AccountDraftDocument} from '../api/accountDrafts.ts';
import {createDraftAutosave,type DraftSaveStatus,type DraftSaveMode} from './compose/accountDraftAutosave.ts';
import {toast} from './toast/toast.ts';
import {observePreferenceAccount,preferenceAccount, preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import { settings, writeSetting } from '../api/settings.ts';
import { supportsDeliveryReceipts } from '../api/deliveryReceipts.ts';
import { externalRecipients } from './compose/recipientTrust.ts';
import { CHECK_KEYS, externalDomains, invalidRecipients, mentionsAttachment, planSend, resolvePolicy, type CheckId, type SendIssue } from './compose/sendChecks.ts';
import { isRecipientSeparator, type RecipientSeparator } from './compose/recipientChips.ts';
import { RecipientField as RecipientTokenField } from './compose/RecipientField.tsx';
import { createAddressContactCard } from './AddressContactCard.tsx';
import { Icon } from './Icon.tsx';
import {
  createEffect,
  batch,
  on,
  createMemo,
  createSignal,
  createUniqueId,
  lazy,
  For,
  Show,
  Suspense,
  onMount,
  onCleanup,
  untrack,
  type JSX,
} from 'solid-js';
import { useApp } from '../state/context.ts';
import { rawT, t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import { AsyncBoundary } from './ErrorBoundary.tsx';
import type { RichTextApi } from './compose/RichTextEditor.tsx';
import {RibbonGroup,RibbonItem} from './bars/index.ts';
import {BodyModeGroup,EditorRibbon,createEditorRibbon,type BodyMode} from './compose/EditorRibbon.tsx';
import {RibbonPopover} from './compose/ribbonParts.tsx';
import {
  SignaturePicker,
  RecallPanel,
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
  embedSignatureImages,
  inlineAttachments,
  inlineRefusal,
  newCid,
  referencedCids,
  storedImage,
  type InlineImage,
} from './compose/inlineImages.ts';
import { clientPolicy, loadClientPolicy } from '../api/policy.ts';
import { buildDownloadUrl, fetchObjectUrl } from '../viewers/attachments.ts';
import {blobUrlToDataUrl} from '../viewers/sandbox.ts';

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
  recipientToken,
  replaceRecipientToken,
  type ContactSuggestion,
} from '../modules/contacts/autocomplete.ts';
import { ComposeCrypto, cryptoDraftSource, type ComposeCryptoState } from './compose-crypto.tsx';
import type { ComposeInitial } from './compose/reply.ts';
import { chooseReplyIdentity, isReplyFromMode, type ReplyFromMode, type ReplyIdentityChoice } from './compose/replyIdentity.ts';
import {
  assertSigningKeyOwner,
  clearSignBody,
  createJmapDlpScan,
  createJmapKeyLookup,
  type DlpScanFn,
  type KeyLookupFn,
  type SigningSession,
} from './compose/crypto-jmap.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { isMailbox, parseRecipients, uploadBlob } from '../api/jmap.ts';
import { CAP_CORE } from '../api/jmap-types.ts';
// V7 last-mile mailbox integration (plan §2.7/§14, e14b). All ADDITIVE: each block
// is gated so a deployment with no directory / disabled Assist / no Nextcloud sees
// the exact same composer as before.
import { DirectorySearch, GroupExpand, type GalEntry } from '../modules/directory/index.ts';
import { ComposerTools, Dictation } from '../modules/assist/index.ts';
import { NextcloudAttach, ShareLinkComposer, type AttachedFile, type ShareLink } from '../modules/nextcloud/index.ts';
import { ConfirmDialog } from '../screens/Admin/shell/ConfirmDialog.tsx';
import { featureOn } from '../state/features.ts';
import { mailFeature, mailFeatureOffTitle } from '../state/mailFeatures.ts';

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
  /** `handler(true)`: the user already chose to discard (a bulk tab close), so it does not ask. `dirty`: closing now would lose the message, asked without prompting. */
  registerBeforeClose?: (handler:(discard?:boolean)=>Promise<boolean>,dirty:()=>boolean)=>void;
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
  let autoBccValue='';let bccEdited=false;
  // The Cc and Bcc fields stay folded away until asked for, or until one of
  // them holds something (a reply-all, a resumed draft).
  const [ccBccOpen, setCcBccOpen] = createSignal((initial?.cc ?? '') !== '');
  // The ids a reply carries (`In-Reply-To`, `References`). Not editable; a
  // resumed draft brings its own.
  const [threading, setThreading] = createSignal<{ inReplyTo: string[]; references: string[] }>({
    inReplyTo: initial?.inReplyTo ?? [],
    references: initial?.references ?? [],
  });
  const [draftSource,setDraftSource]=createSignal(initial?.source);
  const [replyContext,setReplyContext]=createSignal(initial?.source?.keyword==='$answered'||initial?.mode==='reply'||initial?.mode==='reply-all');
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
  // t98: a profile without rich text composes plain text only (the server refuses an HTML body).
  createEffect(() => { if (!featureOn('compose.rich_text') && richMode()) untrack(() => changeFormat('plain')); });
  // New messages use the native HTML visual editor. Saved drafts still restore
  // their explicit structured HTML or plain-text editor in resumeDraft.
  const [sourceMode,setSourceMode]=createSignal(true);
  const [editorApi, setEditorApi] = createSignal<RichTextApi | null>(null);
  // W9 drafts drawer + W10 recall panel visibility, and the loaded draft list.
  const [draftsOpen, setDraftsOpen] = createSignal(false);
  const [recallOpen, setRecallOpen] = createSignal(false);
  const [recallError,setRecallError]=createSignal(false);
  const [recallBusy,setRecallBusy]=createSignal(false);
  const [drafts, setDrafts] = createSignal<StoredDraft[]>([]);
  // Stable id for THIS composer's auto-saved draft (W9).
  let draftId = newDraftId();
  const [liveDraftVersion,setLiveDraftVersion]=createSignal(0);
  let releaseLiveReply=()=>{};
  const [remoteRevision,setRemoteRevision]=createSignal<string|null>(null);
  const [remoteReady,setRemoteReady]=createSignal(false);
  const [remoteStatus,setRemoteStatus]=createSignal<DraftSaveStatus>('idle');
  const [remoteDrafts,setRemoteDrafts]=createSignal<AccountDraftDocument[]>([]);
  const [remoteListFailed,setRemoteListFailed]=createSignal(false);
  const [remoteLoading,setRemoteLoading]=createSignal(false);
  const [discardCurrentOpen,setDiscardCurrentOpen]=createSignal(false);
  const [discardCurrentBusy,setDiscardCurrentBusy]=createSignal(false);
  const [discardConfirmation,setDiscardConfirmation]=createSignal<AccountDraftDocument|null>(null);
  // Closing a composer whose content nothing would keep asks first (t42).
  const [closePrompt,setClosePrompt]=createSignal<{body:string;answer:(leave:boolean)=>void}|null>(null);
  let remoteWriter:ReturnType<typeof createDraftAutosave>|undefined;
  let remoteWriterId='';
  let resumedUncertain=false;
  let remoteAbort:AbortController|undefined;

  const [identityId, setIdentityId] = createSignal<string>('');
  let initialIdentityLoad:Promise<void>|undefined;
  function loadInitialIdentities():Promise<void>{
    if(!initialIdentityLoad){
      const loading=app.loadIdentities({strict:true});initialIdentityLoad=loading;
      void loading.catch(cause=>{if(initialIdentityLoad===loading)initialIdentityLoad=undefined;if(ownsDraft()&&!(cause instanceof DOMException&&cause.name==='AbortError'))setError(t('mail-compose-identities-failed'));});
    }
    return initialIdentityLoad;
  }
  const [fromOverride, setFromOverride] = createSignal('');
  // mail.replyFromMode: a reply or forward starts from the identity the original was sent to.
  const [autoPick, setAutoPick] = createSignal<ReplyIdentityChoice | null>(null);
  let identityTouched = false;
  let autoPicked = false;
  const pickIdentity = (value: string): void => { identityTouched = true; setAutoPick(null); setIdentityId(value); };
  const [importance,setImportance]=createSignal<'high'|'normal'|'low'>('normal');
  const [customFrom, setCustomFrom] = createSignal(false);
  const fromHelpId = createUniqueId();
  // The composer ribbon (t38; shared with the signature editor, t49): the open
  // tab, and the slots where the editor's groups join each tab.
  const ribbon=createEditorRibbon<'security'|'delivery'>('format');
  const ribbonTab=ribbon.tab,setRibbonTab=ribbon.setTab,showRibbonTab=ribbon.show;
  createEffect(()=>{if(unlockOpen())showRibbonTab('security');});
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
  const [acIndex,setAcIndex]=createSignal(-1);
  const [recipientComposing,setRecipientComposing]=createSignal(false);
  const recipientInputs:Partial<Record<RecipientField,HTMLInputElement>>={};
  const recipientCarets:Partial<Record<RecipientField,number>>={};
  // V7 GAL (plan §2.7): the in-progress recipient token also drives a directory
  // autocomplete as an ADDITIONAL source beside contacts. `pickedGroup` holds a
  // distribution group the sender may expand-before-send into its leaf recipients.
  const [galToken, setGalToken] = createSignal('');
  const [pickedGroup, setPickedGroup] = createSignal<GalEntry | null>(null);
  // V7 Nextcloud attach (plan §18.4): materialised attachments + the picker toggle.
  const [attachments, setAttachments] = createSignal<AttachedFile[]>(initial?.attachments ?? []);
  const [includeFileHashes,setIncludeFileHashes]=createSignal(false);
  const [attachPublicKey,setAttachPublicKey]=createSignal(false);
  const [sentCopyFolder,setSentCopyFolder]=createSignal('');
  const replySource=initial?.mode==='reply'||initial?.mode==='reply-all'?app.openEmail():null;
  const replyMailboxIds=initial?.sourceMailboxIds??(replySource&&replySource.id===initial?.source?.emailId?Object.keys(replySource.mailboxIds):[]);
  const replySourceFolder=initial?.sourceMailboxId??app.selectedMailboxId();
  const effectiveSentCopy=()=>{
    if(sentCopyFolder())return sentCopyFolder();
    if(settings()['mail.replyCopyToOriginal']?.value!==true||!replyMailboxIds.length)return '';
    const candidates=app.mailboxes().filter(box=>box.myRights?.mayAddItems!==false&&replyMailboxIds.includes(box.id));
    return candidates.find(box=>box.id===replySourceFolder)?.id??candidates.find(box=>!['trash','junk','drafts','sent'].includes(box.role??''))?.id??'';
  };
  const [hashResult,setHashResult]=createSignal<{key:string;manifest:string;error:boolean}|null>(null);
  const [ncOpen, setNcOpen] = createSignal(false);
  // Files over the attachment limit, offered as a Nextcloud share link instead
  // (t39 L7) when the account has a linked Nextcloud.
  const [oversized, setOversized] = createSignal<File[]>([]);
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
  const [inlineImages, setInlineImages] = createSignal<InlineImage[]>((initial?.inline??[]).map(image=>({...image,previewUrl:null})));
  const [inlineError, setInlineError] = createSignal<string | null>(null);
  const ownedCids = createMemo<ReadonlySet<string>>(() => new Set(inlineImages().map((i) => i.cid)));
  // The original's images, held back by the reply-images policy until the user decides.
  const imageOffer = initial?.imageOffer;
  const [offerState, setOfferState] = createSignal<'ask'|'loading'|'done'|'dismissed'|'cancelled'|'error'>(imageOffer ? 'ask' : 'dismissed');
  let offerAbort: AbortController | undefined;
  async function includeQuoteImages(trust: boolean): Promise<void> {
    if (!imageOffer || offerState() === 'loading') return;
    const controller = new AbortController(); offerAbort = controller; setOfferState('loading');
    try {
      if (trust) await imageOffer.trust().catch(() => undefined);
      const result = await imageOffer.include(controller.signal);
      if (controller.signal.aborted) { setOfferState('cancelled'); return; }
      if (!ownsDraft()) return;
      setInlineImages((cur) => [...cur, ...result.inline.map((image) => ({ ...image, previewUrl: null }))]);
      const api = editorApi();
      if (bodyHtml() === initial?.bodyHtml) {
        if (api?.replaceHtml) api.replaceHtml(result.bodyHtml); else setBodyHtml(result.bodyHtml);
        setBody(result.bodyText);
      } else if (api) api.appendHtml(result.bodyHtml);
      else { setBodyHtml((cur) => cur + result.bodyHtml); setBody((cur) => cur + result.bodyText); }
      setOfferState(result.inline.length > 0 ? 'done' : 'error');
    } catch { setOfferState(controller.signal.aborted ? 'cancelled' : 'error'); }
  }
  function cancelQuoteImages(): void { offerAbort?.abort(); setOfferState('cancelled'); }
  onCleanup(() => offerAbort?.abort());
  const [downloadUrl, setDownloadUrl] = createSignal<string | null>(null);
  // Receipt preferences supply initial choices; drafts keep their own choices.
  // Read-receipt headers need the engine; DSN needs this account's capability.
  const [receiptAvailable, setReceiptAvailable] = createSignal(false);
  const [requestReceiptValue, setRequestReceiptValue] = createSignal(settings()['mail.requestReadReceipt']?.value===true);
  const requestReceipt=()=>settings()['mail.requestReadReceipt']?.locked?settings()['mail.requestReadReceipt']?.value===true:requestReceiptValue();
  let receiptInitialized=settings()['mail.requestReadReceipt']!==undefined;
  const setRequestReceipt=(value:boolean):void=>{receiptInitialized=true;setRequestReceiptValue(value);};
  createEffect(()=>{const entry=settings()['mail.requestReadReceipt'];if(!receiptInitialized && entry){receiptInitialized=true;setRequestReceiptValue(entry.value===true);}});
  const [deliveryCapable,setDeliveryCapable]=createSignal(false);
  const deliveryAvailable=()=>deliveryCapable() && settings()['receipts.dsnAllowed']?.value===true;
  const [requestDeliveryValue,setRequestDeliveryValue]=createSignal(false);
  const requestDelivery=()=>settings()['mail.requestDeliveryReceipt']?.locked?settings()['mail.requestDeliveryReceipt']?.value===true:requestDeliveryValue();
  let deliveryEdited=false;let deliveryDisposed=false;
  const setRequestDelivery=(value:boolean):void=>{deliveryEdited=true;setRequestDeliveryValue(value);};
  createEffect(()=>{const entry=settings()['mail.requestDeliveryReceipt'];if(!deliveryEdited&&entry&&deliveryAvailable())setRequestDeliveryValue(entry.value===true);});
  const [trackOpensValue, setTrackOpensValue] = createSignal(false);
  let trackingEdited=false,trackingInitialized=false;
  const trackingAllowed=()=>clientPolicy().openTracking&&featureOn('compose.open_tracking')&&settings()['cap.compose.open_tracking']?.value!==false;
  const trackOpens=()=>trackingAllowed()&&(settings()['mail.openTracking']?.locked?settings()['mail.openTracking']?.value===true:trackOpensValue());
  const setTrackOpens=(value:boolean):void=>{if(!trackingAllowed()||settings()['mail.openTracking']?.locked)return;trackingEdited=true;setTrackOpensValue(value);};
  createEffect(()=>{const entry=settings()['mail.openTracking'];if(entry&&!trackingInitialized){trackingInitialized=true;if(!trackingEdited)setTrackOpensValue(entry.value===true);}});
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
  // Bumped on every mailbox switch; an unlock started before one is discarded.
  let signingGeneration = 0;
  let signingDisposed = false;
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
  const contactAc = createContactAutocomplete(() => app.contacts(),{
    lookup:prefix=>app.autocompleteContacts?.(prefix)??Promise.resolve([]),
    scope:()=>[app.accountId(),acField(),app.contacts()],
  });

  // Dialog focus management (self-contained per t8-e1; no import from the
  // e3-owned a11y primitives). On open: pull the mail catalog, remember the
  // trigger, and move focus into the composer. On close: restore focus. Escape
  // closes; Tab is trapped inside the dialog.
  let backdropEl: HTMLDivElement | undefined;
  let toInputEl: HTMLInputElement | undefined;
  let previouslyFocused: HTMLElement | null = null;
  /** Dialogs open in the document that holds the composer, never hidden in the opener. */
  const dialogMount=():Node|undefined=>backdropEl?.ownerDocument.body;

  function focusableIn(root: HTMLElement): HTMLElement[] {
    return Array.from(
      root.querySelectorAll<HTMLElement>(
        'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ),
    ).filter((el) => el.offsetParent !== null || el === document.activeElement);
  }

  function onDialogKeyDown(e: KeyboardEvent): void {
    if (props.embedded || e.defaultPrevented || e.isComposing || recipientComposing()) return;
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
    void loadCatalog('nextcloud'); // t112: the inserted share link text
    previouslyFocused = document.activeElement as HTMLElement | null;
    if (props.active !== false) toInputEl?.focus();
    void loadInitialIdentities();
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
        if(initial?.inline?.length)void loadPreviews(inlineImages());
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
    signingDisposed = true;
    dropSigningSession();
  });

  /** Lock and forget the signing key and its passphrase (t87 D11). */
  function dropSigningSession(): void {
    const s = signingSession();
    setSigningSession(null);
    setUnlockPass('');
    if (s !== null) void app.lockKeyRef(s.keyRef).catch(() => undefined);
  }

  // A mailbox switch (t87 D11) drops the signing session: the next mailbox
  // signs with its own key, unlocked with its own passphrase.
  createEffect(on([() => app.accountId(), () => app.keysEpoch()], () => {
    signingGeneration += 1;
    dropSigningSession();
  }, { defer: true }));

  /** Unlock the sending key ONCE per composer session (decision flag 2): find the
   *  own PGP private bundle (as Reader.tsx::decryptNow does), unlock it in the
   *  worker to get a session keyRef, and cache the ref + bundle + passphrase so
   *  signed sends reuse it without re-prompting. */
  async function unlockSigningKey(e: Event): Promise<void> {
    e.preventDefault();
    setUnlockError(null);
    setUnlocking(true);
    const generation = signingGeneration;
    const composing = app.accountId();
    const passphrase = unlockPass();
    try {
      if (app.ownKeys().length === 0) await app.loadKeys();
      if (generation !== signingGeneration || composing !== app.accountId()) return;
      // Only an own key of the composing mailbox (t87 D11).
      const own = app.ownKeys().find(
        (k) => k.kind === 'pgp' && k.encryptedPrivateBackup !== null && app.keyOwnerAccount(k) === composing && (!identity()?.pgpKeyId||k.id===identity()?.pgpKeyId) && liveIdentityKey(k,customFrom()?fromOverride():identity()?.email??app.me()?.username??''),
      );
      const bundle = own?.encryptedPrivateBackup ?? null;
      if (own === undefined || bundle === null || composing === null) throw new Error(t('mail-compose-sign-no-key'));
      const { keyRef, accountId } = await app.unlockOwnKey(own, passphrase);
      const session: SigningSession = { keyRef, bundle, passphrase, accountId };
      if (signingDisposed || generation !== signingGeneration || composing !== app.accountId()) {
        void app.lockKeyRef(keyRef).catch(() => undefined);
        return;
      }
      try {
        assertSigningKeyOwner(session, composing);
      } catch (refused) {
        void app.lockKeyRef(keyRef).catch(() => undefined);
        throw refused instanceof Error && refused.name === 'SigningKeyOwnerError' ? new Error(t('mail-compose-sign-no-key')) : refused;
      }
      setSigningSession(session);
      setUnlockPass('');
      setUnlockOpen(false);
    } catch (err) {
      if (err instanceof DOMException && err.name === 'AbortError') return;
      setUnlockError(err instanceof Error ? err.message : t('mail-compose-sign-unlock-failed'));
    } finally {
      setUnlocking(false);
    }
  }

  createEffect(() => {
    const received = initial?.received;
    const list = app.identities();
    if (autoPicked || identityTouched || !received || initial?.mode === 'new' || list.length === 0 || identityId() !== '' || customFrom()) return;
    const configured = settings()['mail.replyFromMode']?.value;
    const mode: ReplyFromMode = isReplyFromMode(configured) ? configured : 'match';
    autoPicked = true;
    const choice = chooseReplyIdentity(list, received, mode);
    if (choice.identityId === null) return;
    setIdentityId(choice.identityId);
    setAutoPick(choice);
  });
  const identity = createMemo(() => app.identities().find((i) => i.id === identityId()) ?? null);
  createEffect(on(()=>[identity()?.id,identity()?.autoBcc,identity()?.pgpKeyId,customFrom(),fromOverride()] as const,()=>{
    signingGeneration+=1;dropSigningSession();
    if(!bccEdited&&bcc()===autoBccValue){autoBccValue=customFrom()?'':identity()?.autoBcc??'';setBcc(autoBccValue);if(autoBccValue)setCcBccOpen(true);}
  }));
  const externalMode = () => settings()['mail.externalRecipientWarning']?.value ?? 'warn';
  const outside = createMemo(() => {
    const trusted=settings()['mail.trustedRecipientDomains']?.value;
    return externalRecipients([to(),cc(),bcc()],identity()?.email ?? app.identities()[0]?.email ?? app.me()?.username ?? '',Array.isArray(trusted)?trusted.filter((domain):domain is string=>typeof domain==='string'):[]);
  });
  const recipientReviewKey=()=>JSON.stringify([outside(),identity()?.email,externalMode()]);
  const [reviewedRecipients,setReviewedRecipients]=createSignal('');
  // Pre-send checks (invalid recipients, empty subject, missing attachment, external
  // recipients): each has a policy; only `deny` ever stops a send.
  const policyOf=(id:CheckId)=>resolvePolicy(settings()[CHECK_KEYS[id]]?.value);
  const autoChip=()=>settings()['mail.compose.autoChipRecipients']?.value!==false;
  const separator=():RecipientSeparator=>{const value=settings()['mail.compose.recipientSeparator']?.value;return isRecipientSeparator(value)?value:'comma';};
  const [focusedRecipient,setFocusedRecipient]=createSignal<RecipientField|null>(null);
  const fieldValue=(field:RecipientField)=>field==='to'?to():field==='cc'?cc():bcc();
  /** Finished recipients only: the token still being typed in the focused field is not judged yet. */
  const liveInvalid=createMemo(()=>{
    if(policyOf('invalidRecipients')==='allow')return [] as string[];
    const focus=focusedRecipient();
    return invalidRecipients((['to','cc','bcc'] as const).map(field=>field===focus?fieldValue(field).replace(/[^,;]*$/u,''):fieldValue(field)));
  });
  const subjectAlert=()=>policyOf('emptySubject')!=='allow'&&subject().trim()===''&&body().trim()!=='';
  const [dismissedExternal,setDismissedExternal]=createSignal<string[]>([]);
  const externalHidden=()=>outside().every(address=>dismissedExternal().includes(address));
  let subjectInput:HTMLInputElement|undefined;
  const [checkPrompt,setCheckPrompt]=createSignal<{asked:SendIssue[];rememberable:CheckId[]}|null>(null);
  const [rememberChecked,setRememberChecked]=createSignal(false);
  let checkResolve:((answer:'send'|'review')=>void)|undefined;
  onCleanup(()=>checkResolve?.('review'));
  function collectIssues():SendIssue[] {
    const issues:SendIssue[]=[];
    const bad=invalidRecipients([to(),cc(),bcc()]);
    if(bad.length)issues.push({id:'invalidRecipients',details:bad});
    if(subject().trim()==='')issues.push({id:'emptySubject',details:[]});
    if(attachments().length===0&&inlineImages().length===0&&!attachPublicKey()&&mentionsAttachment(body()))issues.push({id:'missingAttachment',details:[]});
    const external=outside().filter(isMailbox);
    if(external.length)issues.push({id:'externalRecipients',details:externalDomains(external)});
    return issues;
  }
  function issueText(issue:SendIssue,denied=false):string {
    const list=isolate(issue.details.join(', '));
    const key=`mail-check-${denied?'denied-':''}${issue.id==='invalidRecipients'?'invalid':issue.id==='emptySubject'?'subject':issue.id==='missingAttachment'?'attachment':'external'}`;
    return t(key,{list});
  }
  function focusIssue(issue:SendIssue|undefined):void {
    if(!issue)return;
    if(issue.id==='emptySubject'){subjectInput?.focus();return;}
    if(issue.id==='missingAttachment')return;
    const needle=issue.id==='invalidRecipients'?issue.details[0]??'':'';
    const field=(['to','cc','bcc'] as const).find(f=>needle?fieldValue(f).includes(needle):fieldValue(f).trim()!=='')??'to';
    if(field!=='to')setCcBccOpen(true);
    const input=recipientInputs[field];input?.focus();
    const at=needle&&!autoChip()?fieldValue(field).indexOf(needle):-1;
    if(at>=0)input?.setSelectionRange(at,at+needle.length);
  }
  function askChecks(asked:SendIssue[],rememberable:CheckId[]):Promise<'send'|'review'> {
    return new Promise(resolve=>{checkResolve=resolve;setRememberChecked(false);setCheckPrompt({asked,rememberable});});
  }
  function answerChecks(answer:'send'|'review'):void {
    const prompt=checkPrompt();const remember=rememberChecked();
    setCheckPrompt(null);
    if(answer==='send'&&remember&&prompt)for(const id of prompt.rememberable)void writeSetting(CHECK_KEYS[id],id==='externalRecipients'?'off':'allow').catch(()=>undefined);
    const resolve=checkResolve;checkResolve=undefined;resolve?.(answer);
  }
  /** True when the send may go on. One combined dialog covers every `warn` check. */
  async function runSendChecks():Promise<boolean> {
    const ids:CheckId[]=['invalidRecipients','emptySubject','missingAttachment','externalRecipients'];
    const policies=Object.fromEntries(ids.map(id=>[id,policyOf(id)])) as Record<CheckId,ReturnType<typeof policyOf>>;
    const locked=Object.fromEntries(ids.map(id=>[id,settings()[CHECK_KEYS[id]]?.locked===true])) as Record<CheckId,boolean>;
    const plan=planSend(collectIssues(),policies,locked);
    if(plan.denied.length){setError(plan.denied.map(issue=>issueText(issue,true)).join(' '));focusIssue(plan.denied[0]);return false;}
    if(plan.asked.length&&await askChecks(plan.asked,plan.rememberable)==='review'){focusIssue(plan.asked[0]);return false;}
    return true;
  }

  /** Plain text → the same minimal HTML the plain-text send path has always
   *  produced (escaped, newlines as `<br>`). Used to seed the rich editor when
   *  switching plain → rich, so the typed text carries over. No ProseMirror here
   *  (that would drag the lazy editor's libraries onto the entry chunk). */
  function plainToHtml(text: string): string {
    return `<p>${escapeHtml(text).replace(/\n/g, '<br>')}</p>`;
  }

  /** The rich editor reports HTML (for the send) + a plain-text projection (for
   *  crypto/DLP/dictation, which read `body()`). */
  function insertResponseTemplate(text:string,replace:boolean):void {
    if(busy()||!ownsDraft())return;
    const api=editorApi();
    if(richMode()&&api){if(replace)api.replaceHtml?.(plainToHtml(text));else api.appendHtml(plainToHtml(text));return;}
    const next=replace?text:body()+(body()?'\n\n':'')+text;setBody(next);setBodyHtml(plainToHtml(next));
  }
  function onEditorChange(html: string, text: string): void {
    setBodyHtml(html);
    setBody(text);
  }

  /** Toggle the body between the rich editor and a plain-text / format=flowed
   *  textarea. Rich → plain just reveals the already-synced text; plain → rich
   *  re-seeds the editor from that text so the content round-trips. */
  function changeFormat(mode:BodyMode):void {
    const api=editorApi();
    if(sourceMode()&&mode!=='html'&&api?.whenReady&&featureOn('compose.rich_text')){
      void api.whenReady().then(()=>{if(ownsDraft()&&editorApi()===api)applyFormat(mode);},()=>setError(t('mail-compose-html-failed')));
      return;
    }
    applyFormat(mode);
  }
  function applyFormat(mode:BodyMode):void {
    if(!richMode() && mode!=='plain')setBodyHtml(plainToHtml(body()));
    if(mode!=='rich'&&ribbonTab()==='table')setRibbonTab('format');
    if(sourceMode()!==(mode==='html'))setEditorApi(null);
    setSourceMode(mode==='html');
    setRichMode(mode!=='plain');
  }


  const signatures=createMemo(()=>signatureVariants(app.identities(),{new:t('mail-compose-signature-new'),replies:t('mail-compose-signature-replies')}));

  const hashKey=createMemo(()=>JSON.stringify([includeFileHashes(),attachments(),app.accountId(),composerAuthority(),downloadUrl()]));
  const hashesReady=()=>!includeFileHashes()||(hashResult()?.key===hashKey()&&!hashResult()?.error);
  const checksumText=()=>includeFileHashes()&&hashesReady()?hashResult()?.manifest??'':'';
  const outgoingPlaintext=()=>body()+(checksumText()?`\n\n${checksumText()}`:'');
  createEffect(()=>{
    const key=hashKey();if(!includeFileHashes()){setHashResult(null);return;}
    const files=attachments().map(file=>({...file}));const template=downloadUrl();const account=app.accountId();
    setHashResult(null);if(!files.length){setHashResult({key,manifest:'',error:false});return;}
    if(!template||!account||!ownsDraft())return;
    const controller=new AbortController();onCleanup(()=>controller.abort());
    void attachmentHashManifest(files,template,account,controller.signal,maxUploadSize()).then(manifest=>{
      if(!controller.signal.aborted&&ownsDraft()&&hashKey()===key)setHashResult({key,manifest,error:false});
    },()=>{if(!controller.signal.aborted&&ownsDraft()&&hashKey()===key)setHashResult({key,manifest:'',error:true});});
  });

  /** The signature the send appends by itself: the sending identity's, unless
   *  the picker already put it in the body, and never on an encrypted or signed
   *  send — there the body is ciphertext or a clear-signed block, and anything
   *  appended after it would sit outside the encryption or break the signature. */
  const autoSignature = createMemo<ComposeSignature | null>(() => {
    const id=customFrom()?null:identity()??app.identities().find(item=>item.email.trim().toLowerCase()===app.me()?.username?.trim().toLowerCase())??null;
    if (id === null) return null;
    const sig=assignedSignature(id,replyContext());
    if (sig === null || insertedSignatures().has(sig.id)||insertedSignatures().has(id.id)) return null;
    const cs = cryptoState();
    if (cs !== null && (cs.encrypt || cs.sign)) return null;
    return sig;
  });
  const signatureScope=()=>JSON.stringify([identityId(),customFrom(),fromOverride(),replyContext(),autoSignature()]);
  const sendingSelection=()=>JSON.stringify([app.accountId(),composerAuthority()?.generation,draftId,identityId(),customFrom(),fromOverride(),replyContext()]);

  /** "Record when this message is opened" is offered only when the server has
   *  open tracking on and the message is neither encrypted nor signed (the
   *  image would sit outside the encryption or break the signature). */
  const trackingAvailable = createMemo(() => {
    if (!trackingAllowed() || !richMode() || editorApi() === null) return false;
    const cs = cryptoState();
    return !(cs !== null && (cs.encrypt || cs.sign));
  });

  /** Stored HTML signatures keep their native email layout when appended. The
   * HTML sanitizer remains the boundary, independently of the visual schema. */
  async function signatureBlock(sig: ComposeSignature): Promise<string> {
    if (sig.html !== null && sig.html !== '') {
        const {normalizeHtmlSource} = await import('./compose/htmlSource.ts');
        // Embedded images become inline images of this draft (t49), sent as `cid:` parts.
        const html = await embedSignatureImages(sig.html, uploadInline);
        return (await normalizeHtmlSource(html,ownedCids())).html;
    }
    return plainToHtml(sig.text);
  }

  async function signaturePlain(sig: ComposeSignature): Promise<string> {
    if(sig.plainText?.trim())return sig.plainText;
    if (sig.html) {
      try { return (await import('./compose/richtext.ts')).htmlToText(sig.html); } catch { /* Plain fallback remains inert. */ }
    }
    return sig.text;
  }

  /** Insert a chosen signature (W12). In rich mode it appends as HTML through the
   *  editor handle (keeping existing formatting); otherwise it appends its text
   *  to the plain body. */
  let pendingSignature:Promise<void>|undefined;
  function insertSignature(sig: ComposeSignature): void {
    // Text-only choices remain immediate. HTML work is awaited by Send so a
    // quick submission cannot omit the insertion or append it a second time.
    if(!sig.html){
      if(richMode()&&editorApi())editorApi()!.appendHtml(plainToHtml(sig.text));
      else setBody(cur=>cur.trim()?`${cur}\n\n-- \n${sig.text}`:sig.text);
      setInsertedSignatures(prev=>new Set(prev).add(sig.id).add(sig.identityId??sig.id));return;
    }
    pendingSignature=(pendingSignature??Promise.resolve()).catch(()=>undefined).then(async()=>{
      if(!ownsDraft())return;
      if(!richMode()){
        const plain=await signaturePlain(sig);if(!ownsDraft())return;
        setBody(cur=>cur.trim()?`${cur}\n\n-- \n${plain}`:plain);
      }else{
        const html=await signatureBlock(sig);if(!ownsDraft())return;
        if(sig.format==='html'||sourceMode()||editorApi()===null){
          if(sourceMode()&&editorApi()){
            editorApi()!.appendHtml(html);await editorApi()!.whenReady?.();
          }else{
            const {normalizeHtmlSource}=await import('./compose/htmlSource.ts');
            const combined=await normalizeHtmlSource(`${bodyHtml()}<br>${html}`,ownedCids());
            if(!ownsDraft())return;
            batch(()=>{setBodyHtml(combined.html);setBody(combined.text);changeFormat('html');});
          }
        }else editorApi()!.appendHtml(html);
      }
      if(!ownsDraft())return;
      setInsertedSignatures(prev=>new Set(prev).add(sig.id).add(sig.identityId??sig.id));
    });
    void pendingSignature.catch(()=>{if(ownsDraft())setError(t('mail-compose-html-failed'));});
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
    identityTouched = true;setAutoPick(null);setIdentityId(d.identityId ?? '');
    setFromOverride(d.fromOverride ?? '');
    setImportance(d.importance ?? 'normal');
    setCustomFrom(Boolean(d.fromOverride));
    setInsertedSignatures(new Set(d.insertedSignatures ?? []));
    setTo(d.to);
    setCc(d.cc ?? '');
    bccEdited=true;setBcc(d.bcc ?? '');
    setCcBccOpen((d.cc ?? '') !== '' || (d.bcc ?? '') !== '');
    setThreading({ inReplyTo: d.inReplyTo ?? [], references: d.references ?? [] });
    setDraftSource(d.source);
    setReplyContext(d.source?.keyword==='$answered');
    setSubject(d.subject);
    setRequestReceipt(d.requestReceipt === true);
    setRequestDelivery(d.requestDeliveryReceipt === true);
    trackingEdited=true;setTrackOpensValue(d.openTracking === true);
    const images = (d.inline ?? []).map((i) => ({ ...i, previewUrl: null }));
    setInlineImages(images);
    setAttachments(d.attachments ?? []);
    setIncludeFileHashes(d.includeFileHashes===true);setAttachPublicKey(d.attachPublicKey===true);setSentCopyFolder(d.sentMailboxId??'');
    const nextRich=d.bodyFormat!=='plain',nextNative=nextRich&&d.editorMode!=='rich';
    const changingEditor=nextRich!==richMode()||nextNative!==sourceMode();
    batch(()=>{if(changingEditor)setEditorApi(null);setSourceMode(nextNative);setRichMode(nextRich);setBody(d.bodyText);setBodyHtml(d.bodyHtml);});
    const api = editorApi();
    if (!changingEditor&&nextRich&&api!==null)api.setHtml(d.bodyHtml);
    setDraftsOpen(false);
    void loadPreviews(images);
    setLiveDraftVersion(value=>value+1);
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
        if (!ownsDraft() || app.accountId() !== acct) return [];
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
    setRemoteLoading(true);setRemoteListFailed(false);
    try{const result=await accountDraftApi(owner,remoteAbort?.signal).list();if(!ownsDraft())return;setRemoteReady(result.supported);setRemoteDrafts(result.items);if(!result.supported)setRemoteStatus('unsupported');}
    catch{if(ownsDraft()){setRemoteStatus('error');setRemoteListFailed(true);}}
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
  async function discardRemoteDraft(document:AccountDraftDocument):Promise<boolean>{
    const owner=composerAuthority();if(!owner?.storageAuthority||!ownsDraft()||remoteLoading())return false;
    if(remoteWriterId===document.id){remoteWriter?.suspend(true);await remoteWriter?.settled();}
    setRemoteLoading(true);
    try{await accountDraftApi(owner,remoteAbort?.signal).discard(document.id,document.revision);if(!ownsDraft())return false;deleteDraft(document.id,owner.id,owner.storageAuthority);setRemoteDrafts(rows=>rows.filter(row=>row.id!==document.id));void app.refreshFolders();
      if(draftId===document.id){allowRecoveryClose=true;pendingDraft=undefined;sent=true;props.onClose();}
      return true;
    }catch{remoteFailure('draftDiscardPending');return false;}finally{if(ownsDraft())setRemoteLoading(false);}
  }
  async function discardCurrentDraft():Promise<void>{
    if(!ownsDraft()||busy()||uploading()||discardCurrentBusy())return;
    const owner=composerAuthority();
    const id=draftId;
    const writer=remoteWriter;
    const current=()=>ownsDraft()&&draftId===id&&remoteWriter===writer;
    setDiscardCurrentBusy(true);writer?.suspend(true);
    try{
      await writer?.settled();
      if(!current())return;
      if(remoteStatus()==='uncertain'||remoteStatus()==='conflict'){remoteFailure('draftConflict');return;}
      // Include our own completed save, but never adopt a revision fetched from
      // another editor. The DELETE retains the same server-side CAS boundary.
      const expectedRevision=remoteRevision();
      if(expectedRevision){
        if(!owner?.storageAuthority)return;
        const api=accountDraftApi(owner,remoteAbort?.signal);
        const document=await api.load(id);
        if(!current())return;
        if(remoteRevision()!==expectedRevision||document.id!==id||document.revision!==expectedRevision){
          remoteFailure('draftConflict');return;
        }
        await api.discard(id,expectedRevision);
        if(!current()||remoteRevision()!==expectedRevision)return;
        deleteDraft(id,owner.id,owner.storageAuthority);
        setRemoteDrafts(rows=>rows.filter(row=>row.id!==id));
        void app.refreshFolders();
      }else{
        deleteDraft(id,draftAccount,draftAuthority?.storageAuthority);
      }
      clearTimeout(saveTimer);pendingDraft=undefined;
      sent=true;allowRecoveryClose=true;props.onClose();
    }catch{if(current())remoteFailure('draftDiscardPending');}
    finally{
      setDiscardCurrentBusy(false);setDiscardCurrentOpen(false);
      if(!sent&&current())writer?.suspend(protectedDraft());
    }
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
  /** Why closing now would lose the message (it is never stored), or null when a draft keeps it. */
  function unsavedCloseReason():string|null{
    const content=[to(),cc(),bcc(),subject(),body()].some(value=>value.trim()!=='')||attachments().length>0||inlineImages().length>0;
    if(!content)return null;
    if(protectedDraft())return 'mail-draft-close-protected';
    if(!draftAuthority?.storageAuthority)return 'mail-draft-close-unstored';
    return null;
  }
  function confirmUnsavedClose(reason:string):Promise<boolean>{
    // The composer may be in a detached window: bring that window forward, the dialog opens there.
    const view=backdropEl?.ownerDocument.defaultView;if(view&&view!==window)view.focus();
    return new Promise(resolve=>setClosePrompt({body:t(reason),answer:leave=>{setClosePrompt(null);resolve(leave);}}));
  }
  onCleanup(()=>closePrompt()?.answer(false));
  async function beforeDraftClose(discard=false):Promise<boolean>{
    if(sent||allowRecoveryClose||!ownsDraft())return true;
    if(uploading()||remoteWriter?.isBusy())return false;
    if(closePrompt())return false;
    const reason=unsavedCloseReason();
    if(reason)return discard||confirmUnsavedClose(reason);
    if(settings()['mail.draftSaveMode']?.value!=='manual'&&remoteWriter?.hasPending()&&!await remoteWriter.flush())return false;
    return true;
  }
  async function closeWithDraftSave():Promise<void>{if(await beforeDraftClose())props.onClose();}
  onMount(()=>props.registerBeforeClose?.(beforeDraftClose,()=>!sent&&!allowRecoveryClose&&ownsDraft()&&unsavedCloseReason()!==null));
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
  createEffect(()=>{
    liveDraftVersion();const source=draftSource(),owner=composerAuthority(),account=app.accountId();
    if(sent||!owner||owner.id!==account||!ownsDraft())return;
    const release=registerLiveReplyDraft(owner,draftId,source);releaseLiveReply=release;onCleanup(release);
  });
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
      ...(draftSource()?{source:draftSource()!}:{}),
      ...(remoteRevision()?{serverRevision:remoteRevision()!}:{}),
      ...(remoteStatus()==='uncertain'?{serverStatus:'uncertain' as const}:{}),
      identityId: identityId(),
      importance:importance(),
      includeFileHashes:includeFileHashes(),attachPublicKey:attachPublicKey(),...(effectiveSentCopy()?{sentMailboxId:effectiveSentCopy()}:{}),
      ...(customFrom() ? {fromOverride: fromOverride()} : {}),
      bodyFormat: richMode() ? 'html' : 'plain',
      ...(richMode()?{editorMode:sourceMode()?'html' as const:'rich' as const}:{}),
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
  function onRecipientInput(field: RecipientField, value: string, caret=value.length): void {
    if(field==='bcc'&&value!==bcc())bccEdited=true;
    recipientFields[field][1](value);
    recipientCarets[field]=caret;setAcIndex(-1);
    if(recipientComposing())return;
    const token = recipientToken(value,caret).query;
    contactAc.setQuery(token);
    setGalToken(field==='to'?token:'');
    setAcField(token.length > 0 ? field : null);
  }

  /** Replace the in-progress recipient token of `field` with a resolved
   *  address (`, `-joined). */
  function insertRecipient(field: RecipientField, address: string): void {
    if(field==='bcc')bccEdited=true;
    const [value, setValue] = recipientFields[field];
    const replacement=replaceRecipientToken(value(),recipientCarets[field]??value().length,address);
    if(separator()==='semicolon'&&replacement.value.endsWith(', ')&&replacement.caret===replacement.value.length)replacement.value=`${replacement.value.slice(0,-2)}; `;
    setValue(replacement.value);
    recipientCarets[field]=replacement.caret;
    contactAc.reset();
    if (field === 'to') setGalToken('');
    setAcField(null);
    setAcIndex(-1);
    const input=recipientInputs[field];input?.focus();input?.setSelectionRange(replacement.caret,replacement.caret);
  }

  const recipientListId=(field:RecipientField)=>`${composeId}-${field}-suggestions`;
  const recipientListOpen=(field:RecipientField)=>!recipientComposing()&&acField()===field&&contactAc.suggestions().length>0;
  function dismissRecipients():void {setAcField(null);setAcIndex(-1);contactAc.reset();setGalToken('');}
  const caretOf=(field:RecipientField,input:HTMLInputElement):number=>autoChip()?fieldValue(field).length:(input.selectionStart??input.value.length);
  const recipientAttrs=(field:RecipientField):JSX.InputHTMLAttributes<HTMLInputElement>=>({
    role:'combobox','aria-autocomplete':'list','aria-haspopup':'listbox',
    'aria-label':t(field==='to'?'mail-compose-to':field==='cc'?'mail-compose-cc':'mail-compose-bcc'),
    get 'aria-expanded'(){return recipientListOpen(field);},
    get 'aria-controls'(){return recipientListOpen(field)?recipientListId(field):undefined;},
    get 'aria-activedescendant'(){return recipientListOpen(field)&&acIndex()>=0&&acIndex()<contactAc.suggestions().length?`${recipientListId(field)}-${acIndex()}`:undefined;},
  });
  const contactName=(email:string):string|null=>{
    const lower=email.toLowerCase();
    for(const card of app.contacts())if(card.name.full&&card.emails.some(entry=>entry.value.toLowerCase()===lower))return card.name.full;
    return null;
  };
  // One contact card for the three fields: a chip click opens it, as an address does in the reader.
  const addressCard=createAddressContactCard({scope:()=>JSON.stringify([app.accountId(),'compose']),active:()=>true});
  /** Props of the token field for To, Cc or Bcc. The contact suggestions are its children. */
  const recipientProps=(field:RecipientField)=>({
    field,
    get groupLabel(){return t('mail-recipient-chips',{field:t(field==='to'?'mail-compose-to':field==='cc'?'mail-compose-cc':'mail-compose-bcc')});},
    get value(){return fieldValue(field);},
    onChange:(value:string,caret:number)=>onRecipientInput(field,value,caret),
    get separator(){return separator();},
    get autoChip(){return autoChip();},
    nameOf:contactName,
    inputAttrs:recipientAttrs(field),
    inputRef:(input:HTMLInputElement)=>{recipientInputs[field]=input;if(field==='to')toInputEl=input;},
    isComposing:recipientComposing,
    openCard:addressCard.open,
    closeCard:addressCard.close,
    onInputFocus:()=>setFocusedRecipient(field),
    onInputClick:(event:MouseEvent&{currentTarget:HTMLInputElement})=>onRecipientInput(field,fieldValue(field),caretOf(field,event.currentTarget)),
    onCompositionStart:()=>{setRecipientComposing(true);dismissRecipients();},
    onCompositionEnd:(event:CompositionEvent&{currentTarget:HTMLInputElement})=>{setRecipientComposing(false);onRecipientInput(field,fieldValue(field),caretOf(field,event.currentTarget));},
    onInputKeyUp:(event:KeyboardEvent&{currentTarget:HTMLInputElement})=>{if(['ArrowLeft','ArrowRight','Home','End'].includes(event.key)&&!event.isComposing&&!recipientComposing())onRecipientInput(field,fieldValue(field),caretOf(field,event.currentTarget));},
    onInputBlur:(event:FocusEvent&{currentTarget:HTMLInputElement})=>{
      const next=event.relatedTarget as Node|null;
      if(next?.nodeType&&event.currentTarget.ownerDocument.getElementById(recipientListId(field))?.contains(next))return;
      // Keep the separate GAL picker mounted while focus moves into its buttons.
      setAcField(null);setAcIndex(-1);contactAc.reset();
      setFocusedRecipient(current=>current===field?null:current);
    },
    onInputKeyDown:(event:KeyboardEvent&{currentTarget:HTMLInputElement})=>{
      if(event.isComposing||recipientComposing()||event.keyCode===229)return;
      if(event.key==='Escape'&&(acField()===field||galToken())){event.preventDefault();event.stopPropagation();dismissRecipients();return;}
      if(event.key==='Enter'){
        const input=event.currentTarget;
        const caret=caretOf(field,input);
        recipientCarets[field]=caret;
        const token=recipientToken(fieldValue(field),caret);
        const typed=fieldValue(field).slice(token.start,token.end).trim();
        const addresses=parseRecipients(typed);
        let handled=false;
        // An explicitly highlighted suggestion wins; otherwise a complete
        // typed mailbox must not silently become a different contact match.
        if(acIndex()<0&&addresses.length===1&&isMailbox(addresses[0]!.email)){insertRecipient(field,typed);handled=true;}
        else if(recipientListOpen(field)){
          const pick=contactAc.suggestions()[Math.max(0,acIndex())];
          if(pick){insertRecipient(field,pick.display);handled=true;}
        }
        // Nothing to pick: the token field finishes the typed text itself.
        if(handled||!autoChip()){event.preventDefault();event.stopPropagation();}
        return;
      }
      if(event.key==='ArrowDown'||event.key==='ArrowUp'){
        const caret=caretOf(field,event.currentTarget);
        if(!recipientListOpen(field)||recipientCarets[field]!==caret)onRecipientInput(field,fieldValue(field),caret);
        const count=contactAc.suggestions().length;if(!recipientListOpen(field)||!count)return;
        event.preventDefault();event.stopPropagation();
        const next=event.key==='ArrowDown'?(acIndex()+1)%count:(acIndex()<0?count-1:(acIndex()+count-1)%count);setAcIndex(next);
        event.currentTarget.ownerDocument.getElementById(`${recipientListId(field)}-${next}`)?.scrollIntoView?.({block:'nearest'});
      }
    },
  });

  /** The contact suggestions under `field`, while it is the one being typed in. */
  const suggestionList = (field: RecipientField): JSX.Element => (
    <Show when={recipientListOpen(field)}>
      <ul id={recipientListId(field)} class="compose__ac" role="listbox" aria-label={t('mail-compose-contact-suggestions')}>
        <For each={contactAc.suggestions()}>
          {(s: ContactSuggestion,index) => (
            <li role="presentation">
              <button
                type="button"
                role="option"
                id={`${recipientListId(field)}-${index()}`}
                tabindex={-1}
                aria-selected={acIndex()===index()}
                class="compose__ac-item"
                data-testid="contact-suggestion"
                onPointerDown={e=>e.preventDefault()}
                onMouseDown={e=>e.preventDefault()}
                onPointerMove={()=>setAcIndex(index())}
                onClick={()=>insertRecipient(field,s.display)}
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
        if (file.size > max && app.nextcloud.enabled()) {
          setOversized((cur) => (cur.includes(file) ? cur : [...cur, file]));
          continue;
        }
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
          if (!ownsDraft() || app.accountId() !== acct) return;
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

  /** Put a created share link at the end of the body (t39 L7): name and size,
   *  the link, and its expiry and password state, as text or as a paragraph. */
  function insertShareLink(file: File, link: ShareLink): void {
    setOversized((cur) => cur.filter((f) => f !== file));
    const head = rawT('nextcloud-share-body', { name: file.name, size: `${megabytes(file.size)} MB`, url: link.url });
    const notes = [
      ...(link.expiresAt ? [rawT('nextcloud-share-body-expires', { date: link.expiresAt })] : []),
      ...(link.passwordProtected ? [rawT('nextcloud-share-body-password')] : []),
    ].join(' ');
    const api = editorApi();
    if (richMode() && api !== null) {
      const label = escapeHtml(head.slice(0, head.length - link.url.length));
      const href = escapeHtml(link.url).replace(/"/g, '&quot;');
      api.appendHtml(`<p>${label}<a href="${href}">${escapeHtml(link.url)}</a>${notes ? `<br>${escapeHtml(notes)}` : ''}</p>`);
    } else {
      const text = notes ? `${head}\n${notes}` : head;
      setBody((cur) => (cur.trim() !== '' ? `${cur}\n\n${text}` : text));
    }
    toast.success(t('nextcloud-share-inserted', { name: file.name }));
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    if(!mailFeature('allowSend')||busy()||uploading()||discardCurrentOpen()||discardCurrentBusy()||checkPrompt()!==null)return;
    setBusy(true);
    try {
    const expectedSelection=sendingSelection();let identityTimer:ReturnType<typeof setTimeout>|undefined;
    try{await Promise.race([loadInitialIdentities(),new Promise<never>((_,reject)=>{identityTimer=setTimeout(()=>reject(new Error('Identity loading timed out')),15_000);})]);}
    catch{if(ownsDraft())setError(t('mail-compose-identities-failed'));return;}
    finally{clearTimeout(identityTimer);}
    if(!ownsDraft())return;
    if(!await runSendChecks()||!ownsDraft())return;
    if(sendingSelection()!==expectedSelection){setError(t('mail-compose-send-failed'));return;}
    if(pendingSignature)await pendingSignature;
    if(sourceMode()&&editorApi()===null){setError(t('mail-compose-html-processing'));return;}
    const readyEditor=editorApi();if(readyEditor?.whenReady)await readyEditor.whenReady();
    remoteWriter?.suspend(true);
    if(remoteWriter?.isBusy())await remoteWriter.settled();
    if(!ownsDraft())return;
    setError(null);
    if(customFrom()&&!isMailbox(fromOverride().trim())){setError(t('mail-compose-from-invalid'));return;}
    if(externalMode()==='confirm' && outside().length && reviewedRecipients()!==recipientReviewKey()){
      setError(t('mail-external-confirm-required'));return;
    }
    const cs = cryptoState();
    const expectedSignatureScope=signatureScope();
    if(!hashesReady()){setError(t('mail-compose-hashes-pending'));return;}
    if(cs?.encrypt&&(!cs.encryptedDraft||cs.draftSource!==cryptoDraftSource(outgoingPlaintext(),parseRecipients([to(),cc(),bcc()].join(',')).map(r=>r.email),subject(),cs.protectSubject,cs.sign?signingSession()?.keyRef??null:null))){setError(t('mail-compose-crypto-pending'));return;}
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
    // Defence in depth (t87 D11): never sign with a key of another mailbox, and
    // never send a ciphertext signed by a key other than the current session's.
    if (cs !== null && cs.sign) {
      const held = signingSession();
      const signedBy = cs.encrypt && cs.encryptedDraft !== null ? cs.draftSignedWith : held?.keyRef ?? null;
      let owned = held !== null && signedBy === held.keyRef;
      if (owned && held !== null) {
        try { assertSigningKeyOwner(held, app.accountId()); } catch { owned = false; }
      }
      if (!owned) {
        dropSigningSession();
        setUnlockOpen(true);
        setError(t('mail-compose-sign-unlock-required'));
        return;
      }
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
        htmlBody = await clearSignBody(getCryptoWorker(), session, outgoingPlaintext());
        textBody = htmlBody;
      } else {
        // The rich editor's serialized HTML, but only while the editor is
        // actually mounted. In plain-text mode, and when the editor's chunk is
        // still loading or failed to load, the user typed into the textarea and
        // `bodyHtml` does not hold their text — the body is built from `body`.
        const rich = richMode() && editorApi() !== null;
        htmlBody = rich ? bodyHtml() : '';
        if (!rich) textBody = outgoingPlaintext();
        else if(checksumText())htmlBody+=`<pre>${escapeHtml(checksumText())}</pre>`;
        const sig = autoSignature();
        if (sig !== null) {
          if (rich) htmlBody += `<br>${await signatureBlock(sig)}`;
          else textBody += `\n\n-- \n${await signaturePlain(sig)}`;
        }
        if (rich) htmlBody = htmlMessageDocument(htmlBody);
      }
      const subjectToSend =
        enc !== null && cs !== null && cs.protectSubject && enc.encryptedSubjectApplied
          ? rawT('mail-compose-encrypted-subject')
          : subject();
      const attached = [...attachments()];
      if(attachPublicKey()){
        if(app.ownKeys().length===0)await app.loadKeys();
        if(!ownsDraft())throw new Error(t('mail-compose-account-changed'));
        const file=publicIdentityFile(identity()?.pgpKeyId?app.ownKeys().filter(key=>key.id===identity()?.pgpKeyId):app.ownKeys(),customFrom()?fromOverride():identity()?.email??app.me()?.username??'');
        const endpoint=uploadUrl();const account=app.accountId();
        if(!file||!endpoint||!account)throw new Error(t('mail-compose-public-key-unavailable'));
        const uploaded=await uploadBlob(endpoint,account,file);
        if(!ownsDraft())throw new Error(t('mail-compose-account-changed'));
        attached.push({blobId:uploaded.blobId,name:file.name,size:uploaded.size,contentType:uploaded.type});
      }
      // Only the images the body still shows go out (none for ciphertext or a
      // clear-signed body, which carry no `cid:` reference).
      const inline = inlineAttachments(inlineImages(), referencedCids(htmlBody));
      if(requestDelivery()&&!deliveryAvailable())throw new Error(t('mail-compose-delivery-unavailable'));
      if (!ownsDraft()) throw new Error(t('mail-compose-account-changed'));
      if(sendingSelection()!==expectedSelection||signatureScope()!==expectedSignatureScope)throw new Error(t('mail-compose-send-failed'));
      await app.sendMessage({
        ...(remoteRevision()&&remoteStatus()!=='uncertain'&&remoteStatus()!=='conflict'?{accountDraft:{id:draftId,revision:remoteRevision()!}}:{}),
        to: to(),
        cc: cc(),
        bcc: bcc(),
        inReplyTo: threading().inReplyTo,
        references: threading().references,
        ...(draftSource() !== undefined ? { source: draftSource()! } : {}),
        ...(initial?.sources !== undefined ? { sources: initial.sources } : {}),
        subject: subjectToSend,
        htmlBody,
        ...(textBody !== undefined ? {textBody} : {}),
        identity: identity(),
        ...(effectiveSentCopy()?{sentMailboxId:effectiveSentCopy()}:{}),
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
        ...(receiptAvailable() && featureOn('compose.read_receipts') && requestReceipt() ? { requestReadReceipt: true } : {}),
        ...(requestDelivery() ? { requestDeliveryReceipt: true } : {}),
        ...(trackingAvailable() && trackOpens() ? { openTracking: true } : {}),
      });
      if (!ownsDraft()) return;
      // W9: the composition was sent — drop its auto-saved draft.
      sent=true;releaseLiveReply();clearTimeout(saveTimer);pendingDraft=undefined;
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
          <button type="submit" class={`btn btn--primary ${a11y.focusable}`} disabled={!mailFeature('allowSend')||busy()||discardCurrentOpen()||discardCurrentBusy()} title={mailFeature('allowSend')?undefined:mailFeatureOffTitle()}>
            <Icon name="sent"/> {busy() ? t('mail-compose-sending') : sendAt() !== '' ? t('mail-compose-schedule') : t('mail-compose-send')}
          </button>
          <Show when={featureOn('compose.scheduled_send')&&mailFeature('allowSend')}><SendLaterMenu disabled={busy()} onPick={value=>{setSendAt(value);queueMicrotask(()=>backdropEl?.querySelector<HTMLFormElement>('form.compose')?.requestSubmit());}}/></Show>
          <h2 class={a11y.srOnly}>{t(initial !== undefined ? TITLE_KEY[initial.mode] : 'mail-compose-title')}</h2>
          <div class="compose__header-actions">
            <button type="button" class="btn btn--ghost compose-action--secondary" data-detached-local="" aria-haspopup="dialog" aria-label={t('mail-draft-discard-account')} disabled={busy()||uploading()||discardCurrentBusy()||remoteStatus()==='uncertain'||remoteStatus()==='conflict'} onClick={()=>setDiscardCurrentOpen(true)} data-tooltip={t('mail-draft-discard-confirm')}><Icon name="delete"/><span>{t('mail-draft-discard-account')}</span></button>
        <Show when={remoteReady()&&!protectedDraft()}><button type="button" class="btn btn--ghost" disabled={remoteStatus()==='saving'||remoteStatus()==='uncertain'||remoteStatus()==='conflict'} onClick={()=>void remoteWriter?.flush()}><Icon name="file"/>{t('mail-draft-save-now')}</button></Show>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="open-drafts"
              aria-haspopup="dialog"
              aria-expanded={draftsOpen()}
              onClick={() => (draftsOpen() ? setDraftsOpen(false) : openDrafts())}
            >
              <Icon name="file"/> {t('mail-compose-drafts')}
            </button>
            <button
              type="button"
              class={`btn btn--ghost ${a11y.focusable}`}
              data-testid="open-recall"
              aria-label={t('mail-compose-recall')}
              data-tooltip={t('mail-compose-recall-help')}
              aria-expanded={recallOpen()}
              onClick={() => (recallOpen() ? setRecallOpen(false) : openRecall())}
            >
              <Icon name="clock"/> <span>{t('mail-compose-recall')}</span>
            </button>
            <Show when={!props.embedded}><button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t('mail-compose-close')} onClick={() => void closeWithDraftSave()}>
              <Icon name="close" />
            </button></Show>
          </div>
        </header>

        <p role="status" aria-live="polite" class="compose__sign-unlock-note" data-testid="account-draft-status">{t(protectedDraft()?'mail-draft-protected':`mail-draft-status-${remoteStatus()}`)}</p>
        <Show when={['error','uncertain','conflict'].includes(remoteStatus())&&localDraftAuthority()}><button type="button" class="btn btn--ghost" onClick={()=>{allowRecoveryClose=true;props.onClose();}}>{t('mail-draft-close-recovery')}</button></Show>

        {/* Discard confirmations and the unsaved-close prompt share one themed modal,
            mounted in the composer's own document (a detached window, when torn off). */}
        <ConfirmDialog open={checkPrompt()!==null} testId="compose-send-checks" mount={dialogMount()} danger={false}
          title={t('mail-check-title')} confirmLabel={t('mail-check-send-anyway')} cancelLabel={t('mail-check-review')}
          onCancel={()=>answerChecks('review')} onConfirm={()=>answerChecks('send')}>
          <p>{t('mail-check-intro')}</p>
          <ul class="compose-check-list"><For each={checkPrompt()?.asked??[]}>{issue=><li>{issueText(issue)}</li>}</For></ul>
          <Show when={(checkPrompt()?.rememberable.length??0)>0}><label class="compose-check-remember"><input type="checkbox" checked={rememberChecked()} onChange={event=>setRememberChecked(event.currentTarget.checked)}/>{t('mail-check-remember')}</label></Show>
        </ConfirmDialog>
        <ConfirmDialog open={discardCurrentOpen()} testId="compose-discard-confirm" mount={dialogMount()}
          title={t('mail-draft-discard-title')} body={t('mail-draft-discard-confirm-detail',{subject:subject()||t('mail-no-subject')})}
          confirmLabel={t('mail-draft-discard-action')} cancelLabel={t('mail-draft-keep-editing')}
          onCancel={()=>{if(!discardCurrentBusy())setDiscardCurrentOpen(false);}} onConfirm={discardCurrentDraft}/>
        <ConfirmDialog open={discardConfirmation()!==null} testId="compose-discard-account-confirm" mount={dialogMount()}
          title={t('mail-draft-discard-title')} body={t('mail-draft-discard-confirm-detail',{subject:discardConfirmation()?.summary.subject||t('mail-no-subject')})}
          confirmLabel={t('mail-draft-discard-action')} cancelLabel={t('mail-draft-keep-editing')}
          onCancel={()=>setDiscardConfirmation(null)} onConfirm={async()=>{const row=discardConfirmation();if(row)await discardRemoteDraft(row);setDiscardConfirmation(null);}}/>
        <ConfirmDialog open={closePrompt()!==null} testId="compose-close-confirm" mount={dialogMount()}
          title={t('mail-draft-close-title')} body={closePrompt()?.body??''}
          confirmLabel={t('mail-draft-discard-action')} cancelLabel={t('mail-draft-keep-editing')}
          onCancel={()=>closePrompt()?.answer(false)} onConfirm={()=>closePrompt()?.answer(true)}/>
        <DraftsPopup
          open={draftsOpen()}
          accountDrafts={remoteDrafts}
          localDrafts={drafts}
          accountSupported={remoteReady()}
          loading={remoteLoading()}
          loadFailed={remoteListFailed()}
          currentId={draftId}
          mount={dialogMount()}
          onRetryLoad={()=>void refreshRemoteDrafts()}
          onOpenAccount={id=>void resumeRemoteDraft(id)}
          onOpenLocal={resumeDraft}
          onDiscardAccount={discardRemoteDraft}
          onDiscardLocal={discardDraft}
          onClose={() => setDraftsOpen(false)}
        />
        <Show when={recallOpen()}>
          <RecallPanel submissions={app.cancelableOutbox} onRecall={recallSubmission} loading={app.outboxLoading() || recallBusy()} failed={recallError()} />
        </Show>

        <Show when={!localDraftAuthority()}><p role="status" class="compose__sign-unlock-note">{t('mail-compose-private-storage-unavailable')}</p></Show>
        <Show when={compact()}><section class="compose__quick-summary" aria-label={t('mail-quick-reply')}><p><strong>{t('mail-compose-to')}:</strong> {to()}<Show when={cc()}>{' · '}{t('mail-compose-cc')}: {cc()}</Show><Show when={bcc()}>{' · '}{t('mail-compose-bcc')}: {bcc()}</Show></p><p>{subject()}</p><button type="button" class="btn btn--ghost" onClick={() => setExpanded(true)}>{t('mail-quick-reply-expand')}</button></section></Show>
        <Show when={liveInvalid().length>0||subjectAlert()}>
          <ul class="compose-alerts" role="status" aria-label={t('mail-alerts-label')} data-testid="compose-alerts">
            <For each={liveInvalid()}>{bad=><li><button type="button" class="compose-alerts__item" data-alert="invalid" onClick={()=>focusIssue({id:'invalidRecipients',details:[bad]})}><Icon name="alert"/>{t('mail-alert-invalid',{address:isolate(bad)})}</button></li>}</For>
            <Show when={subjectAlert()}><li><button type="button" class="compose-alerts__item" data-alert="subject" onClick={()=>subjectInput?.focus()}><Icon name="alert"/>{t('mail-alert-subject')}</button></li></Show>
          </ul>
        </Show>
        <Show when={!compact()}>
          <Show when={showFrom()}><label class="field compose__from">
            <span>{t('mail-compose-from')}</span>
            <select value={customFrom() ? "__custom_from__" : identityId()} onChange={(e) => { const value=e.currentTarget.value; setCustomFrom(value === "__custom_from__"); if(value !== "__custom_from__") pickIdentity(value); }}>
              <option value="">{isolate(app.me()?.username ?? t('mail-compose-from-default'))}</option>
              <option value="__custom_from__">{t('mail-compose-from-custom')}</option>
              <For each={app.identities()}>
                {(id) => (
                  <option value={id.id}>
                    {isolate(id.name)} &lt;{id.email}&gt;{id.label ? ` (${isolate(id.label)})` : ''}
                  </option>
                )}
              </For>
            </select>
          </label>
          </Show>
          <Show when={autoPick()}>{(choice) => <p class="compose__from-hint" role="status" data-testid="compose-from-hint">
            {t(`mail-compose-from-why-${choice().reason}`, { address: isolate(choice().address ?? ''), identity: isolate(identity()?.email ?? '') })}{' '}
            <button type="button" class="btn btn--ghost" onClick={() => setFromVisibleOverride(true)}>{t('mail-compose-from-change')}</button>
            <Show when={choice().alternatives.length > 0 && choice().reason === 'ambiguous'}>
              {' '}<For each={choice().alternatives}>{(alt) => <button type="button" class="btn btn--ghost" onClick={() => pickIdentity(alt.id)}>{isolate(alt.email)}</button>}</For>
            </Show>
          </p>}</Show>
          <Show when={customFrom()}><label class="field compose__from"><span>{t('mail-compose-from-alias')}</span><input type="text" inputMode="email" required maxlength={320} autocomplete="off" autocapitalize="off" spellcheck={false} value={fromOverride()} onInput={event=>setFromOverride(event.currentTarget.value)} onKeyDown={event=>{
            if(event.key!=='Enter'||event.isComposing||event.keyCode===229)return;
            event.preventDefault();event.stopPropagation();
            const address=event.currentTarget.value.trim();
            if(!isMailbox(address)){setError(t('mail-compose-from-invalid'));return;}
            setFromOverride(address);setError(previous=>previous===t('mail-compose-from-invalid')?null:previous);toInputEl?.focus();
          }} aria-describedby={fromHelpId}/><small id={fromHelpId}>{t('mail-compose-from-alias-help')}</small></label></Show>

        <div class="field compose__to">
          <span>{t('mail-compose-to')}</span>
          <RecipientTokenField {...recipientProps('to')} placeholder={t('mail-compose-to-placeholder')}>{suggestionList('to')}</RecipientTokenField>
        </div>
        {addressCard.card()}

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
          <div class="field compose__to">
            <span>{t('mail-compose-cc')}</span>
            <RecipientTokenField {...recipientProps('cc')}>{suggestionList('cc')}</RecipientTokenField>
          </div>
          <div class="field compose__to">
            <span>{t('mail-compose-bcc')}</span>
            <RecipientTokenField {...recipientProps('bcc')}>{suggestionList('bcc')}</RecipientTokenField>
          </div>
        </Show>

        <label class="field compose__subject">
          <span>{t('mail-compose-subject')}</span>
          <input type="text" ref={subjectInput} value={subject()} onInput={(e) => setSubject(e.currentTarget.value)} />
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

        {/* A file over the attachment limit (t39 L7): upload it to the linked
            Nextcloud and insert a share link instead. */}
        <For each={featureOn('compose.share_links') ? oversized() : []}>
          {(file) => (
            <div class="compose__share-link" data-testid="compose-share-link">
              <ShareLinkComposer
                file={file}
                service={app.nextcloud.service}
                onCreated={(link) => insertShareLink(file, link)}
                onDismiss={() => setOversized((cur) => cur.filter((f) => f !== file))}
              />
            </div>
          )}
        </For>
        <Show when={externalMode()!=='off' && outside().length>0 && (externalMode()==='confirm' || !externalHidden())}><section class="compose-external" role="status" aria-label={t('mail-external-recipients')}><details><summary><Icon name="alert"/> {t('mail-external-line',{count:outside().length,domains:isolate(externalDomains(outside().filter(isMailbox)).join(', '))})}</summary><p>{t('mail-external-help')}</p><ul><For each={outside()}>{address=><li>{address}</li>}</For></ul></details><Show when={externalMode()!=='confirm'}><button type="button" class="btn btn--ghost compose-external__dismiss" onClick={()=>setDismissedExternal(outside())}>{t('mail-external-dismiss')}</button></Show><Show when={externalMode()==='confirm'}><label><input type="checkbox" checked={reviewedRecipients()===recipientReviewKey()} onChange={event=>setReviewedRecipients(event.currentTarget.checked?recipientReviewKey():'')}/>{t('mail-external-reviewed')}</label></Show></section></Show>
        {/* The composer ribbon (t38): pinned, or auto-hide with only the history
            and tab names showing. It pushes the body down when it opens (never
            covers it) and has a fixed height, so the body (flex:1, t37) takes
            exactly what is left. Every group is three rows; a tab too narrow
            for its groups folds the lowest-priority ones into dropdowns. Hidden
            tabs stay mounted, so the security evaluation is always live. */}
        <EditorRibbon
          state={ribbon}
          visible={()=>richMode()?['format','insert','table','security','delivery']:['format','insert','security','delivery']}
          history={editorApi}
          showHistory={()=>richMode()}
          tabs={[
            {id:'format',after:()=><>
              <RibbonGroup label={t('mail-compose-ribbon-recipients')} priority={70} icon="users" testId="compose-group-recipients">
                <RibbonItem><button type="button" class="btn btn--ghost" onClick={checkNames} data-tooltip={t('mail-compose-check-names-help')}><Icon name="check"/><span class="compose-rib__text">{t('mail-compose-check-names')}</span></button></RibbonItem>
                <RibbonItem><button type="button" class="btn btn--ghost" data-testid="compose-show-cc-bcc" aria-pressed={ccBccOpen()} onClick={()=>setCcBccOpen(!ccBccOpen())}><Icon name="users"/><span class="compose-rib__text">{t('mail-compose-show-cc-bcc')}</span></button></RibbonItem>
                <RibbonItem><button type="button" class="btn btn--ghost" aria-pressed={showFrom()} onClick={()=>setFromVisibleOverride(!showFrom())}><Icon name="mail"/><span class="compose-rib__text">{t('mail-compose-from')}</span></button></RibbonItem>
              </RibbonGroup>
              <Show when={featureOn('compose.rich_text')}><BodyModeGroup label={t('mail-compose-ribbon-message-type')} value={()=>!richMode()?'plain':sourceMode()?'html':'rich'} onChange={changeFormat}/></Show>
            </>},
            {id:'insert',before:()=><>
              <RibbonGroup label={t('response-templates-title')} priority={65} icon="book" testId="compose-response-templates"><RibbonItem><ResponseTemplates disabled={busy()} onInsert={insertResponseTemplate}/></RibbonItem></RibbonGroup>
              {/* New-file attach (26.15 §1): pick a local file, upload its bytes to the
                  account's JMAP upload endpoint, and fold the returned blob into the
                  shared attachment list. Always available (core compose); the input is
                  disabled until the session's upload contract has loaded. The file
                  input is visually hidden behind the styled label. */}
              <Show when={featureOn('compose.attachments')}><RibbonGroup label={t('mail-compose-attachments')} priority={55} icon="attachment" testId="compose-attach">
                <RibbonItem size="large">
                  <label class={`btn btn--ghost compose-rib__large ${a11y.focusable}`}>
                    <Icon name="attachment"/><span class="compose-rib__text">{uploading() ? t('mail-compose-uploading') : t('mail-compose-attach-file')}</span>
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
                </RibbonItem>
                <RibbonItem><InsertAccountItem onFiles={onFilesPicked} disabled={uploading()||busy()}/></RibbonItem>
                {/* V7 Nextcloud attach (plan §18.4): only when a Nextcloud account is
                    linked; the picker opens under its button. */}
                <Show when={app.nextcloud.enabled()}>
                  <RibbonItem>
                    <RibbonPopover testId="compose-nextcloud" label={t('mail-compose-attach-nextcloud')} icon="nextcloud" buttonClass={`btn btn--ghost ${a11y.focusable}`} open={ncOpen()} onOpenChange={setNcOpen}>
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
                    </RibbonPopover>
                  </RibbonItem>
                </Show>
                <Show when={attachError()}>
                  {(message) => <RibbonItem><p class="login__error compose-rib__error" role="alert" data-tooltip={message()}>{message()}</p></RibbonItem>}
                </Show>
              </RibbonGroup></Show>
            </>,after:()=><>
              <Show when={signatures().length > 0}>
                <RibbonGroup label={t('mail-compose-signature')} priority={40} icon="edit" testId="compose-group-signature">
                  <RibbonItem><SignaturePicker signatures={signatures} onInsert={insertSignature} /></RibbonItem>
                </RibbonGroup>
              </Show>
              {/* V7 inline Assist composer tools + dictation (plan §14.3). Each component
                  self-hides on the capabilities it lacks; the whole group is gated on
                  the gateway being enabled, so a Disabled Assist gateway renders
                  NOTHING here. Nothing is auto-applied or sent. The tools and their
                  suggestion open under one button, so they never add a row. */}
              <Show when={app.assist.enabled()}>
                <RibbonGroup label={t('mail-compose-group-assist')} priority={35} icon="assistant" testId="compose-group-assist">
                  <RibbonItem size="large">
                    <RibbonPopover testId="compose-assist" label={t('mail-compose-group-assist')} icon="assistant" buttonClass={`btn btn--ghost compose-rib__large ${a11y.focusable}`}>
                      <div class="compose__assist">
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
                    </RibbonPopover>
                  </RibbonItem>
                </RibbonGroup>
              </Show>
            </>},
            {id:'table'},
            {id:'security',before:()=><>
              {/* Crypto + DLP (plan §2.5): encrypt/sign toggles, the live E2EE/TLS/mixed
                  status from real per-recipient CryptoKey/lookup, and the Dlp/scan
                  pre-send warnings. Reports state up via onChange for the send path.
                  A boundary of its own: the panel reads two resources (the key lookup
                  and the DLP scan) that go pending on every recipient or body change,
                  and without it the nearest boundary is the whole signed-in shell
                  (`LazyRoute` in App.tsx), which detached the editor on each keystroke.
                  The group's height is fixed, so the fallback needs no placeholder. */}
              <RibbonGroup label={t('mail-compose-ribbon-security')} priority={20} icon="lock" testId="compose-crypto">
                <Suspense fallback={<RibbonItem/>}>
                  <ComposeCrypto
                    recipients={() => parseRecipients([to(), cc(), bcc()].join(',')).map((r) => r.email)}
                    subject={() => subject()}
                    bodyText={outgoingPlaintext}
                    lookupKeys={lookupKeys}
                    scanDlp={scanDlp}
                    signingKeyRef={() => signingSession()?.keyRef ?? null}
                    onRequestSigningKey={() => {showRibbonTab('security');setUnlockOpen(true);}}
                    onChange={setCryptoState}
                  />
                </Suspense>
              </RibbonGroup>
              {/* Signing-key unlock (plan §2.5, decision flag 2): opens when the sign
                  toggle is switched on while the key is locked, or on a signed send with
                  no cached session. Unlocks the sending key ONCE — subsequent signed
                  sends this session reuse the cached keyRef with no further prompt. */}
              <Show when={unlockOpen()}>
                <RibbonGroup label={t('mail-compose-sign-unlock-title')} priority={21} icon="unlock" testId="compose-sign-unlock">
                  <RibbonItem><div class="compose-rib__row">
                    <label class="compose-rib__field" data-tooltip={t('mail-compose-sign-unlock-note')}>
                      <span class="compose-rib__text">{t('mail-key-passphrase')}</span>
                      <input
                        type="password"
                        class={a11y.focusable}
                        autocomplete="off"
                        data-testid="sign-passphrase"
                        aria-describedby="compose-sign-unlock-note"
                        value={unlockPass()}
                        onInput={(e) => setUnlockPass(e.currentTarget.value)}
                        onKeyDown={(e) => {
                          // Enter unlocks without submitting the outer compose form.
                          if (e.key === 'Enter') void unlockSigningKey(e);
                        }}
                      />
                    </label>
                    <span id="compose-sign-unlock-note" hidden>{t('mail-compose-sign-unlock-note')}</span>
                  </div></RibbonItem>
                  <RibbonItem>
                    <button
                      type="button"
                      class={`btn btn--primary ${a11y.focusable}`}
                      data-testid="sign-unlock-submit"
                      disabled={unlocking()}
                      onClick={(e) => void unlockSigningKey(e)}
                    >
                      <Icon name="unlock"/><span class="compose-rib__text">{unlocking() ? t('mail-compose-sign-unlocking') : t('mail-compose-sign-unlock')}</span>
                    </button>
                  </RibbonItem>
                  <Show when={unlockError()}>
                    {(message) => <RibbonItem><p class="login__error compose-rib__error" role="alert" data-tooltip={message()}>{message()}</p></RibbonItem>}
                  </Show>
                </RibbonGroup>
              </Show>
            </>},
            {id:'delivery',before:()=><>
              <RibbonGroup label={t('mail-compose-ribbon-delivery')} priority={10} icon="sent" testId="compose-send-options">
                <RibbonItem><label class="compose-rib__field"><span class="compose-rib__text">{t('mail-compose-importance')}</span><select value={importance()} onChange={event=>setImportance(event.currentTarget.value as 'high'|'normal'|'low')}><For each={['high','normal','low'] as const}>{value=><option value={value}>{t(`mail-compose-importance-${value}`)}</option>}</For></select></label></RibbonItem>
                <Show when={featureOn('compose.scheduled_send')}><RibbonItem>
                  <label class="compose-rib__field">
                    <span class="compose-rib__text">{t('mail-compose-send-later')}</span>
                    <input
                      type="datetime-local"
                      min={minSendAt()}
                      value={sendAt()}
                      onFocus={() => setMinSendAt(nextLocalMinute(new Date()))}
                      onInput={(e) => setSendAt(e.currentTarget.value)}
                      // The browser blocks the submit itself for a value under `min`;
                      // say why in the dialog as well as in its own bubble.
                      onInvalid={() => {showRibbonTab('delivery');setError(t('mail-compose-send-later-past'));}}
                    />
                  </label>
                </RibbonItem></Show>
                <RibbonItem><label><input type="checkbox" checked={attachPublicKey()} disabled={busy()} onChange={e=>setAttachPublicKey(e.currentTarget.checked)}/>{t('mail-compose-attach-public-key')}</label></RibbonItem>
                <RibbonItem><label><input type="checkbox" checked={includeFileHashes()} disabled={busy()} onChange={e=>setIncludeFileHashes(e.currentTarget.checked)}/>{t('mail-compose-include-hashes')}</label><Show when={includeFileHashes()&&!hashesReady()}><span role="status" data-testid="compose-hash-status">{t(hashResult()?.error?'mail-compose-hashes-failed':'mail-compose-hashes-pending')}</span></Show></RibbonItem>
                <RibbonItem><label class="compose-rib__field"><span class="compose-rib__text">{t('mail-compose-sent-copy')}</span><select value={sentCopyFolder()} disabled={busy()} onChange={e=>setSentCopyFolder(e.currentTarget.value)}><option value="">{t('mail-compose-sent-copy-default')}</option><For each={app.mailboxes().filter(box=>box.myRights?.mayAddItems!==false)}>{box=><option value={box.id}>{box.name}</option>}</For></select></label></RibbonItem>
                <SendOptions
                  deliveryLocked={settings()['mail.requestDeliveryReceipt']?.locked===true}
                  receiptLocked={settings()['mail.requestReadReceipt']?.locked===true}
                  deliveryAvailable={deliveryAvailable()}
                  requestDelivery={requestDelivery()}
                  onRequestDelivery={setRequestDelivery}
                  receiptAvailable={receiptAvailable() && featureOn('compose.read_receipts')}
                  requestReceipt={requestReceipt()}
                  onRequestReceipt={setRequestReceipt}
                  trackingAvailable={trackingAvailable()}
                  trackingLocked={settings()['mail.openTracking']?.locked===true}
                  tracking={trackOpens()}
                  onTracking={setTrackOpens}
                />
              </RibbonGroup>
            </>},
          ]}
        />
        <div class="field field--grow">
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
                <Show when={!sourceMode()} fallback={<HtmlSourceEditor reply={replyContext()} externalHistory ribbonSlots={ribbon.slots} initialHtml={bodyHtml()} externalText={body} owned={ownedCids} upload={uploadInline} preview={async cid=>{const image=inlineImages().find(image=>image.cid===cid);return image?.previewUrl?blobUrlToDataUrl(image.previewUrl,image.type):null;}} onChange={onEditorChange} onReady={setEditorApi}/>}><RichTextEditor
                  externalHistory
                  ribbonSlots={ribbon.slots}
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
          <Show when={imageOffer && ['ask', 'loading', 'cancelled', 'error'].includes(offerState())}>
            <div class="compose__image-offer" role="group" aria-label={t('mail-reply-images-title')} data-testid="compose-image-offer" aria-busy={offerState() === 'loading'}>
              <span class="compose__image-offer-text" role="status">
                {offerState() === 'loading' ? t('mail-reply-images-loading') : offerState() === 'cancelled' ? t('mail-reply-images-cancelled') : offerState() === 'error' ? t('mail-reply-images-failed') : t('mail-reply-images-ask')}
              </span>
              <Show when={offerState() === 'loading'} fallback={<>
                <button type="button" class="btn btn--ghost" data-testid="image-offer-include" onClick={() => void includeQuoteImages(false)}>{offerState() === 'ask' ? t('mail-reply-images-include') : t('mail-reply-images-retry')}</button>
                <Show when={offerState() === 'ask'}>
                  <button type="button" class="btn btn--ghost" data-testid="image-offer-dismiss" onClick={() => setOfferState('dismissed')}>{t('mail-reply-images-not-now')}</button>
                  <Show when={imageOffer?.canTrust && imageOffer.domain !== ''}><button type="button" class="btn btn--ghost" data-testid="image-offer-trust" onClick={() => void includeQuoteImages(true)}>{t('mail-reply-images-trust', { domain: imageOffer!.domain })}</button></Show>
                </Show>
              </>}>
                <button type="button" class="btn btn--ghost" data-testid="image-offer-cancel" onClick={cancelQuoteImages}>{t('mail-image-cancel')}</button>
              </Show>
            </div>
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
