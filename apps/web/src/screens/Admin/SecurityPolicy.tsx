// Admin › Security policy (§19). Min-TLS, 2FA requirement, Argon2id params, DLP
// rules, the max-security floor, and capture policy. Persisted via PUT; audited.

import { createSignal, Show, onMount, type JSX } from 'solid-js';
import { useAdmin } from './context.ts';
import type { SecurityPolicy as Policy } from '../../state/slices/admin.ts';
import * as css from './admin.css.ts';

const EMPTY: Policy = {
  minTls: '1.2',
  require2fa: false,
  argon2MCost: 19_456,
  argon2TCost: 2,
  argon2PCost: 1,
  dlpRulesJson: '[]',
  maxSecurityFloor: false,
  capturePolicy: 'off',
};

export function SecurityPolicy(): JSX.Element {
  const { api } = useAdmin();
  const [policy, setPolicy] = createSignal<Policy>(EMPTY);
  const [error, setError] = createSignal<string | null>(null);
  const [saved, setSaved] = createSignal(false);

  onMount(() => {
    void (async () => {
      try {
        setPolicy(await api.getSecurityPolicy());
      } catch {
        setError('Could not load the security policy');
      }
    })();
  });

  function patch<K extends keyof Policy>(key: K, value: Policy[K]): void {
    setPolicy({ ...policy(), [key]: value });
    setSaved(false);
  }

  async function onSave(e: Event): Promise<void> {
    e.preventDefault();
    try {
      await api.setSecurityPolicy(policy());
      setSaved(true);
      setError(null);
    } catch {
      setError('Could not save the security policy');
    }
  }

  return (
    <section class={css.section} aria-label="Security policy">
      <h2 class={css.heading}>Security policy</h2>
      <Show when={error()}>
        <p class={css.error} role="alert">
          {error()}
        </p>
      </Show>

      <form class={css.card} onSubmit={(e) => void onSave(e)}>
        <div class={css.grid}>
          <label class="field">
            <span>Minimum TLS</span>
            <input type="text" value={policy().minTls} onInput={(e) => patch('minTls', e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Capture policy</span>
            <input type="text" value={policy().capturePolicy} onInput={(e) => patch('capturePolicy', e.currentTarget.value)} />
          </label>
          <label class="field">
            <span>Argon2 memory cost (KiB)</span>
            <input
              type="number"
              value={policy().argon2MCost}
              onInput={(e) => patch('argon2MCost', Number(e.currentTarget.value) || 0)}
            />
          </label>
          <label class="field">
            <span>Argon2 time cost</span>
            <input
              type="number"
              value={policy().argon2TCost}
              onInput={(e) => patch('argon2TCost', Number(e.currentTarget.value) || 0)}
            />
          </label>
          <label class="field">
            <span>Argon2 parallelism</span>
            <input
              type="number"
              value={policy().argon2PCost}
              onInput={(e) => patch('argon2PCost', Number(e.currentTarget.value) || 0)}
            />
          </label>
        </div>
        <label class="field">
          <span>DLP rules (JSON)</span>
          <textarea value={policy().dlpRulesJson} rows={3} onInput={(e) => patch('dlpRulesJson', e.currentTarget.value)} />
        </label>
        <label class="field">
          <input
            type="checkbox"
            checked={policy().require2fa}
            aria-label="Require two-factor authentication"
            onChange={(e) => patch('require2fa', e.currentTarget.checked)}
          />{' '}
          Require 2FA
        </label>
        <label class="field">
          <input
            type="checkbox"
            checked={policy().maxSecurityFloor}
            aria-label="Enforce maximum-security floor"
            onChange={(e) => patch('maxSecurityFloor', e.currentTarget.checked)}
          />{' '}
          Enforce maximum-security floor
        </label>
        <button type="submit" class="btn btn--primary">
          Save policy
        </button>
        <Show when={saved()}>
          <p class={css.note} role="status">
            Saved.
          </p>
        </Show>
      </form>
    </section>
  );
}
