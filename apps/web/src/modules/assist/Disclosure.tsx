// V7 "what left the device" disclosure (SPEC §14, plan §1.5 / R4). Shown wherever
// an Assist tool can forward content, so the honesty claim is concrete: it names
// the endpoint host and the ceilings actually in force, and states plainly that
// send is never automated.
//
// i18n (t8): the sentence + the "what can leave" list are authored in `assist.ftl`
// (assist-disclosure-*, assist-left-*). The English wording is unchanged — the
// honesty claim ("never sends …", "excluded by default") stays accurate.

import { For, Show, onMount, type JSX } from 'solid-js';
import { disclosureSentence, type AssistConfig } from './types.ts';
import { t, loadCatalog } from '../../i18n';
import * as css from './styles.css.ts';

/** The five "what can leave this device" line ids, in display order. */
const WHAT_LEFT_IDS = [
  'assist-left-1',
  'assist-left-2',
  'assist-left-3',
  'assist-left-4',
  'assist-left-5',
] as const;

export interface DisclosureProps {
  config: AssistConfig;
  /** Render collapsed by default (a `<details>`); the summary is always visible. */
  collapsible?: boolean;
}

export function Disclosure(props: DisclosureProps): JSX.Element {
  onMount(() => void loadCatalog('assist'));
  const list = (): JSX.Element => (
    <ul>
      <For each={WHAT_LEFT_IDS}>{(id) => <li>{t(id)}</li>}</For>
    </ul>
  );
  return (
    <div class={css.disclosure} data-testid="assist-disclosure">
      <p class={css.prose}>{disclosureSentence(props.config)}</p>
      <Show when={props.collapsible ?? false} fallback={list()}>
        <details>
          <summary>{t('assist-disclosure-summary')}</summary>
          {list()}
        </details>
      </Show>
    </div>
  );
}
