// V7 semantic-search toggle (SPEC §14.3, plan §3 e6). A controlled switch that
// turns embedding-based re-ranking on for the search box. It is OFF by default and
// only rendered when the `search-semantic` capability is granted; enabling it means
// the query text is embedded via the Assist endpoint, so the disclosure says so.

import { onMount, Show, type JSX } from 'solid-js';
import { hasCapability, type AssistConfig } from './types.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

export interface SemanticSearchToggleProps {
  config: AssistConfig;
  enabled: boolean;
  onChange: (enabled: boolean) => void;
}

export function SemanticSearchToggle(props: SemanticSearchToggleProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  return (
    <Show when={hasCapability(props.config, 'search-semantic')}>
      <label class={css.check} data-module="assist-semantic-search">
        <input
          type="checkbox"
          checked={props.enabled}
          onChange={(e) => props.onChange(e.currentTarget.checked)}
        />
        <span>{t('assist-semantic-label')}</span>
        <Show when={props.enabled}>
          <span class={css.meta}>{t('assist-semantic-note')}</span>
        </Show>
      </label>
    </Show>
  );
}
