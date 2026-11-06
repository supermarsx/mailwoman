// The shared busy indicator. Decorative by default (aria-hidden): the host
// already announces the work (a toast's role=status, an AsyncState message).
// Pass `label` where nothing else names it; it becomes a role=status text.
import type { JSX } from 'solid-js';
import * as css from './spinner.css.ts';

export function Spinner(props: { class?: string; label?: string; attr?: string }): JSX.Element {
  const ring = (): JSX.Element => (
    <span
      class={props.class ? `${css.spinner} ${props.class}` : css.spinner}
      data-mw-spinner=""
      data-toast-spinner={props.attr}
      aria-hidden="true"
    />
  );
  return props.label ? (
    <span role="status">
      {ring()}
      <span class={css.label}>{props.label}</span>
    </span>
  ) : (
    ring()
  );
}
