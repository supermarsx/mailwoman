// Generic themed combobox: a text input that filters a grouped listbox. Pinned rows (no
// group) come first, free text can be allowed, and every colour comes from theme tokens
// (the native <select> popup cannot be themed per option, this one can).

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../i18n';
import * as css from './combobox.css.ts';

export interface ComboboxOption {
  value: string;
  label: string;
  /** Secondary text on the row (for example the UTC offset and local time). */
  detail?: string;
  /** Extra searchable text that is not shown. */
  keywords?: string;
  /** What the input shows once this row is chosen (defaults to the label). */
  text?: string | undefined;
  /** Heading the row is listed under; rows without one come first, unlabelled. */
  group?: string;
}

const norm = (s: string): string => s.toLocaleLowerCase().replaceAll('_', ' ').replaceAll('/', ' ').replace(/\s+/g, ' ').trim();

/** Rows matching every word of `query` against value, label, detail and keywords. */
export function filterComboboxOptions(options: readonly ComboboxOption[], query: string): ComboboxOption[] {
  const words = norm(query).split(' ').filter(Boolean);
  if (words.length === 0) return [...options];
  return options.filter((o) => {
    const hay = norm(`${o.value} ${o.label} ${o.detail ?? ''} ${o.keywords ?? ''}`);
    return words.every((w) => hay.includes(w));
  });
}

export function Combobox(props: {
  label: string;
  value: string;
  options: readonly ComboboxOption[];
  onChange: (value: string) => void;
  /** Keep what is typed as the value (the caller validates it). */
  freeText?: boolean | undefined;
  disabled?: boolean | undefined;
  describedBy?: string | undefined;
  id?: string | undefined;
  maxLength?: number | undefined;
}): JSX.Element {
  const uid = createUniqueId();
  const id = props.id ?? `cb-${uid}`;
  const [open, setOpen] = createSignal(false);
  const [active, setActive] = createSignal(0);
  // null while the user is not typing: the field then shows the chosen row's label.
  const [query, setQuery] = createSignal<string | null>(null);
  const shown = () => {
    if (query() !== null) return query()!;
    const row = props.options.find((o) => o.value === props.value);
    return row ? (row.text ?? row.label) : props.value;
  };
  const matches = createMemo(() => filterComboboxOptions(props.options, query() ?? ''));
  const sections = createMemo(() => {
    const out: { group: string; items: { option: ComboboxOption; index: number }[] }[] = [];
    matches().forEach((option, index) => {
      const group = option.group ?? '';
      let section = out[out.length - 1];
      if (!section || section.group !== group) { section = { group, items: [] }; out.push(section); }
      section.items.push({ option, index });
    });
    return out;
  });
  const optionId = (index: number) => `${id}-o-${index}`;
  const activeId = () => (open() && matches()[active()] ? optionId(active()) : undefined);
  const close = () => { setOpen(false); setQuery(null); };
  const choose = (value: string) => { props.onChange(value); setQuery(null); setOpen(false); };
  const move = (to: number) => {
    const n = matches().length;
    if (n === 0) return;
    setOpen(true);
    setActive(Math.max(0, Math.min(n - 1, to)));
    queueMicrotask(() => document.getElementById(optionId(active()))?.scrollIntoView?.({ block: 'nearest' }));
  };
  const onKeyDown = (event: KeyboardEvent): void => {
    if (props.disabled) return;
    if (event.key === 'ArrowDown') { event.preventDefault(); move(open() ? active() + 1 : active()); }
    else if (event.key === 'ArrowUp') { event.preventDefault(); move(open() ? active() - 1 : active()); }
    else if (event.key === 'PageDown') { event.preventDefault(); move(active() + 8); }
    else if (event.key === 'PageUp') { event.preventDefault(); move(active() - 8); }
    else if (event.key === 'Home' && open()) { event.preventDefault(); move(0); }
    else if (event.key === 'End' && open()) { event.preventDefault(); move(matches().length - 1); }
    else if (event.key === 'Escape' && open()) { event.preventDefault(); event.stopPropagation(); close(); }
    else if (event.key === 'Enter' && open() && matches()[active()]) { event.preventDefault(); choose(matches()[active()]!.value); }
  };
  const openList = () => {
    const at = props.options.findIndex((o) => o.value === props.value);
    setActive(Math.max(0, at));
    setOpen(true);
  };
  return (
    <div class={css.root}>
      <label for={id}>{props.label}</label>
      <div class={css.field}>
        <input
          id={id} type="text" role="combobox" class={css.input} autocomplete="off" autocapitalize="none" spellcheck={false}
          aria-autocomplete="list" aria-expanded={open()} aria-controls={`${id}-list`} aria-activedescendant={activeId()}
          aria-describedby={props.describedBy} disabled={props.disabled} maxlength={props.maxLength ?? 64}
          value={shown()}
          onFocus={(e) => { openList(); e.currentTarget.select(); }}
          onBlur={close}
          onInput={(e) => {
            setQuery(e.currentTarget.value);
            if (props.freeText) props.onChange(e.currentTarget.value);
            setActive(0); setOpen(true);
          }}
          onKeyDown={onKeyDown}
        />
        <button
          type="button" tabindex="-1" class={css.toggle} disabled={props.disabled} aria-label={t('combobox-toggle', { label: props.label })} aria-expanded={open()} aria-controls={`${id}-list`}
          onPointerDown={(e) => e.preventDefault()}
          onClick={() => { if (open()) close(); else { setQuery(null); openList(); document.getElementById(id)?.focus(); } }}
        >
          <span aria-hidden="true">▾</span>
        </button>
        <Show when={open() && !props.disabled}>
          <div id={`${id}-list`} role="listbox" aria-label={props.label} class={css.list}>
            <Show when={matches().length > 0} fallback={<div class={css.empty} role="status">{t('combobox-empty')}</div>}>
              <For each={sections()}>{(section, si) => (
                <div role="group" aria-labelledby={section.group ? `${id}-g-${si()}` : undefined}>
                  <Show when={section.group}><div id={`${id}-g-${si()}`} class={css.groupLabel}>{section.group}</div></Show>
                  <For each={section.items}>{({ option, index }) => (
                    <div
                      id={optionId(index)} role="option" class={css.option} aria-selected={option.value === props.value} data-active={index === active() ? 'true' : 'false'}
                      onPointerDown={(e) => e.preventDefault()} onPointerMove={() => { if (active() !== index) setActive(index); }} onClick={() => choose(option.value)}
                    >
                      <span>{option.label}</span>
                      <Show when={option.detail}><span class={css.detail}>{option.detail}</span></Show>
                    </div>
                  )}</For>
                </div>
              )}</For>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
}
