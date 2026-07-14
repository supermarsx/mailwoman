import { createMemo, createSignal, For, Show, onMount, onCleanup, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { t, isolate, loadCatalog } from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import {
  createContactAutocomplete,
  type ContactSuggestion,
} from '../modules/contacts/autocomplete.ts';
import { ComposeCrypto, type ComposeCryptoState } from './compose-crypto.tsx';
import {
  createJmapDlpScan,
  createJmapKeyLookup,
  type DlpScanFn,
  type KeyLookupFn,
} from './compose/crypto-jmap.ts';
import { createConfiguredClient } from '../api/transport.ts';
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
  const [body, setBody] = createSignal('');
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
  // Crypto/DLP state reported up by <ComposeCrypto> (encrypt/sign toggles, the
  // E2EE/TLS/mixed capability, the DLP `canSend` gate, and the WASM-encrypted
  // draft) — plan §2.5, e8 wiring.
  const [cryptoState, setCryptoState] = createSignal<ComposeCryptoState | null>(null);

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
  });

  onCleanup(() => previouslyFocused?.focus());

  const identity = createMemo(() => app.identities().find((i) => i.id === identityId()) ?? null);

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
    setBusy(true);
    try {
      // Encrypt-on-send (plan §2.5): when encryption is on and the worker has
      // produced an encrypted draft (real WASM), send the armored ciphertext as
      // the body so the recipient decrypts it client-side. Protected-subject
      // replaces the visible subject with a placeholder.
      const enc =
        cs !== null && cs.encrypt && cs.encryptedDraft !== null ? cs.encryptedDraft : null;
      const htmlBody =
        enc !== null ? enc.armoredCiphertext : `<p>${escapeHtml(body()).replace(/\n/g, '<br>')}</p>`;
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
          <button type="button" class={`btn btn--ghost ${a11y.iconButton}`} aria-label={t('mail-compose-close')} onClick={() => props.onClose()}>
            ✕
          </button>
        </header>

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
        <label class="field field--grow">
          <span>{t('mail-compose-body')}</span>
          <textarea rows="10" value={body()} onInput={(e) => setBody(e.currentTarget.value)} />
        </label>

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
          </div>
        </Show>

        {/* Crypto + DLP (plan §2.5): encrypt/sign toggles, the live E2EE/TLS/mixed
            banner from real per-recipient CryptoKey/lookup, and the Dlp/scan
            pre-send warnings. Reports state up via onChange for the send path. */}
        <ComposeCrypto
          recipients={() => splitRecipients(to())}
          subject={() => subject()}
          bodyText={() => body()}
          lookupKeys={lookupKeys}
          scanDlp={scanDlp}
          onChange={setCryptoState}
        />

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
