// MultiSelect (t39 L15): pick several values from a list. Selected values are
// chips (each with its own remove button); the text field filters the options
// as you type (typeahead) and is an ARIA 1.2 combobox over a multi-selectable
// listbox.
//
// Keyboard: ArrowDown/ArrowUp open the list and move the active option,
// Home/End jump, Enter toggles the active option, Escape closes the list (or
// clears the filter when it is already closed), Backspace in an empty field
// removes the last chip, Tab leaves. Focus stays in the field; the active
// option is announced through aria-activedescendant. A polite status says how
// many values are selected after each change.
import { createMemo, createSignal, createUniqueId, For, onCleanup, Show, type JSX } from 'solid-js';
import { Icon } from './Icon.tsx';
import { t } from '../i18n';
import * as css from './multiSelect.css.ts';

export interface MultiSelectOption { value: string; label: string; detail?: string }

export interface MultiSelectProps {
  /** The visible label's id, or use `label` for an accessible name only. */
  labelledBy?: string;
  label?: string;
  options: readonly MultiSelectOption[];
  selected: readonly string[];
  onChange: (values: string[]) => void;
  placeholder?: string;
  disabled?: boolean;
  describedBy?: string;
  /** The `id` of the text field, for a `<label for>`. */
  id?: string;
  testId?: string;
}

const fold = (text: string): string => text.normalize('NFKD').replace(/\p{M}/gu, '').toLocaleLowerCase();

