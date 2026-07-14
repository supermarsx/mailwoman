// V7 semantic-search toggle (SPEC §14.3, plan §3 e6). A controlled switch that
// turns embedding-based re-ranking on for the search box. It is OFF by default and
// only rendered when the `search-semantic` capability is granted; enabling it means
// the query text is embedded via the Assist endpoint, so the disclosure says so.

import { Show, type JSX } from 'solid-js';
import { hasCapability, type AssistConfig } from './types.ts';
import * as css from './styles.css.ts';

export interface SemanticSearchToggleProps {
  config: AssistConfig;
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}

export function SemanticSearchToggle(props: SemanticSearchToggleProps): JSX.Element {
  return (
    <Show when={hasCapability(props.config, 'search-semantic')}>
      <label class={css.check} data-module="assist-semantic-search">
        <input
          type="checkbox"
          checked={props.enabled}
          onChange={(e) => props.onChange(e.currentTarget.checked)}
        />
        <span>Semantic search</span>
        <Show when={props.enabled}>
          <span class={css.meta}>Query text is sent to your Assist endpoint to rank by meaning.</span>
        </Show>
      </label>
    </Show>
  );
}
