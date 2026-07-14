// Key-management module (plan §2.5, §3 e2). The Settings/Security surface for
// OpenPGP + S/MIME keys: own-key generation and import (armored + PKCS#12) with a
// preview step, an Autocrypt-Setup-Message backup, the contact/harvested key list
// with consent-gated WKD/VKS/harvest lookup, trust/verify (fingerprint safe-words
// + a scannable QR), Autocrypt status, and per-contact key association (writing the
// V3 `ContactCard.pgpKey`/`smimeCert` fields). Mock-backed via the keys store slice
// + the crypto-worker STUB until e8 swaps in the real engine + wasm worker.
//
// Private keys never enter this component's state: generation/import happens in the
// crypto worker and the wrapped bundle lives in the client vault (plan §1.2).

import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { contactDisplayName } from '../../state/slices/contacts.ts';
import type { CryptoKey, KeyKind, KeyTrust } from '../../api/crypto-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import type { ImportPreview, KeyLookupSource } from '../../state/slices/keys.ts';
import { t, loadCatalog, isolate } from '../../i18n';
import { fingerprintWords, groupFingerprint } from './proquint.ts';
import { encodeQr, qrToSvg } from './qr.ts';
import * as css from './keys.css.ts';

const TRUST_OPTIONS: KeyTrust[] = ['unverified', 'tofu', 'verified', 'revoked'];
const LOOKUP_SOURCES: KeyLookupSource[] = ['wkd', 'vks', 'autocrypt', 'harvested'];

/** A short one-line label for a key row. */
function keyTitle(key: CryptoKey): string {
  return key.addresses[0] ?? t('keys-untitled', { kind: key.kind.toUpperCase() });
}

export function KeysModule(): JSX.Element {
  const app = useApp();
  const [selectedId, setSelectedId] = createSignal<Id | null>(null);
  const [showGenerate, setShowGenerate] = createSignal(false);
  const [showImport, setShowImport] = createSignal(false);

  onMount(() => {
    void loadCatalog('keys');
    void app.loadKeys();
    void app.loadContacts();
  });

  const selected = createMemo(() => app.keys().find((k) => k.id === selectedId()) ?? null);

  function select(id: Id): void {
    setSelectedId(id);
  }

  return (
    <section aria-label={t('keys-panel-label')} data-module="keys" class={css.layout}>
      <div class={css.listPane}>
        <header class={css.head}>
          <h1 class={css.title}>{t('keys-title')}</h1>
          <p class={css.subtitle}>{t('keys-subtitle')}</p>
        </header>

        <div class={css.toolbar}>
          <button type="button" class={css.button} onClick={() => setShowGenerate(true)}>
            {t('keys-generate')}
          </button>
          <button type="button" class={css.buttonGhost} onClick={() => setShowImport(true)}>
            {t('keys-import')}
          </button>
        </div>

        <KeyGroup
          heading={t('keys-your-keys')}
          keys={app.ownKeys()}
          loading={app.keysLoading()}
          emptyText={t('keys-empty-own')}
          selectedId={selectedId()}
          onSelect={select}
        />

        <KeyGroup
          heading={t('keys-contact-keys')}
          keys={app.contactKeys()}
          loading={false}
          emptyText={t('keys-empty-contact')}
          selectedId={selectedId()}
          onSelect={select}
        />

        <LookupForm />
      </div>

      <div class={css.detail}>
        <Show when={selected()} fallback={<p class={css.empty}>{t('keys-select-prompt')}</p>}>
          {(key) => <KeyDetail key={key()} />}
        </Show>
      </div>

      <Show when={showGenerate()}>
        <GenerateDialog
          onClose={() => setShowGenerate(false)}
          onGenerated={(id) => {
            setShowGenerate(false);
            setSelectedId(id);
          }}
        />
      </Show>
      <Show when={showImport()}>
        <ImportDialog
          onClose={() => setShowImport(false)}
          onImported={(id) => {
            setShowImport(false);
            setSelectedId(id);
          }}
        />
      </Show>
    </section>
  );
}

// ── Accessible modal shell (WCAG 2.2 — self-contained, no shared a11y import) ──
// Wraps dialog content with role="dialog" + aria-modal, moves focus in on open,
// traps Tab within, closes on Escape, and restores focus to the trigger on close.

