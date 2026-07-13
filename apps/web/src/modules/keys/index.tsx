// Key-management module PLACEHOLDER (plan §2.5, §3 e0 → e2 fills → e8 mounts).
// The full surface (own-key generate/import(PKCS#12, armored)/backup, contact-key
// list with WKD/VKS consent lookup, trust/verify via fingerprint safe-words + QR,
// Autocrypt status, per-contact key association) is built by e2 against the frozen
// `CryptoKey/*` mock + the crypto-worker stub; e8 mounts it into the shell nav
// (under Settings/Security) and swaps to the real engine + wasm worker.
//
// e0 ships this reachable placeholder so the `keys` AppModule registry entry
// resolves + the keys store slice is exercised. It reads the mock-backed key list
// through `useApp()` (the `KeysSlice`, `state/slices/keys.ts`).

import { For, Show, onMount, type JSX } from 'solid-js';
import { useApp } from '../../state/context.ts';
import type { CryptoKey } from '../../api/crypto-types.ts';

/** A short label for a key row (kind + primary address + trust). */
function keyLabel(key: CryptoKey): string {
  const addr = key.addresses[0] ?? key.fingerprint;
  return `${key.kind.toUpperCase()} · ${addr} · ${key.trust}`;
}

export function KeysModule(): JSX.Element {
  const app = useApp();

  onMount(() => {
    void app.loadKeys();
  });

  return (
    <section class="keys" aria-label="Key management" data-module="keys">
      <header class="keys__head">
        <h1 class="keys__title">Keys &amp; certificates</h1>
        <p class="keys__subtitle">
          OpenPGP and S/MIME keys. Private keys stay on this device and never reach the server.
        </p>
      </header>

      <Show
        when={app.keys().length > 0}
        fallback={
          <p class="keys__empty">
            {app.keysLoading() ? 'Loading keys…' : 'No keys yet. Key management arrives with the full module (e2).'}
          </p>
        }
      >
        <ul class="keys__list" aria-label="Keys">
          <For each={app.keys()}>
            {(key) => (
              <li class="keys__item" classList={{ 'keys__item--own': key.isOwn }}>
                <span class="keys__item-label">{keyLabel(key)}</span>
                <span class="keys__item-fpr" aria-label="Fingerprint">
                  {key.fingerprint}
                </span>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </section>
  );
}
