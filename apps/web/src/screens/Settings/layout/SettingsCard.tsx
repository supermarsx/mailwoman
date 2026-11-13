import { type JSX } from 'solid-js';
import * as css from './layout.css.ts';

/** A titled group of rows, stacked full width in its section. `wide` is kept for
 * callers that marked tables and long lists; a stacked card is always full width.
 * `collapsible` makes it a native disclosure (closed until opened, keyboard and
 * screen-reader operable) for a rarely used block; a deep link opens it. */
export function SettingsCard(props: { id: string; title: string; description?: string; testId?: string; wide?: boolean; collapsible?: boolean; actions?: JSX.Element; children: JSX.Element }): JSX.Element {
  const head = (): JSX.Element => <>
    <h4 class={css.cardTitle} id={`${props.id}-title`}>{props.title}</h4>
    {props.description ? <p class={css.cardDesc} id={`${props.id}-desc`}>{props.description}</p> : null}
  </>;
  return <section class={css.card} id={props.id} aria-labelledby={`${props.id}-title`} data-testid={props.testId}>
    {props.collapsible
      ? <details class={css.disclosure}>
        <summary class={css.disclosureSummary}>{head()}</summary>
        <div class={css.rows}>{props.children}</div>
      </details>
      : <>
        <header class={props.actions?css.cardHeadWithActions:css.cardHead}>{props.actions?<><div class={css.cardHead}>{head()}</div>{props.actions}</>:head()}</header>
        <div class={css.rows}>{props.children}</div>
      </>}
  </section>;
}