/** Tab-order-visible focusable descendants of `root` (excludes disabled/hidden). */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  const sel =
    'a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex="-1"])';
  return [...root.querySelectorAll<HTMLElement>(sel)];
}

function Dialog(props: { label: string; onClose: () => void; children: JSX.Element }): JSX.Element {
  let dialogRef: HTMLDivElement | undefined;
  // The element focused when the dialog opened (the trigger) — restored on close.
  const trigger = document.activeElement as HTMLElement | null;

  onMount(() => {
    const focusables = focusableWithin(dialogRef!);
    (focusables[0] ?? dialogRef!).focus();
  });
  onCleanup(() => {
    trigger?.focus();
  });

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      props.onClose();
      return;
    }
    if (e.key !== 'Tab' || dialogRef === undefined) return;
    const focusables = focusableWithin(dialogRef);
    if (focusables.length === 0) {
      e.preventDefault();
      return;
    }
    const first = focusables[0]!;
    const last = focusables[focusables.length - 1]!;
    const active = document.activeElement;
    if (e.shiftKey && active === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && active === last) {
      e.preventDefault();
      first.focus();
    }
  }

  return (
    <div class={css.dialogBackdrop} onClick={props.onClose}>
      <div
        ref={dialogRef}
        class={css.dialog}
        role="dialog"
        aria-modal="true"
        aria-label={props.label}
        tabindex={-1}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        {props.children}
      </div>
    </div>
  );
}

// ── Key list group ────────────────────────────────────────────────────────────

