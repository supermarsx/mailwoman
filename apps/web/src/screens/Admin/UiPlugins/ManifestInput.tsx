// A UI plugin's manifest, read from a file, an address on this server, or pasted
// text, and shown before it is registered (t32-cfg §4.12: manifest upload or URL
// with preview instead of pasted JSON).
//
// The console's content security policy lets it fetch only from its own origin
// (`connect-src 'self'`), so an address elsewhere is refused with that reason and
// the admin is asked to download the file and upload it; nothing is fetched by a
// side route. The manifest is checked here the way the server checks it
// (`UiManifest`, crates/mw-server/src/ui_plugins.rs: id, name and version are
// text, capabilities are known ones), and the preview lists what it declares.

import { For, Show, createSignal, createUniqueId, type JSX } from 'solid-js';
import { isolate, t } from '../../../i18n';
import { apiFetch } from '../../../api/http.ts';
import { UI_CAPABILITIES } from '../../../plugins-ui/types.ts';
import { StatusBadge } from '../shell/StatusBadge.tsx';
import * as shell from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as css from '../shell/extras.css.ts';
import { readFileText } from '../shell/files.ts';

export interface ParsedManifest {
  id: string;
  name: string;
  version: string;
  signed: boolean;
  capabilities: string[];
  extensionPoints: string[];
  /** The manifest as read, sent to the server unchanged. */
  raw: Record<string, unknown>;
}

/** The form value: the text as typed or read, and the manifest when it is valid. */
export interface ManifestValue {
  text: string;
  parsed: ParsedManifest | null;
  /** Why the text is not a usable manifest, or null. */
  problem: string | null;
}

export const EMPTY_MANIFEST: ManifestValue = { text: '', parsed: null, problem: null };

/** Check a manifest text the way the server reads it. */
export function readManifest(text: string): ManifestValue {
  if (text.trim() === '') return EMPTY_MANIFEST;
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch {
    return { text, parsed: null, problem: t('plugins-admin-ui-manifest-invalid') };
  }
  if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) return { text, parsed: null, problem: t('plugins-admin-ui-manifest-not-object') };
  const o = raw as Record<string, unknown>;
  for (const key of ['id', 'name', 'version'] as const) {
    if (typeof o[key] !== 'string' || (o[key] as string).trim() === '') return { text, parsed: null, problem: t('plugins-admin-ui-manifest-missing', { key }) };
  }
  const list = (v: unknown): string[] | null => (v === undefined ? [] : Array.isArray(v) && v.every((x) => typeof x === 'string') ? (v as string[]) : null);
  const capabilities = list(o.capabilities);
  const extensionPoints = list(o.extensionPoints);
  if (capabilities === null || extensionPoints === null) return { text, parsed: null, problem: t('plugins-admin-ui-manifest-lists') };
  const unknown = capabilities.find((c) => !(UI_CAPABILITIES as readonly string[]).includes(c));
  if (unknown !== undefined) return { text, parsed: null, problem: t('plugins-admin-ui-manifest-capability', { capability: isolate(unknown) }) };
  return {
    text,
    problem: null,
    parsed: {
      id: o.id as string,
      name: o.name as string,
      version: o.version as string,
      signed: typeof o.signature === 'string' && o.signature !== '',
      capabilities,
      extensionPoints,
      raw: o,
    },
  };
}

/** Whether `address` is on this console's own origin (the only one it may fetch). */
export function sameOrigin(address: string, here: string = location.href): boolean {
  try {
    return new URL(address, here).origin === new URL(here).origin;
  } catch {
    return false;
  }
}

export interface ManifestInputProps {
  id: string;
  describedBy: string | undefined;
  invalid: boolean;
  value: ManifestValue;
  onChange: (v: ManifestValue) => void;
}