export function MultiSelect(props: MultiSelectProps): JSX.Element {
  const uid = createUniqueId();
  const inputId = () => props.id ?? `ms-${uid}-input`;
  const listId = `ms-${uid}-list`;
  const statusId = `ms-${uid}-status`;
  const [query, setQuery] = createSignal('');
  const [open, setOpen] = createSignal(false);
  const [active, setActive] = createSignal(0);
  const [announce, setAnnounce] = createSignal('');
  let input!: HTMLInputElement;
  let root!: HTMLDivElement;

  const chosen = createMemo(() => new Set(props.selected));
  const byValue = createMemo(() => new Map(props.options.map(option => [option.value, option])));
  const chips = createMemo(() => props.selected.map(value => byValue().get(value) ?? { value, label: value }));
  const visible = createMemo(() => {
    const q = fold(query().trim());
    if (!q) return [...props.options];
    // Options that start with the typed text come first, then other matches.
    const starts: MultiSelectOption[] = []; const contains: MultiSelectOption[] = [];
    for (const option of props.options) {
      const label = fold(option.label); const detail = fold(option.detail ?? '');
      if (label.startsWith(q) || label.split(/\s+/).some(word => word.startsWith(q))) starts.push(option);
      else if (label.includes(q) || detail.includes(q)) contains.push(option);
    }
    return [...starts, ...contains];
  });
  const activeOption = () => (open() ? visible()[Math.min(active(), visible().length - 1)] : undefined);
  const optionId = (value: string) => `ms-${uid}-opt-${props.options.findIndex(option => option.value === value)}`;

  function commit(values: string[]): void {
    props.onChange(values);
    setAnnounce(t('multiselect-count', { count: values.length }));
  }
  function toggle(value: string): void {
    if (props.disabled) return;
    const adding = !chosen().has(value);
    commit(adding ? [...props.selected, value] : props.selected.filter(v => v !== value));
    // A picked value ends the search: the filter clears and the full list returns.
    if (adding && query()) { setQuery(''); setActive(0); }
  }
  function remove(value: string): void {
    if (props.disabled) return;
    commit(props.selected.filter(v => v !== value));
    input.focus();
  }
  function show(index = active()): void {
    if (props.disabled) return;
    setOpen(true);
    setActive(Math.max(0, Math.min(index, visible().length - 1)));
    queueMicrotask(() => document.getElementById(activeOption() ? optionId(activeOption()!.value) : '')?.scrollIntoView?.({ block: 'nearest' }));
  }
  function onKeyDown(event: KeyboardEvent): void {
    if (props.disabled) return;
    const count = visible().length;
    switch (event.key) {
      case 'ArrowDown': event.preventDefault(); show(open() ? (active() + 1) % Math.max(count, 1) : 0); break;
      case 'ArrowUp': event.preventDefault(); show(open() ? (active() - 1 + count) % Math.max(count, 1) : count - 1); break;
      case 'Home': if (open()) { event.preventDefault(); show(0); } break;
      case 'End': if (open()) { event.preventDefault(); show(count - 1); } break;
      case 'Enter': { const option = activeOption(); if (option) { event.preventDefault(); toggle(option.value); } break; }
      case 'Escape': if (open()) { event.preventDefault(); event.stopPropagation(); setOpen(false); } else if (query()) { event.preventDefault(); setQuery(''); } break;
      case 'Backspace': if (!query() && props.selected.length > 0) { event.preventDefault(); remove(props.selected[props.selected.length - 1]!); } break;
      case 'Tab': setOpen(false); break;
    }
  }
  const outside = (event: PointerEvent) => { if (!root.contains(event.target as Node)) setOpen(false); };
  document.addEventListener('pointerdown', outside);
  onCleanup(() => document.removeEventListener('pointerdown', outside));

  return <div ref={root} class={css.root} data-testid={props.testId}>
    <div class={css.field} data-disabled={props.disabled ? '' : undefined} onClick={event => { if (event.target === event.currentTarget) input.focus(); }}>
      <ul class={css.chips} aria-label={t('multiselect-selected', { label: props.label ?? (props.labelledBy ? document.getElementById(props.labelledBy)?.textContent ?? '' : '') })}>
        <For each={chips()}>{chip => <li class={css.chip}>
          <span class={css.chipLabel}><bdi>{chip.label}</bdi></span>
          <button type="button" class={css.chipRemove} disabled={props.disabled} aria-label={t('multiselect-remove', { label: chip.label })} onClick={() => remove(chip.value)}><Icon name="close" /></button>
        </li>}</For>
      </ul>
      <input ref={input} id={inputId()} class={css.input} type="text" role="combobox" autocomplete="off" spellcheck={false}
        aria-label={props.labelledBy ? undefined : props.label} aria-labelledby={props.labelledBy}
        aria-describedby={props.describedBy}
        aria-expanded={open()} aria-controls={listId} aria-autocomplete="list" aria-haspopup="listbox"
        aria-activedescendant={activeOption() ? optionId(activeOption()!.value) : undefined}
        placeholder={props.placeholder} disabled={props.disabled} value={query()}
        onInput={event => { setQuery(event.currentTarget.value); show(0); }}
        on:keydown={onKeyDown} onClick={() => (open() ? setOpen(false) : show(0))}
        onBlur={event => { if (!root.contains(event.relatedTarget as Node | null)) setOpen(false); }} />
    </div>
    <Show when={open()}>
      <ul id={listId} class={css.listbox} role="listbox" aria-multiselectable="true" aria-label={props.label} aria-labelledby={props.labelledBy}>
        <For each={visible()} fallback={<li class={css.empty} role="presentation">{t('multiselect-no-match')}</li>}>{(option, index) =>
          <li id={optionId(option.value)} role="option" class={css.option} aria-selected={chosen().has(option.value)} data-active={activeOption()?.value === option.value ? '' : undefined}
            onPointerDown={event => event.preventDefault()} onPointerMove={() => setActive(index())} onClick={() => { toggle(option.value); input.focus(); }}>
            <span class={css.tick} aria-hidden="true">{chosen().has(option.value) ? '✓' : ''}</span>
            <span class={css.optionText}><bdi>{option.label}</bdi><Show when={option.detail}><span class={css.optionDetail}><bdi>{option.detail}</bdi></span></Show></span>
          </li>}</For>
      </ul>
    </Show>
    <p id={statusId} class={css.srOnly} role="status" aria-live="polite">{announce()}</p>
  </div>;
}
