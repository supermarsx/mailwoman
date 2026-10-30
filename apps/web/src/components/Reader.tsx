import {AsyncPending,AsyncError} from './AsyncState.tsx';
import {RemoteImageDownloads} from './RemoteImageDownloads.tsx';
import {SaveToNextcloud} from '../modules/nextcloud/SaveToNextcloud.tsx';
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  on,
  onCleanup,
  onMount,
  Show,
  type JSX,
  type Resource,
} from 'solid-js';
import { printMessage } from '../viewers/printMessage.ts';
import { MessageTimes } from './TimeHelpers.tsx';
import { settings as effectiveSettings } from '../api/settings.ts';
import { AppContext, useApp } from '../state/context.ts';
import type { MessageView } from '../state/slices/messageView.ts';
import { createEntryMotion } from '../shell/createEntryMotion.ts';
import * as motion from '../styles/motion.css.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import { formatDateTime } from '../i18n/format.ts';
import { useActions } from '../keymap/index.ts';
import * as a11y from './mailA11y.css.ts';
// Reading-pane layout switch (W3): the globalStyle overrides keyed on
// `:root[data-reading-pane]`. Imported for its side effect so the rules ship.
import './readerPane.css.ts';
import './readerMetadata.css';
import { createNarrowViewport, focusIsLost } from './narrowViewport.ts';
import { readingPane } from './readingPane.ts';
import { accountUiPreferences } from '../api/accountPreferences.ts';
import { ThumbnailStrip, type StripItem } from '../viewers/ThumbnailStrip.tsx';
import { AttachmentViewer } from '../viewers/AttachmentViewer.tsx';
import { attachmentDecision, attachmentPolicyUrl, loadAttachmentPolicy } from '../api/attachmentPolicy.ts';
import { buildDownloadUrl, fetchObjectUrl, type AttachmentPart } from '../viewers/attachments.ts';
import { SecurityPanel } from './SecurityPanel.tsx';
import { SpamScore } from './security/SpamScore.tsx';
import { DeliveryStatus } from './security/DeliveryStatus.tsx';
import { ReadReceipt } from './ReadReceipt.tsx';
import { MailingListActions } from './MailingListActions.tsx';
import {ReaderToolbar} from './ReaderToolbar.tsx';
import {reviewMessageLinks} from '../viewers/messageLinks.ts';
import { InviteCard, hasCalendarPart } from './InviteCard.tsx';
// V7 auto-tag (SPEC §14.3, e14b): model-suggested labels for the open message.
// Gated on the `auto-tag` Assist capability, so a Disabled gateway renders nothing.
import { AutoTag, type TagSuggestion } from '../modules/assist/index.ts';
import type { SenderControlRequest, SenderControlResult } from './security/model.ts';
import { ReaderSearchMenu } from './search/ReaderSearchMenu.tsx';
import { defaultSenderControl } from './security/model.ts';
import { createMaxSecurityStore } from '../viewers/max-security.ts';
import { bodyFrameDoc } from '../viewers/sandbox.ts';
import { createObjectUrlOwner } from '../viewers/objectUrl.ts';
import { getCryptoWorker } from '../crypto/index.ts';
import { pickSignerArmors } from '../crypto/signerMatch.ts';
import { createConfiguredClient } from '../api/transport.ts';
import { fetchThreadingHeaders, parentReferences, responseFor, type ThreadingHeaders } from '../api/jmap.ts';
import { CAP_CORE, CAP_MAIL } from '../api/jmap-types.ts';
import { CAP_CRYPTO, CAP_SECURITY } from '../api/crypto-types.ts';
import type { Email, EmailAddress } from '../api/jmap-types.ts';
import type { SecurityVerdict, SignatureVerdict } from '../api/security-types.ts';
import {
  analyzeBlockedContent,
  coveringGrant,
  createRemoteImageApi,
  hasBlockedContent,
  type GrantScope,
  type MutationOptions,
} from '../api/remote-images.ts';
import { extractHtmlBody } from '../state/slices/mail.ts';
import { RemoteContentBar } from './RemoteContentBar.tsx';
import { LinkedFilesBar } from './LinkedFilesBar.tsx';
import { NoticeButton, ReaderNoticeBar, noticeCss } from './ReaderNoticeBar.tsx';
import { createContextMenu, type ContextAction } from './ContextMenu.tsx';
import { formatBytes } from '../i18n/format.ts';
import { writeSetting } from '../api/settings.ts';
import { createRemoteImageLoad } from './remoteImageToast.ts';
import { DirectReaderFrame } from './DirectReaderFrame.tsx';
import { resourceValue } from './resourceValue.ts';
import { useReaderZoom } from './readerZoom.ts';
import { ReaderZoomBox } from './ReaderZoomBox.tsx';
import { remotePolicy, remoteServerBase, signContent, readerReferences, rewriteReaderContent, BLOCKED_POLICY, EMPTY_SIGNED, type RemotePolicy } from '../api/remote-content.ts';
import { settings } from '../api/settings.ts';
import { themeCssVars } from '../theme/themeCssVars.ts';
import { MessageLoading } from './MessageLoading.tsx';
import { Icon } from './Icon.tsx';
import { RemoteChangeNotice } from './RemoteChangeNotice.tsx';
import type { ReaderAppearance } from '../viewers/readerAppearance.ts';
import { isNativeAuth } from '../api/serverAddress.ts';
import {
  formatAddressList,
  forwardSubject,
  replyRecipients,
  replySubject,
  replyThreading,
  type ComposeInitial,
  type ReplyMode,
} from './compose/reply.ts';
import type { QuoteSource } from './compose/quote.ts';
import { stripIsolates } from '../i18n/index.ts';

// The crypto/security JMAP surface (`SecurityVerdict/get`, `SenderControl/set`)
// is not exposed on `AppState`, so this component drives it over its own client
// (stateless — hits the same session as the store's client; browser: same-origin
// cookie, native shell: configured base + bearer). The max-security policy is an
// app-singleton (per-sender + global, persisted in localStorage) — plan §2.5.
const jmapClient = createConfiguredClient();
const maxsec = createMaxSecurityStore();
// Remote-image grant surface (t16 §S8/S9): the reader drives grant/revoke/list
// over e6's same-origin REST endpoints. The wire shapes are localized in
// `api/remote-images.ts` (the e6 seam); this reader only calls the interface.
const remoteImages = createRemoteImageApi();

const SECURITY_USING = [CAP_CORE, CAP_CRYPTO, CAP_SECURITY];

