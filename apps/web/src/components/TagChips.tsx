import { Icon, tagIconName } from './Icon.tsx';
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

export function labelTextColor(color:string):string {
  if(!/^#[0-9a-f]{6}$/i.test(color))return '#fff';
  const channels=[1,3,5].map(offset=>{const value=parseInt(color.slice(offset,offset+2),16)/255;return value<=0.04045?value/12.92:((value+0.055)/1.055)**2.4;});
  return channels[0]!*0.2126+channels[1]!*0.7152+channels[2]!*0.0722>0.179?'#000':'#fff';
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
                style={tag ? { 'background-color': tag.color, color: labelTextColor(tag.color) } : undefined}
                data-keyword={kw}
              >
                <Show when={tag?.icon}>{(icon) => <span class="tag-chip__icon"><Icon name={tagIconName(icon())} /></span>}</Show>
                {tag?.name ?? kw}
              </span>
            );
          }}
        </For>
      </span>
    </Show>
  );
}
