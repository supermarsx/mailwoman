import { For, type JSX } from 'solid-js';
import { SettingRow } from './SettingRow.tsx';
import * as legacy from '../styles.css.ts';
import type { SettingHandle } from '../useSetting.ts';

/** A setting's static description for the cross-section search index. */
export interface RowSpec { id: string; label: string; help?: string; keywords?: string }
/** A card's static description: `title`/`help`/row text are Fluent keys. */
export interface CardSpec { id: string; section: string; title: string; help?: string; rows: readonly RowSpec[] }

const frozen = (h: SettingHandle<unknown>): boolean => !h.entry() || h.locked() || h.saving();

export function ToggleRow(props: { id: string; label: string; help?: string; handle: SettingHandle<boolean>; onChange?: (value: boolean) => void; disabled?: boolean }): JSX.Element {
  return <SettingRow controlId={props.id} label={props.label} {...(props.help ? { help: props.help } : {})} handle={props.handle}>{describedBy =>
    <input id={props.id} type="checkbox" checked={props.handle.value()} disabled={props.disabled ?? frozen(props.handle)} aria-describedby={describedBy()}
      onChange={e => { if (props.onChange) props.onChange(e.currentTarget.checked); else void props.handle.set(e.currentTarget.checked); }} />
  }</SettingRow>;
}

export function SelectRow<T extends string>(props: { id: string; label: string; help?: string; handle: SettingHandle<T>; options: readonly { value: T; label: string }[]; accept: (v: string) => v is T; onPick?: (value: T) => void }): JSX.Element {
  return <SettingRow controlId={props.id} label={props.label} {...(props.help ? { help: props.help } : {})} handle={props.handle}>{describedBy =>
    <select id={props.id} class={legacy.select} value={props.handle.value()} disabled={frozen(props.handle)} aria-describedby={describedBy()}
      onChange={e => { const next = e.currentTarget.value; if (!props.accept(next)) return; if (props.onPick) props.onPick(next); else void props.handle.set(next); }}>
      <For each={props.options}>{o => <option value={o.value}>{o.label}</option>}</For>
    </select>
  }</SettingRow>;
}
