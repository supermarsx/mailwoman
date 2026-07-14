// V7 Assist panel (SPEC §14, plan §3 e6). SCAFFOLD stub (e0): inert placeholder.
// e6 builds the real chat panel + composer tools + dictation + disclosure; e14
// wires it to the gateway. Hidden entirely when availability === 'disabled'.

import { Show, For, type JSX } from 'solid-js';
import { WHAT_LEFT_THE_DEVICE, type AssistAvailability } from './index.ts';

export interface AssistPanelProps {
  /** Gateway availability. 'disabled' ⇒ render NOTHING (the §14 hard-hide rule). */
  availability?: AssistAvailability;
}

export function AssistPanel(props: AssistPanelProps): JSX.Element {
  const available = (): boolean => (props.availability ?? 'disabled') === 'enabled';
  return (
    <Show when={available()}>
      <section data-module="assist" aria-label="Assist">
        <p>Assist is not yet implemented (t7 e6).</p>
        <details>
          <summary>What could leave this device</summary>
          <ul>
            <For each={WHAT_LEFT_THE_DEVICE}>{(item) => <li>{item}</li>}</For>
          </ul>
        </details>
      </section>
    </Show>
  );
}
