// A byte size with units for `attachments.maxBytes`: an amount and a unit
// (B, KiB, MiB, GiB), with the result in words. Zero means no limit. The draft
// stays the plain number of bytes the registry stores.

import { For, Show, createEffect, createSignal, on, type JSX } from 'solid-js';
import { formatBytes, formatNumber, t } from '../../../i18n';
import type { InlineEditorContext } from '../Settings/InlineSettingEditor.tsx';
import { BYTE_UNITS, splitBytes, toBytes, type ByteUnit } from './rules.ts';
import * as css from './attachments.css.ts';

export function ByteSizeEditor(props: { ctx: InlineEditorContext }): JSX.Element {
  const bytes = (): number => (typeof props.ctx.draft === 'string' && props.ctx.draft.trim() !== '' ? Number(props.ctx.draft) : 0);
  const initial = splitBytes(bytes());
  const [unit, setUnit] = createSignal<ByteUnit>(initial.unit);
  const [text, setText] = createSignal(String(initial.amount));
  let lastWritten = bytes();
  // A draft changed from outside (Reset, a reload) is shown again in a fitting unit.
  createEffect(on(bytes, (value) => {
    if (value === lastWritten) return;
    lastWritten = value;
    const s = splitBytes(Number.isFinite(value) ? value : 0);
    setUnit(s.unit);
    setText(String(s.amount));
  }, { defer: true }));
  function commit(nextText: string, nextUnit: ByteUnit): void {
    setText(nextText);
    setUnit(nextUnit);
    const amount = Number(nextText.replace(',', '.'));
    if (nextText.trim() === '' || !Number.isFinite(amount) || amount < 0) {
      props.ctx.onDraft(nextText.trim() === '' ? '' : 'NaN');
      return;
    }
    lastWritten = toBytes(amount, nextUnit);
    props.ctx.onDraft(String(lastWritten));
  }
  return (
    <div class={css.byteSize} data-testid="attachment-max-bytes">
      <input id={props.ctx.wiring.id} type="text" inputmode="decimal" value={text()} disabled={props.ctx.disabled} aria-describedby={props.ctx.wiring.describedBy} aria-invalid={props.ctx.wiring.invalid} onInput={(e) => commit(e.currentTarget.value, unit())} />
      <select aria-label={t('admin-attachments-bytes-unit')} value={unit()} disabled={props.ctx.disabled} onChange={(e) => commit(text(), e.currentTarget.value as ByteUnit)}>
        <For each={BYTE_UNITS}>{(u) => <option value={u} selected={u === unit()}>{u}</option>}</For>
      </select>
      <span class={css.muted} role="status">
        <Show when={bytes() > 0} fallback={t('admin-attachments-unlimited')}>{t('admin-attachments-bytes-equals', { size: formatBytes(bytes()), bytes: formatNumber(bytes()) })}</Show>
      </span>
    </div>
  );
}
