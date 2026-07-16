// Rules (mail-filter) settings section (audit #1, SPEC §6.1/§10.5). Mounted into
// Settings.tsx for an authenticated account. Three panes: the rule list + the
// condition/action builder, the raw-Sieve editor (highlight + lint), and the
// dry-run preview. Rides the EXISTING `MailRule` JMAP + server codegen/PUTSCRIPT
// path via the rules slice. WCAG 2.2 AA: labelled tablist, live regions.
//
// This module does NOT touch the router or the global store (ownership boundary):
// it instantiates its own rules slice over the same-origin JMAP client, mirroring
// how apikeys/zeroaccess mount self-contained. Tests inject a `slice` directly.

import { createMemo, createSignal, For, onMount, Show, type JSX } from 'solid-js';
import { t, loadCatalog, isolate } from '../../i18n';
import { createClient } from '../../api/client.ts';
import { useApp } from '../../state/context.ts';
import type { MailRule } from '../../api/crypto-types.ts';
import {
  createRulesSlice,
  emptyRuleDraft,
  type RuleDraft,
  type RulesSlice,
} from '../../state/slices/rules.ts';
import { RuleBuilder } from './RuleBuilder.tsx';
import { RawEditor } from './RawEditor.tsx';
import { DryRun } from './DryRun.tsx';
import { WhereItRuns } from './WhereItRuns.tsx';
import { rulesToSieve } from './sieve.ts';
import * as css from './styles.css.ts';

type Pane = 'builder' | 'raw' | 'dryrun';

const PANES: ReadonlyArray<{ id: Pane; label: string }> = [
  { id: 'builder', label: 'rules-tab-builder' },
  { id: 'raw', label: 'rules-tab-raw' },
  { id: 'dryrun', label: 'rules-tab-dryrun' },
];

export interface RulesSettingsProps {
  accountId: string;
  /** Injected in tests; production builds a same-origin JMAP-backed slice. */
  slice?: RulesSlice;
}

export function RulesSettings(props: RulesSettingsProps): JSX.Element {
  onMount(() => void loadCatalog('rules'));

  const slice: RulesSlice =
    props.slice ??
    createRulesSlice({ client: createClient(), showToast: useApp().showToast });

  const [pane, setPane] = createSignal<Pane>('builder');
  const [editing, setEditing] = createSignal<RuleDraft | null>(null);
  const [rawBuffer, setRawBuffer] = createSignal<string | null>(null);

  onMount(() => void slice.loadRules());

  const generated = createMemo(() => rulesToSieve(slice.rules()));
  const rawSource = (): string => rawBuffer() ?? generated();

  const startNew = (): void => {
    setEditing(emptyRuleDraft());
    setPane('builder');
  };

  const startEdit = (rule: MailRule): void => {
    setEditing({ ...rule });
    setPane('builder');
  };

  const onSave = async (draft: RuleDraft): Promise<void> => {
    await slice.saveRule(draft);
    setEditing(null);
  };

  return (
    <section class={css.panel} aria-labelledby="rules-heading">
      <h2 id="rules-heading" class={css.heading}>
        {t('rules-title')}
      </h2>
      <p class={css.prose}>{t('rules-intro')}</p>

      {/* Rule list */}
      <Show
        when={slice.rules().length > 0}
        fallback={<p class={css.prose}>{t('rules-empty')}</p>}
      >
        <ul class={css.list} aria-label={t('rules-list-label')}>
          <For each={slice.rules()}>
            {(rule) => (
              <li class={css.ruleRow}>
                <span class={css.ruleName}>{isolate(rule.name)}</span>
                <span class={rule.runsAt === 'server-sieve' ? `${css.badge} ${css.badgeServer}` : css.badge}>
                  {rule.runsAt === 'server-sieve' ? t('rules-runs-server') : t('rules-runs-engine')}
                </span>
                <label class={css.row}>
                  <input
                    type="checkbox"
                    checked={rule.enabled}
                    onChange={(e) => void slice.toggleRule(rule.id, e.currentTarget.checked)}
                  />
                  {t('rules-enabled')}
                </label>
                <button type="button" class={css.btn} onClick={() => startEdit(rule)}>
                  {t('common-edit')}
                </button>
                <button
                  type="button"
                  class={css.btnDanger}
                  aria-label={t('rules-delete-named', { name: isolate(rule.name) })}
                  onClick={() => void slice.deleteRule(rule.id)}
                >
                  {t('common-delete')}
                </button>
              </li>
            )}
          </For>
        </ul>
      </Show>

      <div class={css.row}>
        <button type="button" class={css.btnPrimary} onClick={startNew}>
          {t('rules-new')}
        </button>
      </div>

      {/* Panes */}
      <div class={css.tabs} role="tablist" aria-label={t('rules-panes-label')}>
        <For each={PANES}>
          {(p) => (
            <button
              type="button"
              role="tab"
              id={`rules-tab-${p.id}`}
              class={css.tab}
              aria-selected={pane() === p.id}
              aria-controls={`rules-panel-${p.id}`}
              onClick={() => setPane(p.id)}
            >
              {t(p.label)}
            </button>
          )}
        </For>
      </div>

      <Show when={pane() === 'builder'}>
        <div role="tabpanel" id="rules-panel-builder" aria-labelledby="rules-tab-builder">
          <Show
            when={editing()}
            fallback={<p class={css.prose}>{t('rules-builder-hint')}</p>}
          >
            {(draft) => (
              <RuleBuilder initial={draft()} onSave={(d) => void onSave(d)} onCancel={() => setEditing(null)} />
            )}
          </Show>
        </div>
      </Show>

      <Show when={pane() === 'raw'}>
        <div role="tabpanel" id="rules-panel-raw" aria-labelledby="rules-tab-raw">
          <RawEditor source={rawSource()} onInput={(next) => setRawBuffer(next)} />
          <div class={css.row}>
            <button type="button" class={css.btn} onClick={() => setRawBuffer(null)}>
              {t('rules-raw-reset')}
            </button>
          </div>
        </div>
      </Show>

      <Show when={pane() === 'dryrun'}>
        <div role="tabpanel" id="rules-panel-dryrun" aria-labelledby="rules-tab-dryrun">
          <DryRun rules={slice.rules()} />
        </div>
      </Show>

      {/* Standing where-it-runs legend so both modes are explained. */}
      <WhereItRuns runsAt="server-sieve" />
    </section>
  );
}
