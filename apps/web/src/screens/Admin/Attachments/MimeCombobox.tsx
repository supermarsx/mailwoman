// Editable combobox for a MIME type: grouped common types to pick from, and any
// custom type/subtype typed freely. The typed text is always the value; the list
// only suggests, so it can never block a test.

import { For, Show, createMemo, createSignal, createUniqueId, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import { filterMimeGroups, isMimeShape, type MimeEntry } from './mimeTypes.ts';
import * as css from './mimeCombobox.css.ts';

export function MimeCombobox(props: { value: string; onChange: (value: string) => void }): JSX.Element {
  const id = createUniqueId();
  const [open, setOpen] = createSignal(false);
  const [active, setActive] = createSignal(-1);
  // Filter only what the user typed; after a pick or on open the whole list shows.
  const [filtering, setFiltering] = createSignal(false);
  const groups = createMemo(() => filterMimeGroups(filtering() ? props.value : ''));
  const flat = createMemo<MimeEntry[]>(() => groups().flatMap((g) => [...g.entries]));
  const invalid = () => props.value.trim() !== '' && !isMimeShape(props.value);
  const optionId = (value: string) => `${id}-o-${value.replace(/[^a-z0-9]/gi, '_')}`;
  const activeId = () => (open() && active() >= 0 && flat()[active()] ? optionId(flat()[active()]!.value) : undefined);

  const choose = (value: string): void => {
    props.onChange(value);
    setFiltering(false);
    setOpen(false);
    setActive(-1);
  };
  const move = (delta: number): void => {
    const n = flat().length;
    if (n === 0) return;
    setOpen(true);
    setActive((i) => (i < 0 ? (delta > 0 ? 0 : n - 1) : Math.max(0, Math.min(n - 1, i + delta))));
    queueMicrotask(() => document.getElementById(activeId() ?? '')?.scrollIntoView?.({ block: 'nearest' }));
  };
  const onKeyDown = (event: KeyboardEvent): void => {
    if (event.key === 'ArrowDown') { event.preventDefault(); move(1); }
    else if (event.key === 'ArrowUp') { event.preventDefault(); move(-1); }
    else if (event.key === 'Escape' && open()) { event.preventDefault(); event.stopPropagation(); setOpen(false); setActive(-1); }
    else if (event.key === 'Enter' && open() && active() >= 0 && flat()[active()]) { event.preventDefault(); choose(flat()[active()]!.value); }
  };

  return (
    <div class={css.root}>
      <label for={`${id}-input`}>{t('admin-attachments-sim-mime')}</label>
      <div class={css.field}>
        <input
          id={`${id}-input`} type="text" role="combobox" class={css.input} autocomplete="off" autocapitalize="none" spellcheck={false}
          aria-autocomplete="list" aria-expanded={open()} aria-controls={`${id}-list`} aria-activedescendant={activeId()}
          aria-invalid={invalid() ? 'true' : undefined} aria-describedby={`${id}-hint`}
          value={props.value}
          onFocus={() => setOpen(true)}
          onBlur={() => { setOpen(false); setActive(-1); }}
          onInput={(e) => { props.onChange(e.currentTarget.value); setFiltering(true); setActive(-1); setOpen(true); }}
          onKeyDown={onKeyDown}
        />
        <button
          type="button" tabindex="-1" class={css.toggle} aria-label={t('admin-attachments-mime-toggle')} aria-expanded={open()} aria-controls={`${id}-list`}
          onPointerDown={(e) => e.preventDefault()}
          onClick={() => { setFiltering(false); setOpen(!open()); }}
        >
          <span aria-hidden="true">▾</span>
        </button>
        <Show when={open() && flat().length > 0}>
          <div id={`${id}-list`} role="listbox" aria-label={t('admin-attachments-sim-mime')} class={css.list}>
            <For each={groups()}>{(group) => (
              <div role="group" aria-labelledby={`${id}-g-${group.id}`}>
                <div id={`${id}-g-${group.id}`} class={css.groupLabel}>{t(`admin-attachments-mime-group-${group.id}`)}</div>
                <For each={group.entries}>{(entry) => (
                  <div
                    id={optionId(entry.value)} role="option" class={css.option} aria-selected={flat()[active()]?.value === entry.value}
                    onPointerDown={(e) => e.preventDefault()} onClick={() => choose(entry.value)}
                  >
                    <span>{entry.label}</span><code class={css.code}>{entry.value}</code>
                  </div>
                )}</For>
              </div>
            )}</For>
          </div>
        </Show>
      </div>
      <p id={`${id}-hint`} class={invalid() ? css.hintInvalid : css.hint} role={invalid() ? 'status' : undefined}>
        {invalid() ? t('admin-attachments-mime-invalid') : t('admin-attachments-mime-hint')}
      </p>
    </div>
  );
}