export function ManifestInput(props: ManifestInputProps): JSX.Element {
  const own = createUniqueId();
  const [address, setAddress] = createSignal('');
  const [fetchProblem, setFetchProblem] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  let file: HTMLInputElement | undefined;

  async function fromFile(f: File | undefined): Promise<void> {
    if (!f) return;
    props.onChange(readManifest(await readFileText(f)));
    if (file) file.value = '';
  }

  async function fromAddress(): Promise<void> {
    setFetchProblem(null);
    const a = address().trim();
    if (!sameOrigin(a)) {
      setFetchProblem(t('plugins-admin-ui-address-elsewhere'));
      return;
    }
    setBusy(true);
    try {
      const res = await apiFetch(new URL(a, location.href).href, { credentials: 'same-origin' });
      if (!res.ok) {
        setFetchProblem(t('plugins-admin-ui-address-failed', { status: res.status }));
        return;
      }
      props.onChange(readManifest(await res.text()));
    } catch {
      setFetchProblem(t('plugins-admin-ui-address-failed', { status: 0 }));
    } finally {
      setBusy(false);
    }
  }

  const m = (): ParsedManifest | null => props.value.parsed;

  return (
    <div class={css.listEditor} id={props.id} role="group" aria-label={t('plugins-admin-ui-manifest')} aria-describedby={props.describedBy} data-testid="ui-manifest">
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-file`}>
          {t('plugins-admin-ui-manifest-file')}
        </label>
        <input ref={file} id={`${own}-file`} class={css.fileInput} type="file" accept=".json,application/json" onChange={(e) => void fromFile(e.currentTarget.files?.[0])} />
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-address`}>
          {t('plugins-admin-ui-manifest-address')}
        </label>
        <div class={css.listAdd}>
          <input
            id={`${own}-address`}
            class={`${ux.control} ${css.listInput}`}
            type="url"
            spellcheck={false}
            value={address()}
            aria-describedby={`${own}-address-help${fetchProblem() ? ` ${own}-address-error` : ''}`}
            aria-invalid={fetchProblem() ? true : undefined}
            onInput={(e) => setAddress(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                void fromAddress();
              }
            }}
          />
          <button type="button" class="btn" disabled={busy() || address().trim() === ''} onClick={() => void fromAddress()}>
            {t('plugins-admin-ui-manifest-read')}
          </button>
        </div>
        <p id={`${own}-address-help`} class={shell.help}>
          {t('plugins-admin-ui-manifest-address-help')}
        </p>
        <Show when={fetchProblem()}>
          {(msg) => (
            <p id={`${own}-address-error`} class={shell.fieldError} role="alert">
              <span>{msg()}</span>
            </p>
          )}
        </Show>
      </div>
      <div class={shell.field}>
        <label class={shell.fieldLabel} for={`${own}-paste`}>
          {t('plugins-admin-ui-manifest-paste')}
        </label>
        <textarea
          id={`${own}-paste`}
          class={`${ux.control} ${css.mono}`}
          rows={4}
          spellcheck={false}
          value={props.value.text}
          aria-invalid={props.invalid ? true : undefined}
          onInput={(e) => props.onChange(readManifest(e.currentTarget.value))}
        />
      </div>
      <Show when={m()}>
        {(p) => (
          <div class={css.panel} data-testid="ui-manifest-preview">
            <h4 class={css.panelTitle}>{t('plugins-admin-ui-preview')}</h4>
            <dl class={css.summary}>
              <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-id')}</dt>
              <dd class={`${css.summaryValue} ${css.mono}`}>{p().id}</dd>
              <dt class={css.summaryTerm}>{t('plugins-admin-register-name')}</dt>
              <dd class={css.summaryValue} dir="auto">
                {p().name}
              </dd>
              <dt class={css.summaryTerm}>{t('plugins-admin-register-version')}</dt>
              <dd class={css.summaryValue}>{p().version}</dd>
              <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-signature')}</dt>
              <dd class={css.summaryValue}>
                <StatusBadge tone={p().signed ? 'ok' : 'warn'}>{p().signed ? t('admin-plugins-signed') : t('admin-plugins-unsigned')}</StatusBadge>
              </dd>
              <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-capabilities')}</dt>
              <dd class={`${css.summaryValue} ${css.mono}`}>{p().capabilities.length > 0 ? p().capabilities.join(', ') : t('plugins-admin-hosts-empty')}</dd>
              <dt class={css.summaryTerm}>{t('plugins-admin-ui-preview-points')}</dt>
              <dd class={css.summaryValue}>
                <Show when={p().extensionPoints.length > 0} fallback={t('plugins-admin-hosts-empty')}>
                  <ul class={css.listItems}>
                    <For each={p().extensionPoints}>{(x) => <li class={css.mono}>{x}</li>}</For>
                  </ul>
                </Show>
              </dd>
            </dl>
          </div>
        )}
      </Show>
    </div>
  );
}