function KeyGroup(props: {
  heading: string;
  keys: CryptoKey[];
  loading: boolean;
  emptyText: string;
  selectedId: Id | null;
  onSelect: (id: Id) => void;
}): JSX.Element {
  return (
    <div>
      <h2 class={css.heading}>{props.heading}</h2>
      <Show
        when={props.keys.length > 0}
        fallback={<p class={css.empty}>{props.loading ? t('keys-loading') : props.emptyText}</p>}
      >
        <ul class={css.keyList} aria-label={props.heading}>
          <For each={props.keys}>
            {(key) => (
              <li>
                <button
                  type="button"
                  class={css.keyRow}
                  aria-current={props.selectedId === key.id ? 'true' : undefined}
                  onClick={() => props.onSelect(key.id)}
                >
                  <span class={css.rowBody}>
                    <span class={css.rowName}>{keyTitle(key)}</span>
                    <span class={css.rowMeta}>{groupFingerprint(key.fingerprint)}</span>
                  </span>
                  <TrustBadge trust={key.trust} kind={key.kind} />
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </div>
  );
}

function TrustBadge(props: { trust: KeyTrust; kind: KeyKind }): JSX.Element {
  const cls = createMemo(() => {
    if (props.trust === 'verified') return `${css.badge} ${css.badgeVerified}`;
    if (props.trust === 'revoked') return `${css.badge} ${css.badgeRevoked}`;
    return css.badge;
  });
  // A text label always accompanies the colour (WCAG 1.4.1 — never colour-only).
  const trustLabel = createMemo(() => t(`keys-trust-${props.trust}`));
  return (
    <span class={cls()} aria-label={`${props.kind.toUpperCase()} ${trustLabel()}`}>
      {props.kind.toUpperCase()} · {trustLabel()}
    </span>
  );
}

// ── Key detail / trust / verify ───────────────────────────────────────────────

function KeyDetail(props: { key: CryptoKey }): JSX.Element {
  const app = useApp();
  const key = (): CryptoKey => props.key;
  const words = createMemo(() => fingerprintWords(key().fingerprint));
  const qrSvg = createMemo(() => qrToSvg(encodeQr(key().fingerprint)));
  const [backup, setBackup] = createSignal<string | null>(null);
  const [associateContact, setAssociateContact] = createSignal<Id>('');

  const canBackup = createMemo(() => key().isOwn && app.hasVaultedKey(key().fingerprint));

  async function onExportBackup(): Promise<void> {
    try {
      setBackup(await app.exportKeyBackup(key().fingerprint));
    } catch {
      app.showToast('error', t('keys-no-private-backup'));
    }
  }

  async function onAssociate(): Promise<void> {
    const cid = associateContact();
    if (cid === '') return;
    await app.associateKeyWithContact(cid, key());
  }

  return (
    <article class={css.card} aria-label={t('keys-key-card', { name: isolate(keyTitle(key())) })}>
      <div>
        <h2 class={css.cardName}>{keyTitle(key())}</h2>
        <p class={css.cardSub}>
          {key().kind.toUpperCase()} · {key().algorithm} · {key().source}
        </p>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('keys-fingerprint')}</span>
        <p class={css.fingerprint} aria-label={t('keys-fingerprint')}>
          {groupFingerprint(key().fingerprint)}
        </p>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('keys-safe-words')}</span>
        <p class={css.cardSub}>{t('keys-safe-words-help')}</p>
        <ul class={css.words} aria-label={t('keys-safe-words-list')}>
          <For each={words()}>{(w) => <li class={css.word}>{w}</li>}</For>
        </ul>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('keys-scan-label')}</span>
        {/* QR SVG is generated in-module from the fingerprint — never user HTML. */}
        <div class={css.qr} role="img" aria-label={t('keys-qr-label')} innerHTML={qrSvg()} />
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('keys-autocrypt')}</span>
        <p class={css.cardSub}>{key().autocrypt ? t('keys-autocrypt-on') : t('keys-autocrypt-off')}</p>
      </div>

      <div class={css.fieldGroup}>
        <label class={css.fieldLabel} for={`trust-${key().id}`}>
          {t('keys-trust-label')}
        </label>
        <select
          id={`trust-${key().id}`}
          class={css.select}
          aria-label={t('keys-trust-level')}
          value={key().trust}
          onChange={(e) => void app.setKeyTrust(key().id, e.currentTarget.value as KeyTrust)}
        >
          <For each={TRUST_OPTIONS}>{(opt) => <option value={opt}>{t(`keys-trust-${opt}`)}</option>}</For>
        </select>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>{t('keys-associate-label')}</span>
        <div class={css.fieldRow}>
          <select
            class={css.select}
            aria-label={t('keys-associate-select')}
            value={associateContact()}
            onChange={(e) => setAssociateContact(e.currentTarget.value)}
          >
            <option value="">{t('keys-choose-contact')}</option>
            <For each={app.contacts()}>{(c) => <option value={c.id}>{contactDisplayName(c)}</option>}</For>
          </select>
          <button type="button" class={css.button} disabled={associateContact() === ''} onClick={() => void onAssociate()}>
            {t('keys-associate-button')}
          </button>
        </div>
      </div>

      <Show when={canBackup()}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>{t('keys-backup-label')}</span>
          <p class={css.cardSub}>{t('keys-backup-help')}</p>
          <div class={css.actions}>
            <button type="button" class={css.buttonGhost} onClick={() => void onExportBackup()}>
              {t('keys-export-backup')}
            </button>
          </div>
          <Show when={backup()}>
            {(msg) => (
              <textarea class={css.textarea} readOnly aria-label={t('keys-asm-label')} value={msg()} />
            )}
          </Show>
        </div>
      </Show>
    </article>
  );
}

// ── Consent-gated lookup ──────────────────────────────────────────────────────