/**
 * A resource's value once it has one for its CURRENT source, else `undefined`.
 *
 * Calling a resource while it is loading suspends the nearest Suspense
 * boundary, and the nearest one above the reader is the one around the whole
 * mailbox screen (`LazyRoute`, App.tsx): every message open took the list, the
 * sidebar and the reader out of the document until the verdict and the grants
 * had loaded. `state` does not suspend, and `latest` does not once the resource
 * is ready. A refetch for another message reads as `undefined` here rather
 * than as the previous message's value.
 */
function settled<T>(resource: Resource<T>): T | undefined {
  return resource.state === 'ready' ? resource.latest : undefined;
}

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

/** A message date as the attribution line and the forwarded header show it,
 *  in the app's locale like the message list's dates. Empty when unreadable. */
function quoteDate(iso: string | null | undefined): string {
  if (iso === null || iso === undefined) return '';
  return formatDateTime(iso);
}

/** Attachment thumbnails + the on-click viewer, rendered AROUND (never inside)
 *  the sandboxed message iframe. Since t110 they sit in a collapsible reader
 *  notice bar ("3 attachments · 4.2 MB"), last in the notice stack; the open or
 *  closed state is the user's `reader.attachmentsCollapsed` setting (default
 *  open, as before), and the thumbnails load only once the bar is open. Each viewer keeps its own sandbox (§2.4). In a
 *  locked-down max-security mode attachments open ONLY through this re-encode
 *  preview jail (the AttachmentViewer sandbox), never as original bytes (§7.2). */
export function AttachmentsPane(props: { email: Email; onOpenAttachment?: (item: StripItem) => void }): JSX.Element {
  const app = useApp();
  const cloudAccount=app.accountId();
  const [cloudItem,setCloudItem]=createSignal<StripItem|null>(null);
  onMount(()=>{void loadCatalog('nextcloud');void app.nextcloud.ensureEnabled();});
  createEffect(on(()=>props.email.id,()=>setCloudItem(null),{defer:true}));

  const [attachmentPolicy,{refetch:retryPolicy}]=createResource(()=>((props.email as {attachments?:AttachmentPart[]}).attachments?.length??0)>0?props.email.id:undefined,loadAttachmentPolicy);
  const [acknowledged,setAcknowledged]=createSignal(false);
  const decision=(item:StripItem)=>attachmentPolicy.state==='ready'?attachmentDecision(attachmentPolicy()!,item):undefined;
  const [openItem, setOpenItem] = createSignal<StripItem | null>(null);

  const items = createMemo<StripItem[]>(() => {
    const parts = (props.email as { attachments?: AttachmentPart[] }).attachments ?? [];
    return parts
      .filter((a) => a.blobId !== null && a.blobId !== undefined && a.blobId !== '')
      .map((a) => ({
        blobId: a.blobId as string,
        emailId: props.email.id,
        name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : t('mail-attachment-unnamed'),
        mime: a.type.length > 0 ? a.type : 'application/octet-stream',
        size: a.size,
      }));
  });

  function blobUrlFor(item: StripItem, acknowledge=false): Promise<string> {
    const allowed=decision(item);
    if(!allowed?.preview || (allowed.warning && !acknowledge))return Promise.resolve('');
    const url = app.downloadUrl();
    const acct = app.accountId();
    if (url === null || acct === null) return Promise.resolve('');
    return fetchObjectUrl(
      attachmentPolicyUrl(buildDownloadUrl(url, { accountId: acct, blobId: item.blobId, name: item.name, mime: item.mime }),item.emailId,true,acknowledge),
    );
  }

  // Every object URL this pane mints is owned (t22 L6). Without this the opened
  // attachment's Blob stayed pinned for the tab's lifetime: the modal's URL is
  // revoked when the open item changes and when the pane unmounts, and each
  // thumbnail's URL is owned by its own <Thumb> (ThumbnailStrip.tsx).
  const urls = createObjectUrlOwner();
  /** The URL the viewer is currently showing, so it can be released the moment
   *  it is superseded rather than banked until the pane unmounts. */
  let shownUrl: string | null = null;

  const [blobUrl, { refetch: retryAttachment }] = createResource(() => {const item=openItem();return item && decision(item)?.preview && (!decision(item)?.warning||acknowledged()) ? {item,ack:acknowledged()}:undefined;}, async ({item,ack}) => {
    if (shownUrl !== null) {
      urls.release(shownUrl);
      shownUrl = null;
    }
    shownUrl = urls.adopt(await blobUrlFor(item,ack));
    return shownUrl;
  });

  // Closing the viewer sets `openItem` to null, which does NOT run the fetcher —
  // and Solid keeps a resource's last value when its source goes falsy, so the
  // close path cannot be driven off the resource value. Release here instead.
  createEffect(
    on(
      openItem,
      (item) => {
        if (item === null && shownUrl !== null) {
          urls.release(shownUrl);
          shownUrl = null;
        }
      },
      { defer: true },
    ),
  );

  onMount(() => void loadCatalog('reader'));
  const collapseSetting = () => effectiveSettings()['reader.attachmentsCollapsed'];
  const [collapsedLocal, setCollapsedLocal] = createSignal<boolean | null>(null);
  const collapsed = (): boolean => collapsedLocal() ?? collapseSetting()?.value === true;
  /** Mount the thumbnails on first open, then keep them (no refetch on toggle). */
  const [everOpen, setEverOpen] = createSignal(false);
  createEffect(() => { if (!collapsed()) setEverOpen(true); });
  function toggle(): void {
    const next = !collapsed();
    setCollapsedLocal(next);
    const entry = collapseSetting();
    if (entry && !entry.locked) void writeSetting('reader.attachmentsCollapsed', next).catch(() => undefined);
  }
  const totalSize = createMemo(() => items().reduce((sum, item) => sum + item.size, 0));
  const summaryText = (): string => t('reader-attachments-summary', { count: items().length, size: formatBytes(totalSize()) });
  const tooltip = (): string => [summaryText(), ...items().slice(0, 8).map((item) => `${isolate(item.name)} · ${formatBytes(item.size)}`), ...(items().length > 8 ? ['…'] : [])].join('\n');
  const panelId = `reader-attachments-${Math.random().toString(36).slice(2)}`;
  const cloudEnabled = (): boolean => app.nextcloud.enabled() && cloudAccount === app.accountId();
  const cloudActions = createMemo<ContextAction[]>(() => items().map((item, i) => ({ id: `nc${i}`, label: t('nextcloud-save-attachment', { name: isolate(item.name) }), icon: 'nextcloud', disabled: !decision(item)?.allowed, run: () => { setCloudItem(item); } })));
  const cloudMenu = createContextMenu({ label: () => t('nextcloud-save-options'), scope: () => props.email.id, actions: cloudActions });

  return (
    <Show when={items().length > 0}>
      <ReaderNoticeBar
        label={t('mail-attachments')}
        testid="reader-attachments-bar"
        state={collapsed() ? 'collapsed' : 'expanded'}
        icon="attachment"
        tooltip={tooltip()}
        actions={<>
          <Show when={cloudEnabled()}>
            <NoticeButton icon="nextcloud" label={t('nextcloud-save-panel-label')} haspopup="menu" expanded={cloudMenu.isOpen()} onClick={(e) => cloudMenu.open(e)} onKeyDown={cloudMenu.keydown} />
          </Show>
          <NoticeButton icon={collapsed() ? 'chevronDown' : 'chevronUp'} iconOnly label={collapsed() ? t('reader-attachments-show') : t('reader-attachments-hide')} expanded={!collapsed()} controls={panelId} testid="reader-attachments-toggle" onClick={toggle} />
        </>}
        after={<>
        {cloudMenu.menu()}
        <Show when={cloudEnabled() && cloudItem()} keyed>{item=><div class={noticeCss.panel}><section aria-label={t('nextcloud-save-options')}><button type="button" class="btn" onClick={()=>setCloudItem(null)}>{t('nextcloud-save-close')}</button><SaveToNextcloud {...(cloudAccount?{accountId:cloudAccount}:{})} attachment={{emailId:props.email.id,blobId:item.blobId,name:item.name}} service={app.nextcloud.service} onSaved={()=>setCloudItem(null)}/></section></div>}</Show>
        <div id={panelId} class={noticeCss.panel} hidden={collapsed()}>
      <section class="reader__attachments" aria-label={t('mail-attachments')} data-testid="reader-attachments">
        <Show when={everOpen()}>
        <Show when={attachmentPolicy.state==='ready'} fallback={<Show when={attachmentPolicy.error} fallback={<AsyncPending message={t('mail-attachment-policy-loading')}/>}><AsyncError message={t('mail-attachment-policy-unavailable')} onRetry={()=>{void retryPolicy();}}/></Show>}>
        <ThumbnailStrip
          items={items()}
          selectedBlobId={openItem()?.blobId ?? ''}
          onSelect={(it) => {setAcknowledged(false); if(props.onOpenAttachment)props.onOpenAttachment(it);else setOpenItem(it);}}
          resolveThumb={(item)=>blobUrlFor(item)}
        />
        </Show>
        </Show>
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
                  <Icon name="close" />
                </button>
              </div>
              <Show when={attachmentPolicy.loading}><p role="status">{t('mail-attachment-policy-loading')}</p></Show>
              <Show when={attachmentPolicy.error}><p role="alert">{t('mail-attachment-policy-unavailable')}</p></Show>
              <Show when={decision(item()) && !decision(item())?.allowed}><p role="alert">{t('mail-attachment-policy-blocked')}</p></Show>
              <Show when={decision(item())?.warning && !acknowledged()}><p role="alert">{t('mail-attachment-policy-warning')}</p><button class="btn" onClick={()=>setAcknowledged(true)}>{t('mail-attachment-policy-acknowledge')}</button></Show>
              <Show when={decision(item())?.allowed && !decision(item())?.preview}><p>{t('mail-attachment-policy-no-preview')}</p></Show>
              <Show when={decision(item())?.preview && (!decision(item())?.warning || acknowledged())}>
              <Show when={settled(blobUrl)} fallback={<Show when={blobUrl.state === 'errored'} fallback={<p class="attachment-modal__loading">{t('mail-attachment-loading')}</p>}><p role="alert">{t('mail-attachment-failed')}</p><button type="button" class="btn" onClick={() => void retryAttachment()}>{t('common-retry')}</button></Show>}>
                {(url) => (
                  <AttachmentViewer
                    part={{ partId: null, blobId: item().blobId, size: item().size, type: item().mime }}
                    blobUrl={url()}
                    mime={item().mime}
                    name={item().name}
                  />
                )}
              </Show>
              </Show>
            </div>
          )}
        </Show>
      </section>
        </div>
        </>}
      >
        <span class={noticeCss.headline}>{summaryText()}</span>
      </ReaderNoticeBar>
    </Show>
  );
}

