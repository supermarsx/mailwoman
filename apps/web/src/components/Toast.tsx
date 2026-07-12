import { Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';

export function Toast(): JSX.Element {
  const app = useApp();
  return (
    <Show when={app.toast()}>
      {(t) => (
        <div class={`toast toast--${t().kind}`} role="status" aria-live="polite">
          {t().message}
        </div>
      )}
    </Show>
  );
}
