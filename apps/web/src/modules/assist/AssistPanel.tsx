// V7 Assist chat panel (SPEC §14.3, plan §3 e6). The assistant is a client of the
// SAME tool surface as MCP (§20.3), so it inherits that surface's scoping and, in
// particular, its send-gating: any tool action the model proposes is shown for
// HUMAN review and routed to the Outbox — this panel NEVER offers a Send button and
// never transmits, deletes, or accepts anything itself.
//
// HARD RULE (§14, R4): when the gateway is `disabled` the panel renders NOTHING.

import { createSignal, For, Show, type JSX } from 'solid-js';
import { AssistService } from './service.ts';
import { Disclosure } from './Disclosure.tsx';
import { hasCapability, type AssistConfig, type ChatMessage, type ContextItem, type ProposedAction } from './types.ts';
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
      setError('The assistant could not respond just now.');
    } finally {
      setBusy(false);
    }
  }

  // Render NOTHING when the assistant capability is not available (§14 hard-hide).
  return (
    <Show when={enabled()}>
      <section class={css.panel} data-module="assist" aria-label="Assist">
        <div class={css.section}>
          <h2 class={css.heading}>Assistant</h2>
          <Disclosure config={props.config} collapsible />

          <div class={css.transcript} role="log" aria-label="Assistant conversation">
            <For each={messages()}>
              {(m) => (
                <div class={m.role === 'user' ? css.bubbleUser : css.bubbleAssistant} data-role={m.role}>
                  <p class={css.prose}>{m.text}</p>
                  <For each={m.actions ?? []}>
                    {(action) => (
                      <div class={css.proposal} data-testid="proposed-action">
                        <span class={css.subHeading}>Proposed action</span>
                        <p class={css.prose}>{action.summary}</p>
                        <Show when={action.wouldSend}>
                          <p class={css.meta}>
                            This would place a message in your Outbox. Nothing is sent until you confirm it
                            there.
                          </p>
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
                            {action.wouldSend ? 'Review in Outbox' : 'Review'}
                          </button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              )}
            </For>
            <Show when={messages().length === 0}>
              <p class={css.meta}>Ask the assistant to summarise, draft, or find things. It can propose
                actions, but you always confirm them yourself.</p>
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
              aria-label="Message the assistant"
              placeholder="Ask the assistant…"
              value={draft()}
              onInput={(e) => setDraft(e.currentTarget.value)}
              disabled={busy()}
            />
            <button type="submit" class={css.button} disabled={busy() || draft().trim().length === 0}>
              {busy() ? 'Thinking…' : 'Ask'}
            </button>
          </form>
        </div>
      </section>
    </Show>
  );
}