/** The unlock affordance for an encrypted (PGP) message: takes the passphrase,
 *  runs the client-side WASM decrypt in the crypto worker (§1.2/§1.3 — private
 *  material + plaintext never leave the client), and hands the plaintext + the
 *  signature verdict back to the reader. */
function DecryptPanel(props: {
  armor: string;
  emailId: string;
  senderAddress: string;
  onDecrypted: (content: { html?: string; text?: string }, signature: SignatureVerdict, emailId: string, accountId: string | null) => void;
}): JSX.Element {
  const app = useApp();
  const [passphrase, setPassphrase] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Ensure own keys are loaded so we can find the private bundle to decrypt with.
  onMount(() => {
    if (app.ownKeys().length === 0) void app.loadKeys();
  });
  // A mailbox switch (t87 D11) forgets a typed passphrase; the next mailbox's
  // keys load for it and unlock with its own passphrase.
  createEffect(on(() => app.keysEpoch(), () => {
    setPassphrase('');
    setError(null);
    if (app.ownKeys().length === 0) void app.loadKeys().catch(() => undefined);
  }, { defer: true }));

  /** Resolve the sender's armored PGP public keys so the worker can VERIFY the
   *  embedded signature — mirrors how compose resolves recipient keys: first the
   *  already-loaded keyring (own + harvested/looked-up), then the same
   *  `CryptoKey/lookup` discovery (harvested/autocrypt/WKD/VKS).
   *
   *  The worker says "verified" for any key that matches the signature, so every
   *  candidate must be a live (not revoked, not expired) key bound to exactly the
   *  From address; see `pickSignerArmors`. ALL such keys are passed, not just the
   *  first: after a sender rotates keys, both the old and the new key are valid,
   *  and mail signed by either must verify. Returns an empty list when no such
   *  key is known, which yields an honest unverified verdict. */
  async function resolveSignerPublicKeys(address: string): Promise<string[]> {
    if (address === '') return [];
    const known = pickSignerArmors(address, app.keys(), new Date());
    if (known.length > 0) return known;
    try {
      const found = await app.lookupContactKey(address, ['harvested', 'autocrypt', 'wkd', 'vks']);
      return pickSignerArmors(address, found, new Date());
    } catch {
      // Key discovery failure → no signer key → honest unverified verdict (never
      // a hard error, and never a false "verified").
    }
    return [];
  }

  async function decryptNow(): Promise<void> {
    setError(null);
    setBusy(true);
    try {
      const requestedEmail = props.emailId;
      const requestedAccount = app.accountId();
      // Only the open mailbox's own keys (t87 D11): a key of another mailbox is
      // never offered to the worker.
      if (app.ownKeys().length === 0) await app.loadKeys();
      const own = app.ownKeys().find(
        (k) => k.kind === 'pgp' && k.encryptedPrivateBackup !== null && requestedAccount !== null && app.keyOwnerAccount(k) === requestedAccount,
      );
      const bundle = own?.encryptedPrivateBackup ?? null;
      if (bundle === null) throw new Error(t('mail-decrypt-no-key'));
      const signerPublicKeys = await resolveSignerPublicKeys(props.senderAddress);
      if (props.emailId !== requestedEmail || app.accountId() !== requestedAccount) return;
      const result = await getCryptoWorker().decrypt({
        readerPolicy: { blockTrackers: false, cleanLinks: true },
        kind: 'pgp',
        ciphertext: props.armor,
        encryptedPrivateBundle: bundle,
        passphrase: passphrase(),
        ...(signerPublicKeys.length > 0 ? { signerPublicKeys } : {}),
      });
      if (props.emailId !== requestedEmail || app.accountId() !== requestedAccount) return;
      // The worker sanitized any HTML plaintext IN-WORKER (§1.3): `plaintextHtml` is
      // already safe to render as HTML; `plaintextText` renders escaped.
      const content =
        result.plaintextHtml !== undefined
          ? { html: result.plaintextHtml }
          : { text: result.plaintextText ?? '' };
      props.onDecrypted(content, result.signature, requestedEmail, requestedAccount);
    } catch (err) {
      setError(err instanceof Error ? err.message : t('mail-decrypt-failed'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section class="reader__decrypt" data-testid="reader-decrypt" aria-label={t('mail-encrypted-region')}>
      <p class="reader__decrypt-note"><Icon name="lock" /> {t('mail-encrypted-note')}</p>
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
      suggestions={settled(suggestions) ?? []}
      mode={app.assist.autoTagMode()}
      onModeChange={app.assist.setAutoTagMode}
      onApply={(kw) => void app.applyTag(props.email.id, kw)}
      onRevert={(kw) => void app.removeTag(props.email.id, kw)}
      onAudit={app.assist.recordTagAudit}
    />
  );
}


interface ReaderProps {
  onExport?: (messageId: string) => void;
  onOpenAttachment?: (item: StripItem) => void;
  onCompose?: (initial: ComposeInitial) => void;
  view?: MessageView;
  onClose?: () => void;
  active?: boolean;
  /** Open Settings at a section (the remote-images banner links to Privacy). */
  onOpenSettings?: (section: string) => void;
}
export function Reader(props: ReaderProps): JSX.Element {
  const app=useApp();
  const scoped={...app,
    openEmail:()=>props.view ? props.view.openEmail() : app.openEmail(),
    sanitizedHtml:()=>props.view ? props.view.sanitizedHtml() : app.sanitizedHtml(),
    readLoading:()=>props.view ? props.view.readLoading() : app.readLoading(),
    openMessage:(id:string)=>props.view ? props.view.openMessage(id) : app.openMessage(id),
    closeMessage:()=>props.onClose ? props.onClose() : props.view ? props.view.closeMessage() : app.closeMessage(),
    exportMessage:(id?:string)=>{const selected=id ?? (props.view ? props.view.openEmail()?.id : app.openEmail()?.id);if(selected && props.onExport){props.onExport(selected);return Promise.resolve();}return selected ? app.exportMessage(selected) : Promise.resolve();},
  };
  return <AppContext.Provider value={scoped}><ReaderBody {...props} /></AppContext.Provider>;
}
function ReaderBody(props: ReaderProps): JSX.Element {
  const app = useApp();
  onMount(() => { void loadCatalog('remote-images'); void loadCatalog('privacy'); });

  // A memo, so readers of the id (the decrypt reset, the verdict fetch, the entry
  // motion) run when ANOTHER message opens, not on every update of the open one:
  // a keyword write (the automatic mark-read a second after opening) replaces the
  // openEmail object, and the decrypted body was reset with it.
  const emailId = createMemo((): string | null => app.openEmail()?.id ?? null);
  const entering = createEntryMotion(emailId);

  // Opening a message never takes focus from a list that is still on screen
  // (t51): the row that opened it keeps focus, so the arrow keys go on working.
  // Only where the reader replaces the list (phone width, t28-e6; the reading
  // pane set to Off; the list collapsed) is the focused row hidden, and focus
  // would fall to <body>: then, and only if it really was lost, it moves to the
  // subject heading. Tab or a click moves focus into the reader otherwise.
  const narrow = createNarrowViewport();
  const listCovered = (): boolean => narrow() || readingPane() === 'off' || accountUiPreferences().messageListCollapsed === true;
  let subjectHeading: HTMLHeadingElement | undefined;
  createEffect(
    on(emailId, (id) => {
      if (id !== null && props.active !== false && listCovered()) queueMicrotask(() => { if (focusIsLost()) subjectHeading?.focus(); });
    }),
  );

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
    app.accountId();
    setClientSig(null);
    setDecryptedHtml(null);
    setDecryptedText(null);
  });

  const verdict = createMemo<SecurityVerdict | null>(() => {
    const v = settled(serverVerdict);
    if (v === undefined || v === null) return null;
    const sig = clientSig();
    if (sig === null) return v;
    // Overlay the client-computed signature + flag the client-decrypt path.
    return { ...v, signature: sig, encryption: { ...v.encryption, decryptsClientSide: true } };
  });

  const armor = (): string | null => extractPgpArmor(app.openEmail());
  const showDecrypt = (): boolean =>
    armor() !== null && decryptedHtml() === null && decryptedText() === null;

  // Active grants for the account, so the bar can show "turn off" (revoke) once a
  // grant covers the open message, and the body render can route that message's
  // remote images through the proxy. Resilient: a missing/failing endpoint yields
  // no grants, so the blocked state still renders. Defined before `bodySrcdoc`
  // because it reads `activeGrant()` (Solid memos evaluate eagerly on creation).
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
    return coveringGrant(settled(grants) ?? [], { emailId: id, sender: sender() });
  });

  const [bodyDisplays, setBodyDisplays] = createSignal(0);
  const [receiptDisplayed,setReceiptDisplayed]=createSignal<{key:string;html:string}|null>(null);
  const receiptDocumentKey=()=>JSON.stringify([app.accountId(),emailId()]);
  const [listHeaders,setListHeaders]=createSignal<{scope:string;url:string;raw:string}|null>(null);
  const messageHeaderUrl=()=>{
    const email=app.openEmail(),accountId=app.accountId(),template=app.downloadUrl();
    return email?.blobId&&accountId&&template?buildDownloadUrl(template,{accountId,blobId:email.blobId,name:'message.eml',mime:'message/rfc822'}):null;
  };
  function collectListHeaders(raw:string|null,url:string):void {
    if(url===messageHeaderUrl())setListHeaders(raw===null?null:{scope:receiptDocumentKey(),url,raw});
  }
  // t110: the mailing-list bar sits above the body, and its headers come from a
  // separate fetch (SpamScore). The body and the notice stack wait for that answer
  // (raw headers, or an empty string on failure), at most 0.5 s, so the bar never appears
  // above a body that is already shown.
  const [headerWaitOver,setHeaderWaitOver]=createSignal<string|null>(null);
  createEffect(on(receiptDocumentKey,(key)=>{const timer=setTimeout(()=>setHeaderWaitOver(key),500);onCleanup(()=>clearTimeout(timer));}));
  const listHeadersSettled=():boolean=>{
    const url=messageHeaderUrl();if(url===null||headerWaitOver()===receiptDocumentKey())return true;
    const h=listHeaders();return h!==null&&h.scope===receiptDocumentKey()&&h.url===url;
  };
  function receiptDidDisplay(html:string,key:string|undefined):void {
    if(key===receiptDocumentKey()&&html===bodySrcdoc()){setReceiptDisplayed({key,html});setBodyDisplays(n=>n+1);}
  }
  // Keyed by a string, so only a change to these values (or the message) asks
  // again: any other preference write (the zoom level, t79) replaces the settings
  // map too, and a fresh key object would re-sign and reload the body.
  const [privacy] = createResource(
    () => app.accountId() && emailId() ? JSON.stringify([app.accountId(), emailId(), ['privacy.remoteContent', 'privacy.trackers', 'privacy.cleanLinks', 'privacy.remoteImageDownloads'].map(key => settings()[key]?.value ?? null)]) : null,
    async () => { try { const p = await remotePolicy(); return isNativeAuth() && p.mode === 'direct' ? { ...p, mode: 'ask' as const, allowDirect: false } : p; } catch { return BLOCKED_POLICY; } },
  );
  const policy = (): RemotePolicy => settled(privacy) ?? BLOCKED_POLICY;
  const [readerBody] = createResource(
    () => {
      const email = app.openEmail(); const account = app.accountId();
      if (!email || !account || privacy.loading || grants.loading) return null;
      return { email, account, policy: policy(), securityMode: maxsec.effectiveMode(sender()), decrypted: decryptedHtml(), granted: activeGrant() !== null, fallback: app.sanitizedHtml() };
    },
    async k => {
      let html = k.decrypted ?? k.fallback ?? ''; let effective = k.policy; let failed = false;
      try { if (k.decrypted === null) html = await getCryptoWorker().sanitizeReader({ html: extractHtmlBody(k.email), blockTrackers: false, cleanLinks: true }); }
      catch { effective = BLOCKED_POLICY; failed = true; }
      const cancelled = () => ({ emailId: k.email.id, account: k.account, html: '', sanitized: '', refs: { urls: [], cids: [], trackers: 0, files: [], links: [] }, policy: BLOCKED_POLICY, failed: false, images: 0 });
      const current = () => app.accountId() === k.account && emailId() === k.email.id && maxsec.effectiveMode(sender()) === k.securityMode;
      if (!current()) return cancelled();
      const refs = readerReferences(html); const cidParts: Record<string,string> = {};
      const walk = (part: unknown): void => {
        if (!part || typeof part !== 'object') return;
        const p = part as { cid?: string; blobId?: string; subParts?: unknown[] };
        if (typeof p.cid === 'string' && typeof p.blobId === 'string') cidParts[p.cid.replace(/^<|>$/g, '')] = p.blobId;
        if (Array.isArray(p.subParts)) p.subParts.forEach(walk);
      };
      if (refs.cids.length) {
        try {
          const property = effective.inlineParts ? 'mailwomanInlineParts' : 'bodyStructure';
          const response = await jmapClient.jmap({ using: [CAP_CORE, CAP_MAIL], methodCalls: [['Email/get', { accountId: k.account, ids: [k.email.id], properties: ['id', property], bodyProperties: ['partId','blobId','cid','subParts','type'] }, 'rc-parts']] });
          const message = responseFor<{ list: Record<string,unknown>[] }>(response, 'rc-parts').list[0];
          const parts = message?.[property]; if (Array.isArray(parts)) parts.forEach(walk); else walk(parts);
        } catch { failed = true; }
      }
      if (!current()) return cancelled();
      const urls = k.securityMode === 'full-sanitized' && (effective.mode === 'proxy' || (effective.mode === 'ask' && k.granted)) ? refs.urls : [];
      const parts = k.securityMode === 'full-sanitized' ? [...new Set(refs.cids.map(id => cidParts[id]).filter((id): id is string => Boolean(id)))] : [];
      let signed = EMPTY_SIGNED;
      if (urls.length || parts.length) { try { signed = await signContent(k.email.id, { urls, parts }); } catch { failed = true; } }
      return { emailId: k.email.id, account: k.account, html: rewriteReaderContent(html, signed, effective, cidParts), sanitized: html, refs, policy: effective, failed, images: signed.images.length };
    },
  );
  const currentBody = () => { const body = readerBody.state === 'ready' || readerBody.state === 'refreshing' ? readerBody.latest : undefined; return body?.emailId === emailId() && body.account === app.accountId() ? body : undefined; };
  const readerAppearance = (): ReaderAppearance => {
    const values = settings();
    const mode = values['reader.colorMode']?.value;
    const colorMode = mode === 'original' || mode === 'adaptive' || mode === 'invert' ? mode : app.appearance() === 'dark' ? 'adaptive' : 'original';
    const width = values['reader.contentWidth']?.value;
    return { colorMode, fontSize: Number(values['reader.fontSize']?.value ?? 14), lineHeight: Number(values['reader.lineHeight']?.value ?? 1.6), contentWidth: width === 'comfortable' || width === 'full' ? width : 'wide' };
  };
  const frameOptions = () => ({ appearance: readerAppearance(), themeVars: readerAppearance().colorMode === 'adaptive' ? themeCssVars(app.theme(), { density: app.density() }) : '' });
  const messageLoading = () => <MessageLoading variant={String(settings()['reader.loading']?.value ?? 'skeleton')} delayMs={Number(settings()['reader.loadingDelayMs']?.value ?? 120)} />;
  const unreviewedBodySrcdoc = createMemo<string | null>(() => {
    const email = app.openEmail(); if (!email) return null;
    const mode = maxsec.effectiveMode(sender());
    const text = decryptedText(); if (text !== null) return bodyFrameDoc('plain-text', { text }, frameOptions());
    if (mode === 'plain-text') return bodyFrameDoc('plain-text', { text: decryptedHtml() ?? plainTextOf(email) }, frameOptions());
    if (mode === 'sanitized-no-media') return bodyFrameDoc(mode, { html: currentBody()?.html ?? decryptedHtml() ?? app.sanitizedHtml() }, frameOptions());
    const body = currentBody(); if (!body || body.emailId !== email.id || body.account !== app.accountId()) return null;
    return bodyFrameDoc('full-sanitized', { html: body.html }, { ...frameOptions(), direct: body.policy.mode === 'direct' && body.policy.allowDirect, serverBase: remoteServerBase() });
  });

  const reviewedBody=createMemo(()=>{const html=unreviewedBodySrcdoc();return html===null?null:reviewMessageLinks(html,t('mail-links-inert'));});
  const bodySrcdoc=()=>reviewedBody()?.html??null;
  const receiptDocument=createMemo(()=>({key:receiptDocumentKey(),html:bodySrcdoc()}));

  // ── Reply / Reply all / Forward ────────────────────────────────────────────
  /** What the reader is showing of the open message, which is all a quote may
   *  hold: the decrypted content of an encrypted message once it has been
   *  unlocked (and nothing before that), the plain text in plain-text mode, and
   *  otherwise the sanitised HTML. `decrypted` marks the first case. */
  function displayedContent(email: Email): { source: QuoteSource; decrypted: boolean } {
    const decHtml = decryptedHtml();
    if (decHtml !== null) return { source: { html: decHtml }, decrypted: true };
    const decText = decryptedText();
    if (decText !== null) return { source: { text: decText }, decrypted: true };
    if (armor() !== null) return { source: {}, decrypted: false };
    if (maxsec.effectiveMode(sender()) === 'plain-text') {
      return { source: { text: plainTextOf(email) }, decrypted: false };
    }
    const html = app.sanitizedHtml();
    return { source: html !== null ? { html } : { text: plainTextOf(email) }, decrypted: false };
  }

  /** The open message's `Message-ID` and `References`. From `Email/get` when
   *  it has the `messageId` property at all — `null` there means the server
   *  found no usable id. The header block of the raw message is read only when
   *  the property is absent: the engine returns it for mail ingested since it
   *  learned to, and not for rows stored before. `null` when there is no id. */
  async function threadingHeaders(email: Email): Promise<ThreadingHeaders | null> {
    if ('messageId' in email) {
      // The server read the headers: `null` is its answer (no id, or not an
      // id list), not a reason to read them again here.
      const id = email.messageId?.[0];
      if (id === undefined) return null;
      return { messageId: id, references: parentReferences(email.references ?? [], email.inReplyTo ?? []) };
    }
    const url = app.downloadUrl();
    const acct = app.accountId();
    const blobId = email.blobId;
    if (url === null || acct === null || blobId === undefined || blobId === '') return null;
    try {
      return await fetchThreadingHeaders(
        buildDownloadUrl(url, { accountId: acct, blobId, name: 'message.eml', mime: 'message/rfc822' }),
      );
    } catch {
      return null;
    }
  }

  const [preparing, setPreparing] = createSignal(false);

  async function composeFrom(email: Email, mode: ReplyMode): Promise<ComposeInitial> {
    const { quoteForReply, quoteForForward, quoteForOutlook } = await import('./compose/quote.ts');
    const shown = displayedContent(email);
    const author = email.from?.[0];
    const authorName = stripIsolates((author?.name ?? '').trim() !== '' ? author!.name! : (author?.email ?? ''));
    const date = quoteDate(email.sentAt ?? email.receivedAt);
    const outlook = effectiveSettings()['mail.quoteStyle']?.value === 'outlook';
    const outlookFields = [
      { label: t('mail-quote-header-from'), value: stripIsolates(addressList(email.from)) },
      ...(date ? [{ label: t('mail-quote-header-sent'), value: date }] : []),
      { label: t('mail-quote-header-to'), value: stripIsolates(addressList(email.to)) },
      ...((email.cc ?? []).length ? [{ label: t('mail-quote-header-cc'), value: stripIsolates(addressList(email.cc ?? null)) }] : []),
      { label: t('mail-quote-header-subject'), value: stripIsolates(email.subject ?? '') },
    ];

    if (mode === 'forward') {
      const lines = [
        t('mail-forward-heading'),
        t('mail-forward-from', { addr: stripIsolates(addressList(email.from)) }),
        ...(date !== '' ? [t('mail-forward-date', { date })] : []),
        t('mail-forward-subject', { subject: stripIsolates(email.subject ?? '') }),
        t('mail-forward-to', { addr: stripIsolates(addressList(email.to)) }),
        ...((email.cc ?? []).length > 0
          ? [t('mail-forward-cc', { addr: stripIsolates(addressList(email.cc ?? null)) })]
          : []),
      ];
      const quote = outlook ? await quoteForOutlook(shown.source, outlookFields) : await quoteForForward(shown.source, lines);
      const parts = (email as { attachments?: AttachmentPart[] }).attachments ?? [];
      return {
        mode,
        to: '',
        cc: '',
        subject: forwardSubject(email.subject),
        bodyHtml: quote.html,
        bodyText: quote.text,
        inReplyTo: [],
        references: [],
        attachments: parts
          .filter((a) => a.blobId !== null && a.blobId !== undefined && a.blobId !== '')
          .map((a) => ({
            name: a.name !== null && a.name !== undefined && a.name.length > 0 ? a.name : t('mail-attachment-unnamed'),
            blobId: a.blobId as string,
            size: a.size,
            contentType: a.type.length > 0 ? a.type : null,
          })),
        quotesDecrypted: shown.decrypted && quote.html !== '',
        source: { emailId: email.id, keyword: '$forwarded' },
      };
    }

    const headers = await threadingHeaders(email);
    if (headers === null || headers.messageId === null) app.showToast('info', t('mail-reply-unthreaded'), 6000);
    const own = {
      addresses: [...app.identities().map((i) => i.email), ...(app.me() !== null ? [app.me()!.username] : [])],
    };
    const recipients = replyRecipients(email, mode, own);
    const attribution =
      date !== ''
        ? t('mail-quote-attribution', { date, name: authorName })
        : t('mail-quote-attribution-undated', { name: authorName });
    const quote = outlook ? await quoteForOutlook(shown.source, outlookFields) : await quoteForReply(shown.source, attribution);
    return {
      mode,
      to: formatAddressList(recipients.to),
      cc: formatAddressList(recipients.cc),
      subject: replySubject(email.subject),
      bodyHtml: quote.html,
      bodyText: quote.text,
      ...replyThreading(headers),
      attachments: [],
      quotesDecrypted: shown.decrypted && quote.html !== '',
      source: { emailId: email.id, keyword: '$answered' },
    };
  }

  /** Toolbar entry point. One preparation at a time; a failure (the quote's
   *  chunk could not be loaded) is reported and opens nothing. */
  async function startReply(mode: ReplyMode, compact = false): Promise<void> {
    const email = app.openEmail();
    if (email === null || preparing() || props.onCompose === undefined) return;
    setPreparing(true);
    try {
      // The identities decide which addresses a reply-all leaves out.
      if (mode === 'reply-all' && app.identities().length === 0) await app.loadIdentities();
      const initial = await composeFrom(email, mode);
      // The reader moved on while this was prepared: do not open a composer
      // about a message that is no longer the one on screen.
      if (app.openEmail()?.id !== email.id) return;
      props.onCompose(compact ? {...initial, compact: true} : initial);
    } catch {
      app.showToast('error', t('mail-reply-failed'));
    } finally {
      setPreparing(false);
    }
  }

  // The `reply` shortcut (src/keymap/): the same as the toolbar's Reply button.
  useActions({
    replyAll: () => {
      if (props.active === false || app.openEmail() === null || props.onCompose === undefined) return false;
      void startReply('reply-all'); return true;
    },
    forward: () => {
      if (props.active === false || app.openEmail() === null || props.onCompose === undefined) return false;
      void startReply('forward'); return true;
    },
    reply: () => {
      if (props.active === false || app.openEmail() === null || props.onCompose === undefined) return false;
      void startReply('reply');
      return true;
    },
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
  const blockedReport = createMemo(() => { const report = analyzeBlockedContent(currentBody()?.sanitized ?? app.sanitizedHtml()); return { ...report, trackerCount: policy().trackers ? report.trackerCount : 0 }; });
  const fullMode = (): boolean => maxsec.effectiveMode(sender()) === 'full-sanitized';

  const showRemoteBar = (): boolean =>
    fullMode() && (hasBlockedContent(blockedReport()) || activeGrant() !== null);
  /** The notice stack appears with the body (t110), never before it settles. */
  const noticesReady = (): boolean =>
    listHeadersSettled() && (bodySrcdoc() !== null || showDecrypt() || !(app.readLoading() || readerBody.loading || privacy.loading || grants.loading));

  async function reloadOpenMessage(): Promise<void> {
    const id = emailId();
    if (id !== null) await app.openMessage(id);
  }

  // The grant is the user's action; reloading the body afterwards is not. A
  // failed reload must not be reported as "could not update remote image
  // permissions" (the grant was saved), so it is caught on its own (t82).
  async function onRemoteGrant(scope: GrantScope, options?: MutationOptions): Promise<void> {
    const acct = app.accountId();
    if (acct === null) return;
    await remoteImages.grant(acct, scope, options);
    void refetchGrants();
    // Re-fetch + re-sanitize so the now-permitted images load through the proxy.
    await reloadOpenMessage().catch(() => undefined);
  }

  async function onRemoteRevoke(scope: GrantScope, options?: MutationOptions): Promise<void> {
    const acct = app.accountId();
    if (acct === null) return;
    await remoteImages.revoke(acct, scope, options);
    void refetchGrants();
    await reloadOpenMessage().catch(() => undefined);
  }

  // Loading-images toast + the one stats read per displayed body (t55). Only the
  // reader on screen reads or toasts; a message switch drops both.
  const bodyLoadIds = new WeakMap<object, string>(); let bodyLoadSeq = 0;
  const bodyLoadId = (body: object): string => { let id = bodyLoadIds.get(body); if (id === undefined) { id = String(++bodyLoadSeq); bodyLoadIds.set(body, id); } return id; };
  const remoteLoad = createRemoteImageLoad({
    source: () => {
      const body = currentBody(); const account = app.accountId();
      if (props.active === false || !body || account === null || !fullMode()) return null;
      const p = body.policy;
      return { scope: `${account}:${body.emailId}`, emailId: body.emailId, load: bodyLoadId(body), total: body.images, stats: p.trackers && (p.mode === 'proxy' || (p.mode === 'ask' && activeGrant() !== null)) };
    },
    displayed: bodyDisplays,
    toastEnabled: () => effectiveSettings()['privacy.remoteImagesToast']?.value !== false,
    retry: () => void reloadOpenMessage(),
  });
  // The banner keeps the last known mode while the policy refetches, so it does
  // not flash "blocked by your privacy settings" (presentation only: the body
  // still renders under `policy()`).
  // Never `privacy.latest` before the first value: that suspends the mailbox
  // shell, and every open message frame reloads (t60).
  const bannerMode = () => (resourceValue(privacy) ?? policy()).mode;
  // t59: body zoom shortcuts and Ctrl/Cmd + wheel, only while focus is in this reader.
  let readerRoot: HTMLElement | undefined;
  useReaderZoom({ root: () => readerRoot, active: () => props.active !== false });

  return (
    <section
      ref={readerRoot}
      class={`reader ${motion.surface}`}
      data-mw-entering={entering()}
      classList={{ 'reader--open': app.openEmail() !== null }}
      aria-label={t('mail-reader-label')}
    >
      <Show when={app.openEmail()} fallback={<p class="reader__empty">{t('mail-reader-empty')}</p>}>
        {(email) => (
          <>
            <Show when={props.view}>{(view) => <RemoteChangeNotice view={view()} />}</Show>
            <header class="reader__header">
              <h2 ref={subjectHeading} tabIndex={-1} class="reader__subject">{email().subject ?? t('mail-no-subject')}</h2>
              <div class="reader__meta">
                <span>{t('mail-reader-from', { addr: isolate(addressList(email().from)) })}<ReaderSearchMenu email={email()} /></span>
                <span>{t('mail-reader-to', { addr: isolate(addressList(email().to)) })}</span>
                <Show when={(email().cc ?? []).length > 0}>
                  <span>{t('mail-reader-cc', { addr: isolate(addressList(email().cc ?? null)) })}</span>
                </Show>
              </div>
              <div class="reader__metadata-line">
              <MessageTimes sent={email().sentAt} received={email().receivedAt}/>
              {/* Security chip → expandable panel: server verdict merged
                  with the client decrypt/verify result. */}
              <Show when={verdict()}>
                {(v) => {
                  const tid = threadId();
                  return (
                    <SecurityPanel
                      inline
                      verdict={v()}
                      senderAddress={sender()}
                      {...(tid !== undefined ? { threadId: tid } : {})}
                      onSenderControl={onSenderControl}
                    />
                  );
                }}
              </Show>
              <Show when={email().blobId && app.downloadUrl() && app.accountId()}><SpamScore url={buildDownloadUrl(app.downloadUrl()!, {accountId:app.accountId()!,blobId:email().blobId!,name:'message.eml',mime:'message/rfc822'})} onHeaders={collectListHeaders}/></Show>
              </div>
              <Show when={app.accountId()}><DeliveryStatus accountId={app.accountId()!} emailId={email().id} enabled={policy().inlineParts === true} proxyUrl={privacy.state === 'ready' && policy().inlineParts !== true && email().blobId && app.downloadUrl() ? buildDownloadUrl(app.downloadUrl()!, {accountId:app.accountId()!,blobId:email().blobId!,name:'message.eml',mime:'message/rfc822'}) : undefined}/></Show>
              <Show when={app.accountId()&&policy().inlineParts===true}><ReadReceipt accountId={app.accountId()!} emailId={email().id} active={props.active!==false} fullView={props.view!==undefined} displayed={receiptDisplayed()?.key===receiptDocumentKey()&&receiptDisplayed()?.html===bodySrcdoc()&&!showDecrypt()}/></Show>

              {/* One action row (t57): reply group, View conversation (threads of more
                  than one message), Review links (with its count), the message
                  actions and the max-security switch; what does not fit folds into More. */}
              <ReaderToolbar email={email()} onReply={(mode) => void startReply(mode)} preparing={preparing()} onPrint={() => printMessage(email(), displayedContent(email()).source)} active={props.active!==false} links={reviewedBody()?.links??[]} linksTruncated={reviewedBody()?.truncated??false} linkScope={receiptDocumentKey()} {...(props.onCompose?{onCompose:props.onCompose}:{})} security={{get value(){return maxsec.effectiveMode(sender());},get floor(){return maxsec.adminFloor();},onChange:(m)=>maxsec.setSenderMode(sender(),m)}} />
              <Show when={effectiveSettings()['mail.quickReply']?.value === true && props.onCompose}><button type="button" class="btn btn--ghost" disabled={preparing()} title={t('mail-quick-reply-help')} onClick={() => void startReply('reply', true)}><Icon name="reply"/>{t('mail-quick-reply')}</button></Show>
              <AutoTagSection email={email()} />
            </header>
            {/* The reader notice bars (t110, ReaderNoticeBar.tsx), in one stack and a
                fixed order: mailing list, remote images, linked files, attachments
                (last, so its expanded thumbnails sit right above the body). The
                stack and the body appear together, once the body is ready and the
                list headers have settled (or 0.5 s have passed), so no bar pushes
                the body down after it is shown. */}
            <Show when={noticesReady()}>
              <div class={noticeCss.stack} data-testid="reader-notice-stack">
                <MailingListActions headers={listHeaders()?.scope===receiptDocumentKey()&&listHeaders()?.url===messageHeaderUrl()?listHeaders()!.raw:''} scope={receiptDocumentKey()} active={props.active!==false} {...(props.onCompose?{onCompose:props.onCompose}:{})}/>
                <Show when={showRemoteBar()}>
                  <RemoteContentBar
                    emailId={email().id}
                    sender={sender()}
                    report={blockedReport()}
                    activeGrant={activeGrant()}
                    mode={bannerMode()}
                    decodedTrackers={remoteLoad.trackers()}
                    onGrant={onRemoteGrant}
                    onRevoke={onRemoteRevoke}
                    {...(props.onOpenSettings ? { onOpenPrivacySettings: () => props.onOpenSettings?.('privacy') } : {})}
                  />
                </Show>
                <Show when={currentBody()?.refs.links.length}><LinkedFilesBar emailId={email().id} scope={`${app.accountId()}:${email().id}`} urls={currentBody()?.refs.files ?? []} links={currentBody()?.refs.links ?? []} /></Show>
                <AttachmentsPane email={email()} {...(props.onOpenAttachment ? {onOpenAttachment:props.onOpenAttachment} : {})} />
              </div>
            </Show>
            <Show when={hasCalendarPart(email()) && app.accountId()}>{(accountId) => (
              <InviteCard accountId={accountId()} emailId={email().id} />
            )}</Show>
            <RemoteImageDownloads emailId={email().id} scope={`${app.accountId()}:${email().id}`} urls={policy().remoteImageDownloads===true && armor()===null && decryptedHtml()===null && decryptedText()===null ? currentBody()?.refs.urls??[] : []}/>
            <Show when={currentBody()?.failed}><p role="alert">{t('privacy-content-failed')}</p></Show>
            <Show
              when={showDecrypt()}
              fallback={
                <Show
                  when={bodySrcdoc() !== null && listHeadersSettled()}
                  fallback={<Show when={app.readLoading() || readerBody.loading || privacy.loading || grants.loading} fallback={<p class="reader__empty">{t('mail-no-content')}</p>}>{messageLoading()}</Show>}
                >
                  {/*
                    Security-critical (SPEC §7.2): the message body renders in a
                    sandboxed iframe whose sandbox attribute intentionally omits
                    allow-scripts AND allow-same-origin, so even if the sanitizer
                    missed something, scripts cannot run and the frame has an
                    opaque origin with no access to the parent.
                  */}
                  <Show when={!isNativeAuth() && fullMode()} fallback={<ReaderZoomBox><Show when={receiptDocument()} keyed>{doc=><iframe onLoad={() => {if(doc.html!==null)receiptDidDisplay(doc.html,doc.key);}} class="reader__frame" title={t('mail-message-body')} sandbox="" srcdoc={doc.html ?? ''} />}</Show></ReaderZoomBox>}>
                    <DirectReaderFrame html={bodySrcdoc() ?? ''} title={t('mail-message-body')} documentKey={receiptDocumentKey()} onDisplayed={receiptDidDisplay}/>
                  </Show>
                </Show>
              }
            >
              <DecryptPanel
                armor={armor() ?? ''}
                emailId={email().id}
                senderAddress={sender()}
                onDecrypted={(content, signature, decryptedId, decryptedAccount) => {
                  if (emailId() !== decryptedId || app.accountId() !== decryptedAccount) return;
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
