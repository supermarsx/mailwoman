// V7 Assist chat panel (SPEC §14.3, plan §3 e6). The assistant is a client of the
// SAME tool surface as MCP (§20.3), so it inherits that surface's scoping and, in
// particular, its send-gating: any tool action the model proposes is shown for
// HUMAN review and routed to the Outbox — this panel NEVER offers a Send button and
// never transmits, deletes, or accepts anything itself.
//
// HARD RULE (§14, R4): when the gateway is `disabled` the panel renders NOTHING.

import { createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { AssistService } from './service.ts';
import { Disclosure } from './Disclosure.tsx';
import { hasCapability, type AssistConfig, type ChatMessage, type ContextItem, type ProposedAction } from './types.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

let msgSeq = 0;
function nextId(): string {
  msgSeq += 1;
  return `assist-msg-${msgSeq}`;
}

export interface AssistPanelProps {
  config: AssistConfig;
  service: AssistService;
  /** Mailbox context (e.g. the open thread) the assistant may reason over. */
  context?: readonly ContextItem[];
  /**
   * Review a proposed tool action. The Assist UI does NOT execute it: the host
   * opens the composer / Outbox so the human confirms. Absent ⇒ the review button
   * is inert (proposals are still shown for transparency).
   */
  onReviewAction?: (action: ProposedAction) => void;
}

export function AssistPanel(props: AssistPanelProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  const enabled = (): boolean => hasCapability(props.config, 'assistant');
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [draft, setDraft] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  async function ask(): Promise<void> {
    const prompt = draft().trim();
    if (prompt.length === 0 || busy()) return;
    setError(null);
    setMessages((prev) => [...prev, { id: nextId(), role: 'user', text: prompt }]);
    setDraft('');
    setBusy(true);
    try {
      const result = await props.service.invoke({
        capability: 'assistant',
        prompt,
        context: props.context ?? [],
      });
      setMessages((prev) => [
        ...prev,
        { id: nextId(), role: 'assistant', text: result.text, actions: result.actions },
      ]);
    } catch {
      setError(t('assist-error'));
    } finally {
      setBusy(false);
    }
  }

  // Render NOTHING when the assistant capability is not available (§14 hard-hide).
  return (
    <Show when={enabled()}>
      <section class={css.panel} data-module="assist" aria-label={t('assist-panel-label')}>
        <div class={css.section}>
          <h2 class={css.heading}>{t('assist-heading')}</h2>
          <Disclosure config={props.config} collapsible />

          <div class={css.transcript} role="log" aria-label={t('assist-transcript-label')}>
            <For each={messages()}>
              {(m) => (
                <div class={m.role === 'user' ? css.bubbleUser : css.bubbleAssistant} data-role={m.role}>
                  {/* Model / user text — `dir="auto"` isolates its bidi run. */}
                  <p class={css.prose} dir="auto">
                    {m.text}
                  </p>
                  <For each={m.actions ?? []}>
                    {(action) => (
                      <div class={css.proposal} data-testid="proposed-action">
                        <span class={css.subHeading}>{t('assist-proposed-action')}</span>
                        <p class={css.prose} dir="auto">
                          {action.summary}
                        </p>
                        <Show when={action.wouldSend}>
                          <p class={css.meta}>{t('assist-outbox-note')}</p>
                        </Show>
                        <div class={css.row}>
                          {/* Review only — the assistant never sends. The label is
                              intentionally "Review", never "Send". */}
                          <button
                            type="button"
                            class={css.ghost}
                            disabled={props.onReviewAction === undefined}
                            onClick={() => props.onReviewAction?.(action)}
                          >
                            {action.wouldSend ? t('assist-review-outbox') : t('assist-review')}
                          </button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              )}
            </For>
            <Show when={messages().length === 0}>
              <p class={css.meta}>{t('assist-empty')}</p>
            </Show>
          </div>

          <Show when={error() !== null}>
            <p class={css.error} role="alert">
              {error()}
            </p>
          </Show>

          <form
            class={css.row}
            onSubmit={(e) => {
              e.preventDefault();
              void ask();
            }}
          >
            <input
              class={css.input}
              style={{ flex: '1 1 auto' }}
              aria-label={t('assist-input-label')}
              placeholder={t('assist-input-placeholder')}
              value={draft()}
              onInput={(e) => setDraft(e.currentTarget.value)}
              disabled={busy()}
            />
            <button type="submit" class={css.button} disabled={busy() || draft().trim().length === 0}>
              {busy() ? t('assist-thinking') : t('assist-ask')}
            </button>
          </form>
        </div>
      </section>
    </Show>
  );
}
