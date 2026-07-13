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

import { For, Show, createMemo, createSignal, onMount, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import { contactDisplayName } from '../../state/slices/contacts.ts';
import type { CryptoKey, KeyKind, KeyTrust } from '../../api/crypto-types.ts';
import type { Id } from '../../api/jmap-types.ts';
import type { ImportPreview, KeyLookupSource } from '../../state/slices/keys.ts';
import { fingerprintWords, groupFingerprint } from './proquint.ts';
import { encodeQr, qrToSvg } from './qr.ts';
import * as css from './keys.css.ts';

const TRUST_OPTIONS: KeyTrust[] = ['unverified', 'tofu', 'verified', 'revoked'];
const LOOKUP_SOURCES: KeyLookupSource[] = ['wkd', 'vks', 'autocrypt', 'harvested'];

/** A short one-line label for a key row. */
function keyTitle(key: CryptoKey): string {
  return key.addresses[0] ?? `${key.kind.toUpperCase()} key`;
}

export function KeysModule(): JSX.Element {
  const app = useApp();
  const [selectedId, setSelectedId] = createSignal<Id | null>(null);
  const [showGenerate, setShowGenerate] = createSignal(false);
  const [showImport, setShowImport] = createSignal(false);

  onMount(() => {
    void app.loadKeys();
    void app.loadContacts();
  });

  const selected = createMemo(() => app.keys().find((k) => k.id === selectedId()) ?? null);

  function select(id: Id): void {
    setSelectedId(id);
  }

  return (
    <section aria-label="Key management" data-module="keys" class={css.layout}>
      <div class={css.listPane}>
        <header class={css.head}>
          <h1 class={css.title}>Keys &amp; certificates</h1>
          <p class={css.subtitle}>
            OpenPGP and S/MIME keys. Private keys stay on this device and never reach the server.
          </p>
        </header>

        <div class={css.toolbar}>
          <button type="button" class={css.button} onClick={() => setShowGenerate(true)}>
            Generate key
          </button>
          <button type="button" class={css.buttonGhost} onClick={() => setShowImport(true)}>
            Import key
          </button>
        </div>

        <KeyGroup
          heading="Your keys"
          keys={app.ownKeys()}
          loading={app.keysLoading()}
          emptyText="No keys yet. Generate or import one to start."
          selectedId={selectedId()}
          onSelect={select}
        />

        <KeyGroup
          heading="Contact keys"
          keys={app.contactKeys()}
          loading={false}
          emptyText="No contact keys. Look one up below."
          selectedId={selectedId()}
          onSelect={select}
        />

        <LookupForm />
      </div>

      <div class={css.detail}>
        <Show when={selected()} fallback={<p class={css.empty}>Select a key to view and verify it.</p>}>
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
        fallback={<p class={css.empty}>{props.loading ? 'Loading keys…' : props.emptyText}</p>}
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
  return (
    <span class={cls()} aria-label={`${props.kind.toUpperCase()} ${props.trust}`}>
      {props.kind.toUpperCase()} · {props.trust}
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
      app.showToast('error', 'No private key held on this device for backup');
    }
  }

  async function onAssociate(): Promise<void> {
    const cid = associateContact();
    if (cid === '') return;
    await app.associateKeyWithContact(cid, key());
  }

  return (
    <article class={css.card} aria-label={`Key ${keyTitle(key())}`}>
      <div>
        <h2 class={css.cardName}>{keyTitle(key())}</h2>
        <p class={css.cardSub}>
          {key().kind.toUpperCase()} · {key().algorithm} · {key().source}
        </p>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>Fingerprint</span>
        <p class={css.fingerprint} aria-label="Fingerprint">
          {groupFingerprint(key().fingerprint)}
        </p>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>Safe words</span>
        <p class={css.cardSub}>Read these aloud with the contact to confirm the key out-of-band.</p>
        <ul class={css.words} aria-label="Fingerprint safe words">
          <For each={words()}>{(w) => <li class={css.word}>{w}</li>}</For>
        </ul>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>Scan to verify</span>
        {/* QR SVG is generated in-module from the fingerprint — never user HTML. */}
        <div class={css.qr} role="img" aria-label="Fingerprint QR code" innerHTML={qrSvg()} />
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>Autocrypt</span>
        <p class={css.cardSub}>{key().autocrypt ? 'Advertised in Autocrypt headers' : 'Not advertised via Autocrypt'}</p>
      </div>

      <div class={css.fieldGroup}>
        <label class={css.fieldLabel} for={`trust-${key().id}`}>
          Trust
        </label>
        <select
          id={`trust-${key().id}`}
          class={css.select}
          aria-label="Trust level"
          value={key().trust}
          onChange={(e) => void app.setKeyTrust(key().id, e.currentTarget.value as KeyTrust)}
        >
          <For each={TRUST_OPTIONS}>{(t) => <option value={t}>{t}</option>}</For>
        </select>
      </div>

      <div class={css.fieldGroup}>
        <span class={css.fieldLabel}>Associate with a contact</span>
        <div class={css.fieldRow}>
          <select
            class={css.select}
            aria-label="Contact to associate"
            value={associateContact()}
            onChange={(e) => setAssociateContact(e.currentTarget.value)}
          >
            <option value="">Choose a contact…</option>
            <For each={app.contacts()}>{(c) => <option value={c.id}>{contactDisplayName(c)}</option>}</For>
          </select>
          <button type="button" class={css.button} disabled={associateContact() === ''} onClick={() => void onAssociate()}>
            Associate
          </button>
        </div>
      </div>

      <Show when={canBackup()}>
        <div class={css.fieldGroup}>
          <span class={css.fieldLabel}>Backup</span>
          <p class={css.cardSub}>Export an Autocrypt Setup Message to move this key to another device.</p>
          <div class={css.actions}>
            <button type="button" class={css.buttonGhost} onClick={() => void onExportBackup()}>
              Export backup
            </button>
          </div>
          <Show when={backup()}>
            {(msg) => (
              <textarea class={css.textarea} readOnly aria-label="Autocrypt Setup Message" value={msg()} />
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
      setNote(found.length > 0 ? `Found ${found.length} key(s) — added to Contact keys.` : 'No key found.');
    } finally {
      setBusy(false);
    }
  }

  return (
    <form
      aria-label="Look up a contact key"
      onSubmit={(e) => {
        e.preventDefault();
        if (canLookup()) void onLookup();
      }}
    >
      <h2 class={css.heading}>Look up a key</h2>
      <div class={css.fieldStack}>
        <input
          type="email"
          class={css.input}
          aria-label="Address to look up"
          placeholder="name@example.org"
          value={address()}
          onInput={(e) => setAddress(e.currentTarget.value)}
        />
        <div class={css.fieldRow} role="group" aria-label="Lookup sources">
          <For each={LOOKUP_SOURCES}>
            {(s) => (
              <label class={css.label} style={{ 'flex-direction': 'row', 'align-items': 'center' }}>
                <input
                  type="checkbox"
                  checked={sources().has(s)}
                  onChange={(e) => toggleSource(s, e.currentTarget.checked)}
                  aria-label={`Source ${s}`}
                />
                {s.toUpperCase()}
              </label>
            )}
          </For>
        </div>
        <label class={css.consent}>
          <input
            type="checkbox"
            checked={consent()}
            onChange={(e) => setConsent(e.currentTarget.checked)}
            aria-label="Consent to external lookup"
          />
          <span>
            Looking a key up contacts external directories (WKD/VKS). I have this person's consent to do so.
          </span>
        </label>
        <div class={css.actions}>
          <button type="submit" class={css.button} disabled={!canLookup()}>
            {busy() ? 'Looking up…' : 'Look up'}
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
    <div class={css.dialogBackdrop} onClick={props.onClose}>
      <div class={css.dialog} role="dialog" aria-modal="true" aria-label="Generate a key" onClick={(e) => e.stopPropagation()}>
        <h2 class={css.title}>Generate a key</h2>
        <div class={css.fieldStack}>
          <label class={css.label}>
            Type
            <select class={css.select} aria-label="Key type" value={kind()} onChange={(e) => setKind(e.currentTarget.value as KeyKind)}>
              <option value="pgp">OpenPGP</option>
              <option value="smime">S/MIME</option>
            </select>
          </label>
          <label class={css.label}>
            Name
            <input class={css.input} aria-label="Name" value={name()} onInput={(e) => setName(e.currentTarget.value)} />
          </label>
          <label class={css.label}>
            Email
            <input type="email" class={css.input} aria-label="Email" value={email()} onInput={(e) => setEmail(e.currentTarget.value)} />
          </label>
          <label class={css.label}>
            Key passphrase
            <input
              type="password"
              class={css.input}
              aria-label="Key passphrase"
              value={passphrase()}
              onInput={(e) => setPassphrase(e.currentTarget.value)}
            />
          </label>
          <p class={css.cardSub}>The passphrase wraps the private key on this device. It never leaves the browser.</p>
        </div>
        <div class={css.actions}>
          <button type="button" class={css.buttonGhost} onClick={props.onClose}>
            Cancel
          </button>
          <button type="button" class={css.button} disabled={!canGenerate()} onClick={() => void onGenerate()}>
            {busy() ? 'Generating…' : 'Generate'}
          </button>
        </div>
      </div>
    </div>
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
    <div class={css.dialogBackdrop} onClick={props.onClose}>
      <div class={css.dialog} role="dialog" aria-modal="true" aria-label="Import a key" onClick={(e) => e.stopPropagation()}>
        <h2 class={css.title}>Import a key</h2>
        <div class={css.toolbar} role="tablist" aria-label="Import type">
          <button
            type="button"
            role="tab"
            aria-selected={mode() === 'armored'}
            class={mode() === 'armored' ? css.button : css.buttonGhost}
            onClick={() => { setMode('armored'); setPreview(null); }}
          >
            Armored (PGP)
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={mode() === 'pkcs12'}
            class={mode() === 'pkcs12' ? css.button : css.buttonGhost}
            onClick={() => { setMode('pkcs12'); setPreview(null); }}
          >
            PKCS#12 (S/MIME)
          </button>
        </div>

        <Show when={mode() === 'armored'}>
          <div class={css.fieldStack}>
            <label class={css.label}>
              Armored key
              <textarea
                class={css.textarea}
                aria-label="Armored key"
                placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
                value={armored()}
                onInput={(e) => setArmored(e.currentTarget.value)}
              />
            </label>
            <label class={css.label}>
              Passphrase (if the key is private)
              <input
                type="password"
                class={css.input}
                aria-label="Import passphrase"
                value={passphrase()}
                onInput={(e) => setPassphrase(e.currentTarget.value)}
              />
            </label>
          </div>
        </Show>

        <Show when={mode() === 'pkcs12'}>
          <div class={css.fieldStack}>
            <label class={css.label}>
              PKCS#12 file (.p12 / .pfx)
              <input
                type="file"
                accept=".p12,.pfx"
                aria-label="PKCS#12 file"
                onChange={(e) => void onFile(e.currentTarget.files?.[0])}
              />
            </label>
            <label class={css.label}>
              Import password
              <input
                type="password"
                class={css.input}
                aria-label="PKCS#12 password"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
              />
            </label>
          </div>
        </Show>

        <Show when={preview()}>
          {(p) => (
            <div class={css.preview} role="group" aria-label="Import preview">
              <strong>Preview</strong>
              <span>Type: {p().key.kind.toUpperCase()}</span>
              <span aria-label="Preview fingerprint">Fingerprint: {groupFingerprint(p().key.fingerprint)}</span>
              <span>
                {p().encryptedPrivateBundle !== null ? 'Includes a private key (will be stored on this device)' : 'Public key only'}
              </span>
            </div>
          )}
        </Show>

        <div class={css.actions}>
          <button type="button" class={css.buttonGhost} onClick={props.onClose}>
            Cancel
          </button>
          <button type="button" class={css.buttonGhost} disabled={busy()} onClick={() => void onPreview()}>
            Preview
          </button>
          <button type="button" class={css.button} disabled={preview() === null || busy()} onClick={() => void onConfirm()}>
            Import
          </button>
        </div>
      </div>
    </div>
  );
}
