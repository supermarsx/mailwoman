// TypeScript UI-plugin tier — SolidJS host surface (t10 plan §3/§6 e10, SPEC §22.2).
//
// Renders the approved+enabled UI plugins, each inside its own opaque-origin sandboxed
// iframe (`PLUGIN_IFRAME_SANDBOX` — `allow-scripts`, NO `allow-same-origin`), with a live
// deny-by-default postMessage broker wired to each frame's `contentWindow`. Approved-but-
// unsigned plugins raise a persistent, host-rendered trust banner the plugin cannot touch.
//
// ADDITIVE + fail-soft: when no plugin is approved (or the registry endpoint is
// absent/offline) the tier renders NOTHING, so the mailbox path is byte-unchanged. The
// tier is mounted lazily by the app shell (e13 MOUNT) — importing it has no side effects.

import { For, Show, createResource, onCleanup, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../i18n';
import { listUiPlugins } from './client';
import { attachBroker } from './broker';
import {
  PLUGIN_IFRAME_SANDBOX,
  buildGuestSrcdoc,
} from './host';
import { EMPTY_REGISTRY, type UiPluginRegistration, type UiPluginRegistry } from './types';
import * as css from './styles.css.ts';

export interface UiPluginTierProps {
  /// Injectable registry (tests / SSR pre-fetch). When omitted the tier fetches
  /// `GET /api/ui-plugins` on mount, fail-soft to the empty registry.
  registry?: UiPluginRegistry;
  /// Same-origin base for the registry + broker endpoints (default `''`).
  base?: string;
}

/// The host trust banner: shown ONLY when the registry reports unsigned plugins. Persistent
/// (no plugin-controllable dismiss), labelled, and colour-independent (icon + text).
export function UnsignedBanner(props: { readonly ids: readonly string[] }): JSX.Element {
  return (
    <Show when={props.ids.length > 0}>
      <section
        class={css.banner}
        role="note"
        aria-label={t('plugins-unsigned-warning-label')}
        data-testid="ui-plugin-unsigned-banner"
      >
        <span class={css.bannerIcon} aria-hidden="true">
          &#9888;
        </span>
        <div class={css.bannerBody}>
          <p class={css.bannerTitle}>{t('plugins-unsigned-title')}</p>
          <p class={css.bannerText}>
            These plugins were admitted without a verified signature. They run sandboxed, but
            their code has not been checked against a trusted key.
          </p>
          <ul class={css.bannerList}>
            <For each={props.ids}>
              {(id) => <li class={css.bannerPluginId}>{id}</li>}
            </For>
          </ul>
        </div>
      </section>
    </Show>
  );
}

/// One plugin's sandboxed frame + its broker connection. The broker attaches to the top
/// `window` on mount and detaches on cleanup, filtering to THIS frame's `contentWindow`.
function PluginFrame(props: {
  readonly registration: UiPluginRegistration;
  readonly base: string;
}): JSX.Element {
  let frameEl: HTMLIFrameElement | undefined;
  const manifest = props.registration.manifest;
  const srcdoc = buildGuestSrcdoc(manifest);

  // Registered synchronously in the component's reactive owner so the broker listener is
  // always torn down on unmount; `onFrameLoad` only assigns the live disconnect.
  let disconnect: (() => void) | undefined;
  onCleanup(() => disconnect?.());

  const onFrameLoad = (): void => {
    if (typeof window === 'undefined' || frameEl === undefined) return;
    disconnect?.(); // re-load ⇒ drop the previous connection first
    disconnect = attachBroker(window, {
      pluginId: manifest.id,
      grants: props.registration.grants,
      frameWindow: frameEl.contentWindow,
      base: props.base,
    });
  };

  return (
    <section class={css.slot} aria-label={`Plugin: ${manifest.name}`} data-plugin-id={manifest.id}>
      <span class={css.slotLabel}>{manifest.name}</span>
      <iframe
        ref={frameEl}
        class={css.frame}
        // SECURITY: opaque-origin sandbox. `allow-scripts` ONLY — never `allow-same-origin`.
        sandbox={PLUGIN_IFRAME_SANDBOX}
        referrerpolicy="no-referrer"
        title={`plugin:${manifest.id}`}
        srcdoc={srcdoc}
        onLoad={onFrameLoad}
      />
    </section>
  );
}

/// The UI-plugin tier. Renders the unsigned banner + one sandboxed frame per
/// approved+enabled plugin. Renders nothing at all when the registry is empty.
export function UiPluginTier(props: UiPluginTierProps): JSX.Element {
  // Pull this surface's copy catalog (unsigned-plugin banner); fail-soft — `t()`
  // shows the message id until it settles, then repaints reactively.
  onMount(() => void loadCatalog('plugins'));
  const base = (): string => props.base ?? '';
  const [fetched] = createResource(
    () => (props.registry === undefined ? base() : null),
    (b) => (b === null ? EMPTY_REGISTRY : listUiPlugins(b)),
  );
  const registry = (): UiPluginRegistry => props.registry ?? fetched() ?? EMPTY_REGISTRY;
  const active = (): readonly UiPluginRegistration[] =>
    registry().plugins.filter((p) => p.approved && p.enabled);

  // Track whether ANYTHING renders, so the tier is truly absent (no empty <div>) when
  // no plugin is configured — keeps the mailbox layout byte-unchanged.
  const hasContent = (): boolean =>
    registry().unsignedBanner.length > 0 || active().length > 0;

  return (
    <Show when={hasContent()}>
      <div class={css.tier} data-testid="ui-plugin-tier">
        <UnsignedBanner ids={registry().unsignedBanner} />
        <For each={active()}>
          {(reg) => <PluginFrame registration={reg} base={base()} />}
        </For>
      </div>
    </Show>
  );
}

export default UiPluginTier;
