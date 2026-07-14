// V7 auto-tag (SPEC §14.3, plan §3 e6). SUGGEST-MODE by default: the model proposes
// labels as badges; the user applies each one explicitly, and every suggestion →
// apply → revert step is written to an audit trail. AUTO-MODE (apply without asking)
// is strictly OPT-IN and, when on, still records who acted ('assist' vs 'user').
//
// The Assist UI never mutates mail itself — it calls `onApply`/`onRevert` (the mail
// slice owns keyword mutation) and `onAudit` (the audit trail). No send path.

import { createEffect, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { hasCapability, type AssistConfig, type AutoTagMode, type TagAuditEntry, type TagSuggestion } from './types.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

let auditSeq = 0;
function auditId(): string {
  auditSeq += 1;
  return `tagaudit-${Date.now()}-${auditSeq}`;
}

export interface AutoTagProps {
  config: AssistConfig;
  messageId: string;
  /** The model's proposed labels (fetched by the caller via the summarize/auto-tag path). */
  suggestions: readonly TagSuggestion[];
  /** suggest (default) shows badges to apply; auto applies immediately. Opt-in to change. */
  mode?: AutoTagMode;
  onModeChange?: (mode: AutoTagMode) => void;
  /** Apply a keyword to the message (mail slice owns the real mutation). */
  onApply: (keyword: string) => void;
  /** Remove a previously applied keyword (bulk-reverse support). */
  onRevert?: (keyword: string) => void;
  /** Append an entry to the audit trail (§14 attribution). */
  onAudit?: (entry: TagAuditEntry) => void;
}

export function AutoTag(props: AutoTagProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  const [applied, setApplied] = createSignal<Set<string>>(new Set());
  // Suggestions already handled (suggested/auto-applied) so a manual revert is not
  // undone by the effect re-running, and each suggestion is audited exactly once.
  const processed = new Set<string>();
  const mode = (): AutoTagMode => props.mode ?? 'suggest';

  function record(keyword: string, action: TagAuditEntry['action'], actor: TagAuditEntry['actor']): void {
    props.onAudit?.({
      id: auditId(),
      messageId: props.messageId,
      keyword,
      action,
      actor,
      ts: new Date().toISOString(),
    });
  }

  function apply(keyword: string, actor: TagAuditEntry['actor']): void {
    if (applied().has(keyword)) return;
    props.onApply(keyword);
    setApplied((prev) => new Set(prev).add(keyword));
    record(keyword, 'applied', actor);
  }

  function revert(keyword: string): void {
    props.onRevert?.(keyword);
    setApplied((prev) => {
      const next = new Set(prev);
      next.delete(keyword);
      return next;
    });
    record(keyword, 'reverted', 'user');
  }

  // In suggest-mode each proposal is audited as 'suggested'; in auto-mode it is
  // applied immediately on Assist's behalf (still audited + reversible). Each
  // keyword is processed once, so a manual revert is never re-applied by the effect.
  createEffect(() => {
    const current = mode();
    for (const s of props.suggestions) {
      if (processed.has(s.keyword)) continue;
      processed.add(s.keyword);
      if (current === 'auto') apply(s.keyword, 'assist');
      else record(s.keyword, 'suggested', 'assist');
    }
  });

  return (
    <Show when={hasCapability(props.config, 'auto-tag') && props.suggestions.length > 0}>
      <div class={css.field} data-module="assist-autotag" aria-label={t('assist-autotag-label')}>
        <div class={css.row}>
          <span class={css.subHeading}>{t('assist-autotag-label')}</span>
          <label class={css.check}>
            <input
              type="checkbox"
              checked={mode() === 'auto'}
              onChange={(e) => props.onModeChange?.(e.currentTarget.checked ? 'auto' : 'suggest')}
            />
            <span>{t('assist-apply-auto')}</span>
          </label>
        </div>

        <div class={css.toolbar}>
          <For each={props.suggestions}>
            {(s) => (
              <span class={css.badge} data-testid="tag-suggestion">
                {/* Model-proposed label — `dir="auto"` isolates its bidi run. */}
                <span dir="auto">{s.label}</span>
                <span class={css.meta}>{Math.round(s.confidence * 100)}%</span>
                <Show
                  when={!applied().has(s.keyword)}
                  fallback={
                    <button
                      type="button"
                      class={css.ghost}
                      aria-label={t('assist-remove-label', { label: s.label })}
                      onClick={() => revert(s.keyword)}
                    >
                      {t('assist-undo')}
                    </button>
                  }
                >
                  <button
                    type="button"
                    class={css.ghost}
                    aria-label={t('assist-apply-label', { label: s.label })}
                    onClick={() => apply(s.keyword, 'user')}
                  >
                    {t('assist-apply')}
                  </button>
                </Show>
              </span>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
