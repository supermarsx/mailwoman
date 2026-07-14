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

import { createEffect, For, Show, type JSX } from 'solid-js';
import {
  createPluginsSlice,
  createHttpPluginsApi,
  type PluginsApi,
  type PluginsSlice,
  type PluginInfo,
} from '../../../state/slices/plugins.ts';
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

  // Load once on mount (idempotent; e14 may also pre-load).
  createEffect(() => {
    void slice.load();
  });

  return (
    <section class={css.screen} data-screen="admin-plugins" aria-label="Plugins">
      <div>
        <h2 class={css.heading}>Plugins</h2>
        <p class={css.prose}>
          Engine plugins run in a capability-gated WebAssembly sandbox. Approve a plugin before it
          can be enabled, and grant only the capabilities it needs.
        </p>
      </div>

      <Show when={slice.hasUnsignedEnabled()}>
        <p class={css.unsignedBanner} role="alert" data-testid="unsigned-banner">
          {UNSIGNED_BANNER}
        </p>
      </Show>

      <Show when={!slice.loading() && slice.plugins().length === 0}>
        <p class={css.meta}>No plugins are registered.</p>
      </Show>

      <ul class={css.list}>
        <For each={slice.plugins()}>{(plugin) => <PluginCard plugin={plugin} slice={slice} />}</For>
      </ul>
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
            {p().name} <span class={css.meta}>v{p().version}</span>
          </p>
          <div class={css.row}>
            <Show
              when={p().signed}
              fallback={
                <span class={`${css.chip} ${css.unsignedChip}`} data-testid="sig-chip">
                  Unsigned
                </span>
              }
            >
              <span class={`${css.chip} ${css.signedChip}`} data-testid="sig-chip">
                Signed
              </span>
            </Show>
            <Show when={p().approved}>
              <span class={css.chip}>Approved</span>
            </Show>
            <Show when={p().enabled}>
              <span class={css.chip}>Enabled</span>
            </Show>
          </div>
        </div>
        <div class={css.row}>
          <Show when={!p().approved}>
            <button type="button" class={css.button} onClick={() => void slice.approve(p().id)}>
              Approve
            </button>
          </Show>
          <Show when={p().approved && !p().enabled}>
            <button
              type="button"
              class={css.button}
              disabled={!p().signed && !p().allowUnsigned}
              onClick={() => void slice.enable(p().id)}
            >
              Enable
            </button>
          </Show>
          <Show when={p().enabled}>
            <button type="button" class={css.danger} onClick={() => void slice.disable(p().id)}>
              Disable
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
        <p class={css.limits}>net: {p().netAllowlist.join(', ')}</p>
      </Show>
      <p class={css.limits}>
        limits: {p().limits.memoryMb} MiB · {p().limits.deadlineMs} ms
        {p().limits.fuel !== null ? ` · ${p().limits.fuel} fuel` : ''}
      </p>

      <Show when={!p().signed}>
        <label class={css.check} data-testid="allow-unsigned">
          <input
            type="checkbox"
            aria-label={`Allow unsigned plugin ${p().name}`}
            checked={p().allowUnsigned}
            onChange={(e) => void slice.setAllowUnsigned(p().id, e.currentTarget.checked)}
          />
          <span>Allow this unsigned plugin to run</span>
        </label>
      </Show>
    </li>
  );
}

export default AdminPlugins;
