// Inline help (t32-ext.md §K): one sentence under a non-obvious field saying what
// it does and what it does not cover. Give it an `id` and point the field's
// `aria-describedby` at it (FormField does this for you).

import type { JSX } from 'solid-js';
import { Icon } from './icons.tsx';
import * as css from './shell.css.ts';

export interface HelpNoteProps {
  id?: string;
  children: JSX.Element;
}

export function HelpNote(props: HelpNoteProps): JSX.Element {
  return (
    <p id={props.id} class={css.helpNote}>
      <Icon name="info" />
      <span>{props.children}</span>
    </p>
  );
}
