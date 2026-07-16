// Raw-Sieve editor (audit #1): an editable Sieve view with syntax highlighting
// and a live lint surface. The server owns codegen/upload; this is the
// power-user read/lint pane. WCAG 2.2 AA: labelled control, live diagnostics.

import { createMemo, For, Show, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { lintSieve, tokenizeSieve, type Token } from './sieve.ts';
import * as css from './styles.css.ts';

export interface RawEditorProps {
  /** The Sieve source to show (generated from the current rules). */
  source: string;
  /** Called on every edit (the parent keeps the edited buffer). */
  onInput?: (next: string) => void;
  /** Read-only when no `onInput` is supplied. */
  readOnly?: boolean;
}

const tokenClass: Partial<Record<Token['kind'], string>> = {
  keyword: css.tokKeyword,
  string: css.tokString,
  tag: css.tokTag,
  number: css.tokNumber,
  comment: css.tokComment,
};

/** Render the highlighted, read-only mirror of the source. */
function Highlighted(props: { source: string }): JSX.Element {
  const tokens = createMemo(() => tokenizeSieve(props.source));
  return (
    <pre class={css.highlight} aria-hidden="true">
      <For each={tokens()}>
        {(tok) => {
          const cls = tokenClass[tok.kind];
          return cls === undefined ? <span>{tok.text}</span> : <span class={cls}>{tok.text}</span>;
        }}
      </For>
    </pre>
  );
}

export function RawEditor(props: RawEditorProps): JSX.Element {
  const diagnostics = createMemo(() => lintSieve(props.source));
  const editable = (): boolean => props.onInput !== undefined && props.readOnly !== true;

  return (
    <div class={css.editorWrap}>
      <label class={css.label} for="rules-raw-sieve">
        {t('rules-raw-label')}
      </label>
      <Show
        when={editable()}
        fallback={<Highlighted source={props.source} />}
      >
        <textarea
          id="rules-raw-sieve"
          class={css.textarea}
          spellcheck={false}
          value={props.source}
          aria-describedby="rules-raw-diagnostics"
          onInput={(e) => props.onInput?.(e.currentTarget.value)}
        />
        <Highlighted source={props.source} />
      </Show>

      <div id="rules-raw-diagnostics" role="status" aria-live="polite">
        <Show
          when={diagnostics().length > 0}
          fallback={<p class={css.okNote}>{t('rules-lint-clean')}</p>}
        >
          <ul class={css.diagList}>
            <For each={diagnostics()}>{(d) => <li>{d}</li>}</For>
          </ul>
        </Show>
      </div>
    </div>
  );
}
