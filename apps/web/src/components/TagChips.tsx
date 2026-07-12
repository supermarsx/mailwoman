import { For, Show, type JSX } from 'solid-js';
import { useApp } from '../state/context.ts';
import { isLabelKeyword } from '../state/slices/tags.ts';
import type { Email } from '../api/jmap-types.ts';

// Renders an email's user labels as colored chips from the tag registry (§1.5).
// System `$`-keywords ($seen/$flagged/…) are filtered out; unregistered labels
// fall back to a neutral chip so a keyword set on another client still shows.

/** The label keywords set on an email, in a stable order. */
export function labelKeywords(email: Email): string[] {
  return Object.keys(email.keywords ?? {})
    .filter((k) => email.keywords?.[k] === true && isLabelKeyword(k))
    .sort();
}

export function TagChips(props: { email: Email }): JSX.Element {
  const app = useApp();
  const labels = () => labelKeywords(props.email);

  return (
    <Show when={labels().length > 0}>
      <span class="tag-chips">
        <For each={labels()}>
          {(kw) => {
            const tag = app.tagByKeyword(kw);
            return (
              <span
                class="tag-chip"
                style={tag ? { 'background-color': tag.color, color: '#fff' } : undefined}
                data-keyword={kw}
              >
                <Show when={tag?.icon}>{(icon) => <span class="tag-chip__icon">{icon()}</span>}</Show>
                {tag?.name ?? kw}
              </span>
            );
          }}
        </For>
      </span>
    </Show>
  );
}
