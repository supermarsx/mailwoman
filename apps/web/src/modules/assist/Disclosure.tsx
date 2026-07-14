// V7 "what left the device" disclosure (SPEC §14, plan §1.5 / R4). Shown wherever
// an Assist tool can forward content, so the honesty claim is concrete: it names
// the endpoint host and the ceilings actually in force, and states plainly that
// send is never automated.

import { For, Show, type JSX } from 'solid-js';
import { disclosureSentence, WHAT_LEFT_THE_DEVICE, type AssistConfig } from './types.ts';
import * as css from './styles.css.ts';

export interface DisclosureProps {
  config: AssistConfig;
  /** Render collapsed by default (a `<details>`); the summary is always visible. */
  collapsible?: boolean;
}

export function Disclosure(props: DisclosureProps): JSX.Element {
  const list = (): JSX.Element => (
    <ul>
      <For each={WHAT_LEFT_THE_DEVICE}>{(item) => <li>{item}</li>}</For>
    </ul>
  );
  return (
    <div class={css.disclosure} data-testid="assist-disclosure">
      <p class={css.prose}>{disclosureSentence(props.config)}</p>
      <Show when={props.collapsible ?? false} fallback={list()}>
        <details>
          <summary>What can leave this device</summary>
          {list()}
        </details>
      </Show>
    </div>
  );
}