function LookupForm(): JSX.Element {
  const app = useApp();
  const [address, setAddress] = createSignal('');
  const [sources, setSources] = createSignal<Set<KeyLookupSource>>(new Set(['wkd', 'vks']));
  const [consent, setConsent] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [note, setNote] = createSignal<string | null>(null);

  function toggleSource(s: KeyLookupSource, on: boolean): void {
    setSources((cur) => {
      const next = new Set(cur);
      if (on) next.add(s);
      else next.delete(s);
      return next;
    });
  }

  const canLookup = createMemo(() => address().trim() !== '' && consent() && sources().size > 0 && !busy());

  async function onLookup(): Promise<void> {
    setBusy(true);
    setNote(null);
    try {
      const found = await app.lookupContactKey(address().trim(), [...sources()]);
      setNote(found.length > 0 ? t('keys-lookup-found', { count: found.length }) : t('keys-lookup-none'));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form
      aria-label={t('keys-lookup-form')}
      onSubmit={(e) => {
        e.preventDefault();
        if (canLookup()) void onLookup();
      }}
    >
      <h2 class={css.heading}>{t('keys-lookup-heading')}</h2>
      <div class={css.fieldStack}>
        <input
          type="email"
          class={css.input}
          aria-label={t('keys-lookup-address')}
          placeholder={t('keys-lookup-placeholder')}
          value={address()}
          onInput={(e) => setAddress(e.currentTarget.value)}
        />
        <div class={css.fieldRow} role="group" aria-label={t('keys-lookup-sources')}>
          <For each={LOOKUP_SOURCES}>
            {(s) => (
              <label class={css.label} style={{ 'flex-direction': 'row', 'align-items': 'center' }}>
                <input
                  type="checkbox"
                  class={css.checkbox}
                  checked={sources().has(s)}
                  onChange={(e) => toggleSource(s, e.currentTarget.checked)}
                  aria-label={t('keys-source', { source: s })}
                />
                {s.toUpperCase()}
              </label>
            )}
          </For>
        </div>
        <label class={css.consent}>
          <input
            type="checkbox"
            class={css.checkbox}
            checked={consent()}
            onChange={(e) => setConsent(e.currentTarget.checked)}
            aria-label={t('keys-consent-label')}
          />
          <span>{t('keys-consent-text')}</span>
        </label>
        <div class={css.actions}>
          <button type="submit" class={css.button} disabled={!canLookup()}>
            {busy() ? t('keys-looking-up') : t('keys-lookup-button')}
          </button>
        </div>
        <Show when={note()}>{(n) => <p class={css.cardSub} role="status">{n()}</p>}</Show>
      </div>
    </form>
  );
}

// ── Generate dialog ───────────────────────────────────────────────────────────

function GenerateDialog(props: { onClose: () => void; onGenerated: (id: Id) => void }): JSX.Element {
  const app = useApp();
  const [kind, setKind] = createSignal<KeyKind>('pgp');
  const [name, setName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [passphrase, setPassphrase] = createSignal('');
  const [busy, setBusy] = createSignal(false);

  const canGenerate = createMemo(() => email().trim() !== '' && passphrase() !== '' && !busy());

  async function onGenerate(): Promise<void> {
    setBusy(true);
    try {
      const userId = name().trim() === '' ? email().trim() : `${name().trim()} <${email().trim()}>`;
      const key = await app.generateOwnKey({ kind: kind(), userId, passphrase: passphrase() });
      props.onGenerated(key.id);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog label={t('keys-generate-title')} onClose={props.onClose}>
      <h2 class={css.title}>{t('keys-generate-title')}</h2>
      <div class={css.fieldStack}>
        <label class={css.label}>
          {t('keys-type')}
          <select class={css.select} aria-label={t('keys-key-type')} value={kind()} onChange={(e) => setKind(e.currentTarget.value as KeyKind)}>
            <option value="pgp">{t('keys-openpgp')}</option>
            <option value="smime">{t('keys-smime')}</option>
          </select>
        </label>
        <label class={css.label}>
          {t('keys-name')}
          <input class={css.input} aria-label={t('keys-name')} value={name()} onInput={(e) => setName(e.currentTarget.value)} />
        </label>
        <label class={css.label}>
          {t('keys-email')}
          <input type="email" class={css.input} aria-label={t('keys-email')} value={email()} onInput={(e) => setEmail(e.currentTarget.value)} />
        </label>
        <label class={css.label}>
          {t('keys-key-passphrase')}
          <input
            type="password"
            class={css.input}
            aria-label={t('keys-key-passphrase')}
            value={passphrase()}
            onInput={(e) => setPassphrase(e.currentTarget.value)}
          />
        </label>
        <p class={css.cardSub}>{t('keys-passphrase-help')}</p>
      </div>
      <div class={css.actions}>
        <button type="button" class={css.buttonGhost} onClick={props.onClose}>
          {t('keys-cancel')}
        </button>
        <button type="button" class={css.button} disabled={!canGenerate()} onClick={() => void onGenerate()}>
          {busy() ? t('keys-generating') : t('keys-generate-submit')}
        </button>
      </div>
    </Dialog>
  );
}

// ── Import dialog (armored + PKCS#12) with a preview step ─────────────────────

function ImportDialog(props: { onClose: () => void; onImported: (id: Id) => void }): JSX.Element {
  const app = useApp();
  const [mode, setMode] = createSignal<'armored' | 'pkcs12'>('armored');
  const [armored, setArmored] = createSignal('');
  const [passphrase, setPassphrase] = createSignal('');
  const [p12, setP12] = createSignal<Uint8Array | null>(null);
  const [password, setPassword] = createSignal('');
  const [preview, setPreview] = createSignal<ImportPreview | null>(null);
  const [busy, setBusy] = createSignal(false);

  async function onPreview(): Promise<void> {
    setBusy(true);
    try {
      if (mode() === 'armored') {
        setPreview(await app.previewArmoredKey(armored(), passphrase() === '' ? undefined : passphrase()));
      } else {
        const bytes = p12();
        if (bytes === null) return;
        setPreview(await app.previewPkcs12Key(bytes, password()));
      }
    } finally {
      setBusy(false);
    }
  }

  async function onConfirm(): Promise<void> {
    const p = preview();
    if (p === null) return;
    setBusy(true);
    try {
      const key = await app.commitImport(p);
      props.onImported(key.id);
    } finally {
      setBusy(false);
    }
  }

  async function onFile(file: File | undefined): Promise<void> {
    if (file === undefined) return;
    setP12(new Uint8Array(await file.arrayBuffer()));
  }

  return (
    <Dialog label={t('keys-import-title')} onClose={props.onClose}>
      <h2 class={css.title}>{t('keys-import-title')}</h2>
      <div class={css.toolbar} role="tablist" aria-label={t('keys-import-type')}>
        <button
          type="button"
          role="tab"
          aria-selected={mode() === 'armored'}
          class={mode() === 'armored' ? css.button : css.buttonGhost}
          onClick={() => { setMode('armored'); setPreview(null); }}
        >
          {t('keys-tab-armored')}
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={mode() === 'pkcs12'}
          class={mode() === 'pkcs12' ? css.button : css.buttonGhost}
          onClick={() => { setMode('pkcs12'); setPreview(null); }}
        >
          {t('keys-tab-pkcs12')}
        </button>
      </div>

      <Show when={mode() === 'armored'}>
        <div class={css.fieldStack}>
          <label class={css.label}>
            {t('keys-armored-key')}
            <textarea
              class={css.textarea}
              aria-label={t('keys-armored-key')}
              placeholder={t('keys-armored-placeholder')}
              value={armored()}
              onInput={(e) => setArmored(e.currentTarget.value)}
            />
          </label>
          <label class={css.label}>
            {t('keys-armored-passphrase-label')}
            <input
              type="password"
              class={css.input}
              aria-label={t('keys-import-passphrase')}
              value={passphrase()}
              onInput={(e) => setPassphrase(e.currentTarget.value)}
            />
          </label>
        </div>
      </Show>

      <Show when={mode() === 'pkcs12'}>
        <div class={css.fieldStack}>
          <label class={css.label}>
            {t('keys-pkcs12-file-label')}
            <input
              type="file"
              accept=".p12,.pfx"
              aria-label={t('keys-pkcs12-file')}
              onChange={(e) => void onFile(e.currentTarget.files?.[0])}
            />
          </label>
          <label class={css.label}>
            {t('keys-import-password-label')}
            <input
              type="password"
              class={css.input}
              aria-label={t('keys-pkcs12-password')}
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
            />
          </label>
        </div>
      </Show>

      <Show when={preview()}>
        {(p) => (
          <div class={css.preview} role="group" aria-label={t('keys-import-preview')}>
            <strong>{t('keys-preview')}</strong>
            <span>{t('keys-preview-type', { kind: p().key.kind.toUpperCase() })}</span>
            <span aria-label={t('keys-preview-fingerprint-aria')}>
              {t('keys-preview-fingerprint', { fp: isolate(groupFingerprint(p().key.fingerprint)) })}
            </span>
            <span>
              {p().encryptedPrivateBundle !== null ? t('keys-preview-has-private') : t('keys-preview-public-only')}
            </span>
          </div>
        )}
      </Show>

      <div class={css.actions}>
        <button type="button" class={css.buttonGhost} onClick={props.onClose}>
          {t('keys-cancel')}
        </button>
        <button type="button" class={css.buttonGhost} disabled={busy()} onClick={() => void onPreview()}>
          {t('keys-preview')}
        </button>
        <button type="button" class={css.button} disabled={preview() === null || busy()} onClick={() => void onConfirm()}>
          {t('keys-import-submit')}
        </button>
      </div>
    </Dialog>
  );
}
