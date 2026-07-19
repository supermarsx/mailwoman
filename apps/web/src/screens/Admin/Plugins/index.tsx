// Admin → Plugins screen (SPEC §22, plan §2.6 / §3 e7): the plugin-registry admin UI —
// approve / enable / disable, per-plugin capability + `allow_unsigned` policy, and the
// SIGNED-vs-UNSIGNED banner. An unsigned plugin can only run when the admin explicitly
// opts it in, and while any unsigned plugin is ENABLED a PERMANENT (non-dismissible)
// banner is shown (§22 / §7.5 — the sandbox trusts the signature; unsigned code is a
// standing risk the admin owns).
//
// This screen does NOT touch the router or a shared Admin index (ownership boundary —
// e14 mounts it under /admin). It takes an injected `PluginsSlice` so it is unit-testable
// and reused by whatever admin shell e14 builds.

import { createEffect, For, onMount, Show, type JSX } from 'solid-js';
import {
  createPluginsSlice,
  createHttpPluginsApi,
  type PluginsApi,
  type PluginsSlice,
  type PluginInfo,
} from '../../../state/slices/plugins.ts';
import { t, loadCatalog } from '../../../i18n';
import { AllowlistPanel } from './Allowlist.tsx';
import * as css from './styles.css.ts';

export interface AdminPluginsProps {
  /** Tests / e14 inject a slice or a client; production defaults to the HTTP client. */
  slice?: PluginsSlice;
  api?: PluginsApi;
}

/** The permanent banner shown while any unsigned plugin is enabled (frozen copy). */
export const UNSIGNED_BANNER =
  'One or more enabled plugins are unsigned. Unsigned plugins run only because an ' +
  'administrator allowed them; their code is not verified against a signature. Review them below.';

export function AdminPlugins(props: AdminPluginsProps): JSX.Element {
  const slice = props.slice ?? createPluginsSlice(props.api ?? createHttpPluginsApi());

  onMount(() => void loadCatalog('admin'));
  // Load once on mount (idempotent; e14 may also pre-load).
  createEffect(() => {
    void slice.load();
  });

  return (
    <section class={css.screen} data-screen="admin-plugins" aria-label={t('admin-plugins-title')}>
      <div>
        <h2 class={css.heading}>{t('admin-plugins-title')}</h2>
        <p class={css.prose}>{t('admin-plugins-intro')}</p>
      </div>

      <Show when={slice.hasUnsignedEnabled()}>
        <p class={css.unsignedBanner} role="alert" data-testid="unsigned-banner">
          {UNSIGNED_BANNER}
        </p>
      </Show>

      <Show when={!slice.loading() && slice.plugins().length === 0}>
        <p class={css.meta}>{t('admin-plugins-empty')}</p>
      </Show>

      <ul class={css.list}>
        <For each={slice.plugins()}>{(plugin) => <PluginCard plugin={plugin} slice={slice} />}</For>
      </ul>

      {/* Third-party allowlist: the trust surface for loading non-first-party components.
          Shares this screen's slice (one client, one load lifecycle). */}
      <AllowlistPanel slice={slice} />
    </section>
  );
}

function PluginCard(props: { plugin: PluginInfo; slice: PluginsSlice }): JSX.Element {
  const p = (): PluginInfo => props.plugin;
  const slice = props.slice;

  return (
    <li class={css.card} data-plugin-id={p().id} data-testid="plugin-card">
      <div class={css.cardHead}>
        <div>
          <p class={css.title}>
            <span dir="auto">{p().name}</span> <span class={css.meta}>{t('admin-plugins-version', { version: p().version })}</span>
          </p>
          <div class={css.row}>
            <Show
              when={p().signed}
              fallback={
                <span class={`${css.chip} ${css.unsignedChip}`} data-testid="sig-chip">
                  {t('admin-plugins-unsigned')}
                </span>
              }
            >
              <span class={`${css.chip} ${css.signedChip}`} data-testid="sig-chip">
                {t('admin-plugins-signed')}
              </span>
            </Show>
            <Show when={p().approved}>
              <span class={css.chip}>{t('admin-plugins-approved')}</span>
            </Show>
            <Show when={p().enabled}>
              <span class={css.chip}>{t('admin-plugins-enabled')}</span>
            </Show>
          </div>
        </div>
        <div class={css.row}>
          <Show when={!p().approved}>
            <button type="button" class={css.button} onClick={() => void slice.approve(p().id)}>
              {t('admin-plugins-approve')}
            </button>
          </Show>
          <Show when={p().approved && !p().enabled}>
            <button
              type="button"
              class={css.button}
              disabled={!p().signed && !p().allowUnsigned}
              onClick={() => void slice.enable(p().id)}
            >
              {t('admin-plugins-enable')}
            </button>
          </Show>
          <Show when={p().enabled}>
            <button type="button" class={css.danger} onClick={() => void slice.disable(p().id)}>
              {t('admin-plugins-disable')}
            </button>
          </Show>
        </div>
      </div>

      <div class={css.row}>
        <For each={p().capabilities}>
          {(cap) => (
            <span class={`${css.chip} ${css.capChip}`} data-testid="cap-chip">
              {cap}
            </span>
          )}
        </For>
      </div>

      <Show when={p().netAllowlist.length > 0}>
        <p class={css.limits}>{t('admin-plugins-net', { hosts: p().netAllowlist.join(', ') })}</p>
      </Show>
      <p class={css.limits}>
        {p().limits.fuel !== null
          ? t('admin-plugins-limits-fuel', {
              memory: p().limits.memoryMb,
              deadline: p().limits.deadlineMs,
              fuel: p().limits.fuel ?? 0,
            })
          : t('admin-plugins-limits', { memory: p().limits.memoryMb, deadline: p().limits.deadlineMs })}
      </p>

      <Show when={!p().signed}>
        <label class={css.check} data-testid="allow-unsigned">
          <input
            type="checkbox"
            aria-label={t('admin-plugins-allow-unsigned-for', { name: p().name })}
            checked={p().allowUnsigned}
            onChange={(e) => void slice.setAllowUnsigned(p().id, e.currentTarget.checked)}
          />
          <span>{t('admin-plugins-allow-unsigned')}</span>
        </label>
      </Show>
    </li>
  );
}

export default AdminPlugins;
