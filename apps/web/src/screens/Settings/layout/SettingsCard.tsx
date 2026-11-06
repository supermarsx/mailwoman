import { type JSX } from 'solid-js';
import * as css from './layout.css.ts';

/** A titled group of rows. The sub-navigation, collapse state and grid are
 * applied to it by SettingsToolbar through its data attributes. */
export function SettingsCard(props: { id: string; title: string; description?: string; testId?: string; /** Span every grid column (tables, long lists). */ wide?: boolean; children: JSX.Element }): JSX.Element {
  return <section class={props.wide ? `${css.card} ${css.cardWide}` : css.card} id={props.id} aria-labelledby={`${props.id}-title`} data-testid={props.testId}>
    <header class={css.cardHead}>
      <h4 class={css.cardTitle} id={`${props.id}-title`}>{props.title}</h4>
      {props.description ? <p class={css.cardDesc} id={`${props.id}-desc`}>{props.description}</p> : null}
    </header>
    <div class={css.rows}>{props.children}</div>
  </section>;
}
