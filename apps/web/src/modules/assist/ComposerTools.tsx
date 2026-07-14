// V7 inline composer tools (SPEC §14.3, plan §3 e6): grammar / rewrite / tone /
// translate. Each transform is offered as a SUGGESTION the user chooses to apply —
// nothing is auto-applied, and (like every Assist path) nothing is ever sent.
//
// Drop this beside a compose textarea: pass the current `text` and an `onApply`
// that replaces the draft body. Capabilities the user lacks are hidden; if the
// gateway is disabled the whole strip renders nothing.

import { createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { AssistService } from './service.ts';
import {
  COMPOSER_TOOLS,
  hasCapability,
  type AssistConfig,
  type ComposerTool,
  type ComposerToolSpec,
  type Disclosure as DisclosureInfo,
  type InvokeRequest,
} from './types.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

/** Localized visible label for a composer tool (the button caption). */
const toolLabel = (id: ComposerTool): string => t(`assist-tool-${id}`);
/** Localized label for a tool's argument select (Tone / Language). */
const toolArgLabel = (id: ComposerTool): string => t(`assist-toolarg-${id}`);

export interface ComposerToolsProps {
  config: AssistConfig;
  service: AssistService;
  /** The current draft body. */
  text: string;
  /** The account/folder the draft belongs to (tags the context for redaction). */
  account: string;
  folder?: string;
  /** Apply the transformed text to the draft (the user's explicit choice). */
  onApply: (text: string) => void;
  /** Record what left the device (fed to the per-message disclosure log). */
  onDisclosure?: (d: DisclosureInfo) => void;
}

interface Pending {
  readonly tool: ComposerTool;
  readonly text: string;
}

export function ComposerTools(props: ComposerToolsProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  const [busy, setBusy] = createSignal<ComposerTool | null>(null);
  const [pending, setPending] = createSignal<Pending | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [arg, setArg] = createSignal<Record<string, string>>({});

  const visibleTools = (): ComposerToolSpec[] =>
    COMPOSER_TOOLS.filter((t) => hasCapability(props.config, t.capability));

  function promptFor(spec: ComposerToolSpec): string {
    const a = spec.arg ? (arg()[spec.id] ?? spec.arg.options[0] ?? '') : '';
    switch (spec.id) {
      case 'grammar':
        return 'Correct spelling and grammar. Return only the corrected text.';
      case 'rewrite':
        return 'Rewrite this message to be clearer. Return only the rewritten text.';
      case 'tone':
        return `Rewrite this message in a ${a} tone. Return only the rewritten text.`;
      case 'translate':
        return `Translate this message into ${a}. Return only the translation.`;
    }
  }

  async function run(spec: ComposerToolSpec): Promise<void> {
    setError(null);
    setPending(null);
    if (props.text.trim().length === 0) {
      setError(t('assist-composer-empty-error'));
      return;
    }
    setBusy(spec.id);
    try {
      const req: InvokeRequest = {
        capability: spec.capability,
        prompt: promptFor(spec),
        context: [{ account: props.account, folder: props.folder ?? 'Drafts', text: props.text, kind: 'plain' }],
      };
      const result = await props.service.invoke(req);
      setPending({ tool: spec.id, text: result.text });
      props.onDisclosure?.(result.disclosure);
    } catch {
      setError(t('assist-composer-error'));
    } finally {
      setBusy(null);
    }
  }

  return (
    <Show when={visibleTools().length > 0}>
      <div class={css.field} data-module="assist-composer" aria-label={t('assist-composer-label')}>
        <div class={css.toolbar} role="group" aria-label={t('assist-composer-toolbar')}>
          <For each={visibleTools()}>
            {(spec) => (
              <div class={css.row}>
                <button
                  type="button"
                  class={css.ghost}
                  disabled={busy() !== null}
                  onClick={() => void run(spec)}
                >
                  {busy() === spec.id
                    ? t('assist-busy', { label: toolLabel(spec.id) })
                    : toolLabel(spec.id)}
                </button>
                <Show when={spec.arg !== undefined}>
                  <label class={css.check}>
                    <span class="sr-only">{toolArgLabel(spec.id)}</span>
                    <select
                      class={css.input}
                      aria-label={toolArgLabel(spec.id)}
                      value={arg()[spec.id] ?? spec.arg?.options[0] ?? ''}
                      onChange={(e) => setArg((prev) => ({ ...prev, [spec.id]: e.currentTarget.value }))}
                    >
                      <For each={spec.arg?.options ?? []}>{(o) => <option value={o}>{o}</option>}</For>
                    </select>
                  </label>
                </Show>
              </div>
            )}
          </For>
        </div>

        <Show when={error() !== null}>
          <p class={css.error} role="alert">
            {error()}
          </p>
        </Show>

        <Show when={pending()}>
          {(p) => (
            <div class={css.field}>
              <span class={css.subHeading}>{t('assist-suggested-edit')}</span>
              {/* Model output — `dir="auto"` isolates its bidi run. */}
              <div class={css.suggestion} data-testid="composer-suggestion" dir="auto">
                {p().text}
              </div>
              <div class={css.row}>
                <button
                  type="button"
                  class={css.button}
                  onClick={() => {
                    props.onApply(p().text);
                    setPending(null);
                  }}
                >
                  {t('assist-apply-draft')}
                </button>
                <button type="button" class={css.ghost} onClick={() => setPending(null)}>
                  {t('assist-discard')}
                </button>
              </div>
            </div>
          )}
        </Show>
      </div>
    </Show>
  );
}
