// Dry-run preview (audit #1): match the current rule set against a sample message
// and show which rules fire (and what they do), honouring `stop`. Purely local —
// mirrors the engine evaluator; nothing is sent.

import { createMemo, createSignal, For, Show, type JSX } from 'solid-js';
import { t, isolate } from '../../i18n';
import type { MailRule } from '../../api/crypto-types.ts';
import { dryRun, type SampleMessage } from './sieve.ts';
import * as css from './styles.css.ts';

export interface DryRunProps {
  rules: MailRule[];
}

export function DryRun(props: DryRunProps): JSX.Element {
  const [sample, setSample] = createSignal<SampleMessage>({
    from: 'sender@example.com',
    to: 'me@example.com',
    subject: 'Hello',
  });

  const results = createMemo(() => dryRun(props.rules, sample()));

  const field = (key: keyof SampleMessage, labelId: string): JSX.Element => (
    <div class={css.field}>
      <label class={css.label} for={`dryrun-${key}`}>
        {t(labelId)}
      </label>
      <input
        id={`dryrun-${key}`}
        class={css.input}
        value={sample()[key]}
        onInput={(e) => setSample((s) => ({ ...s, [key]: e.currentTarget.value }))}
      />
    </div>
  );

  return (
    <section class={css.builder} aria-label={t('rules-dryrun-title')}>
      <h3 class={css.heading}>{t('rules-dryrun-title')}</h3>
      <p class={css.prose}>{t('rules-dryrun-help')}</p>
      <div class={css.row}>
        {field('from', 'rules-cond-from')}
        {field('to', 'rules-cond-to')}
        {field('subject', 'rules-cond-subject')}
      </div>

      <div class={css.dryGrid} role="list" aria-label={t('rules-dryrun-results')}>
        <Show
          when={results().length > 0}
          fallback={<p class={css.prose}>{t('rules-dryrun-none')}</p>}
        >
          <For each={results()}>
            {(r) => (
              <div class={css.dryResult} role="listitem">
                <span
                  class={r.matched ? css.matchYes : css.matchNo}
                  aria-hidden="true"
                />
                <span class={css.ruleName}>{isolate(r.ruleName)}</span>
                <Show
                  when={r.shortCircuited}
                  fallback={
                    <Show
                      when={r.matched}
                      fallback={<span class={css.prose}>{t('rules-dryrun-nomatch')}</span>}
                    >
                      <span class={css.prose}>{r.actions.join(', ')}</span>
                    </Show>
                  }
                >
                  <span class={css.prose}>{t('rules-dryrun-stopped')}</span>
                </Show>
              </div>
            )}
          </For>
        </Show>
      </div>
    </section>
  );
}
