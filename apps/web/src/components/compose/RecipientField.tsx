import { createEffect, createSignal, For, on, onCleanup, Show, type JSX } from 'solid-js';
import { isolate, t } from '../../i18n/index.ts';
import { Icon } from '../Icon.tsx';
import { entriesOf, isCompleteRecipient, recipientOf, serializeRecipients, splitPastedEntries, type Recipient, type RecipientSeparator } from './recipientChips.ts';
import './RecipientField.css';

/**
 * To/Cc/Bcc as one token field: finished recipients are chips inside the same box as
 * the text input, flowing with the caret. The value is still the serialised address
 * list (chips in the configured separator, then whatever is still being typed), so
 * drafts, replies and the pre-send checks keep working on the same data.
 */
export interface RecipientFieldProps {
  /** `to`, `cc` or `bcc`: used for test ids. */
  field: string;
  /** Accessible name of the group, e.g. "Recipients in To". */
  groupLabel: string;
  value: string;
  /** Called with the serialised value and the caret inside it. */
  onChange: (value: string, caret: number) => void;
  separator: RecipientSeparator;
  /** Off: a plain text field. */
  autoChip: boolean;
  placeholder?: string;
  /** Contact name of an address, when known. */
  nameOf?: (email: string) => string | null;
  /** Combobox attributes for the inner input (role, aria-*); getters are fine. */
  inputAttrs?: JSX.InputHTMLAttributes<HTMLInputElement>;
  inputRef?: (input: HTMLInputElement) => void;
  /** Runs first on the input's keydown; a prevented event is left alone. */
  onInputKeyDown?: (event: KeyboardEvent & { currentTarget: HTMLInputElement }) => void;
  onInputFocus?: () => void;
  onInputBlur?: (event: FocusEvent & { currentTarget: HTMLInputElement }) => void;
  onInputClick?: (event: MouseEvent & { currentTarget: HTMLInputElement }) => void;
  onInputKeyUp?: (event: KeyboardEvent & { currentTarget: HTMLInputElement }) => void;
  onCompositionStart?: () => void;
  onCompositionEnd?: (event: CompositionEvent & { currentTarget: HTMLInputElement }) => void;
  isComposing?: () => boolean;
  /** Open the contact card of a chip (click or Enter only). */
  openCard?: (address: string, name: string, anchor: HTMLElement) => void;
  closeCard?: () => void;
  /** Suggestion list, rendered under the box. */
  children?: JSX.Element;
}

interface Chip extends Recipient { id: number }
let chipSeq = 0;
const toChip = (text: string): Chip => ({ ...recipientOf(text), id: ++chipSeq });

export function RecipientField(props: RecipientFieldProps): JSX.Element {
  const [chips, setChips] = createSignal<Chip[]>([]);
  const [draft, setDraft] = createSignal('');
  const [selected, setSelected] = createSignal<ReadonlySet<number>>(new Set());
  const [live, setLive] = createSignal('');
  const buttons = new Map<number, HTMLButtonElement>();
  let box!: HTMLDivElement;
  let input!: HTMLInputElement;
  let emitted: string | undefined;
  let prevAuto: boolean | undefined;
  let touched = false;

  const serialize = () => serializeRecipients(chips(), draft(), props.separator);
  function emit(caret?: number): void {
    const value = props.autoChip ? serialize() : draft();
    emitted = value;
    props.onChange(value, caret ?? value.length);
  }
  const announce = (message: string) => { setLive(''); queueMicrotask(() => setLive(message)); };
  const labelOf = (chip: Chip): string => (chip.valid ? chip.name ?? props.nameOf?.(chip.email) ?? chip.text : chip.text);

  // An outside change (reply prefill, restored draft, suggestion pick, group expand) is
  // parsed into chips; our own emissions are ignored.
  createEffect(on([() => props.value, () => props.autoChip], ([value, auto]) => {
    if (value === emitted && auto === prevAuto) return;
    prevAuto = auto;
    emitted = value;
    setSelected(new Set<number>());
    if (!auto) { setChips([]); setDraft(value); return; }
    setChips(entriesOf(value).map(toChip));
    setDraft('');
  }));

  function addChips(texts: string[], onlyValid: boolean): boolean {
    const made = texts.map(toChip).filter(chip => !onlyValid || chip.valid);
    if (made.length === 0) return false;
    setChips(list => [...list, ...made]);
    announce(made.map(chip => t('mail-recipient-added', { name: isolate(labelOf(chip)) })).join('. '));
    return true;
  }

  /** Turn the typed text into chips. `force` keeps invalid text too, as invalid chips. */
  function commitDraft(force: boolean): boolean {
    const text = draft();
    if (text.trim() === '') {
      if (text !== '') { setDraft(''); input.value = ''; emit(); }
      return false;
    }
    const entries = entriesOf(text);
    if (!force && !entries.every(isCompleteRecipient)) return false;
    addChips(entries, false);
    setDraft(''); input.value = '';
    emit();
    return true;
  }

  function removeChips(gone: Chip[], focusAfter: 'neighbour' | 'input'): void {
    props.closeCard?.();
    const ids = new Set(gone.map(chip => chip.id));
    const before = chips();
    const first = before.findIndex(chip => ids.has(chip.id));
    const rest = before.filter(chip => !ids.has(chip.id));
    setChips(rest); setSelected(new Set<number>());
    emit();
    announce(gone.length === 1 ? t('mail-recipient-removed', { name: isolate(labelOf(gone[0]!)) }) : t('mail-recipient-removed-many', { count: gone.length }));
    const next = focusAfter === 'neighbour' ? rest[Math.min(first, rest.length - 1)] : undefined;
    queueMicrotask(() => { const target = next ? buttons.get(next.id) : undefined; (target ?? input).focus(); });
  }

  function editChip(chip: Chip): void {
    props.closeCard?.();
    const pending = draft();
    const rest = chips().filter(other => other.id !== chip.id);
    if (pending.trim() !== '') rest.push(...entriesOf(pending).map(toChip));
    setChips(rest); setSelected(new Set<number>());
    setDraft(chip.text); input.value = chip.text;
    emit();
    input.focus();
    input.setSelectionRange(chip.text.length, chip.text.length);
    announce(t('mail-recipient-editing', { name: isolate(labelOf(chip)) }));
  }

  const focusChip = (chip: Chip | undefined): void => { if (chip) buttons.get(chip.id)?.focus(); else input.focus(); };
  const rtl = (): boolean => (box.ownerDocument.defaultView?.getComputedStyle(box).direction ?? 'ltr') === 'rtl';
  function selectAll(): void {
    const list = chips();
    if (list.length === 0) return;
    setSelected(new Set(list.map(chip => chip.id)));
    announce(t('mail-recipient-all-selected', { count: list.length }));
    focusChip(list[list.length - 1]);
  }

  function onInputInput(event: InputEvent & { currentTarget: HTMLInputElement }): void {
    const el = event.currentTarget;
    touched = true;
    const value = el.value;
    const caret = el.selectionStart ?? value.length;
    if (!props.autoChip) { setDraft(value); emit(caret); return; }
    const typed = value[caret - 1];
    if (!props.isComposing?.() && (typed === ' ' || typed === ',' || typed === ';')) {
      const entries = entriesOf(value.slice(0, caret - 1));
      if (entries.length > 0 && entries.every(isCompleteRecipient)) {
        addChips(entries, true);
        const tail = value.slice(caret).replace(/^[\s,;]+/u, '');
        setDraft(tail); el.value = tail;
        emit();
        el.setSelectionRange(0, 0);
        return;
      }
    }
    setDraft(value);
    emit();
  }

  function onInputPaste(event: ClipboardEvent & { currentTarget: HTMLInputElement }): void {
    if (!props.autoChip) return;
    const entries = splitPastedEntries(event.clipboardData?.getData('text') ?? '');
    const single = entries.length === 1 && isCompleteRecipient(entries[0]!) && draft().trim() === '';
    if (entries.length < 2 && !single) return;
    event.preventDefault();
    touched = true;
    const pending = draft();
    if (pending.trim() !== '') addChips(entriesOf(pending), false);
    addChips(entries, false);
    setDraft(''); input.value = '';
    emit();
  }

  function onInputKeyDown(event: KeyboardEvent & { currentTarget: HTMLInputElement }): void {
    props.onInputKeyDown?.(event);
    if (event.defaultPrevented) return;
    if (event.isComposing || props.isComposing?.() || event.keyCode === 229) { event.stopPropagation(); return; }
    if (!props.autoChip) return;
    const el = event.currentTarget;
    const atStart = el.selectionStart === 0 && el.selectionEnd === 0;
    const list = chips();
    if (event.key === 'Enter') {
      event.preventDefault(); event.stopPropagation();
      commitDraft(true);
    } else if (event.key === 'Tab') {
      if (draft().trim() !== '') commitDraft(true);
    } else if (event.key === 'Backspace' && draft() === '' && list.length > 0) {
      event.preventDefault();
      editChip(list[list.length - 1]!);
    } else if (atStart && event.key === (rtl() ? 'ArrowRight' : 'ArrowLeft') && list.length > 0) {
      event.preventDefault();
      focusChip(list[list.length - 1]);
    } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a' && draft() === '' && list.length > 0) {
      event.preventDefault();
      selectAll();
    }
  }

  function onChipKeyDown(chip: Chip, event: KeyboardEvent): void {
    const list = chips();
    const index = list.findIndex(other => other.id === chip.id);
    const back = rtl() ? 'ArrowRight' : 'ArrowLeft';
    const forward = rtl() ? 'ArrowLeft' : 'ArrowRight';
    const targets = selected().has(chip.id) ? list.filter(other => selected().has(other.id)) : [chip];
    if (event.key === back) { event.preventDefault(); setSelected(new Set<number>()); focusChip(list[Math.max(0, index - 1)]); }
    else if (event.key === forward) { event.preventDefault(); setSelected(new Set<number>()); focusChip(list[index + 1]); }
    else if (event.key === 'Home') { event.preventDefault(); focusChip(list[0]); }
    else if (event.key === 'End') { event.preventDefault(); focusChip(undefined); }
    else if (event.key === 'Delete') { event.preventDefault(); removeChips(targets, 'neighbour'); }
    else if (event.key === 'Backspace') { event.preventDefault(); if (targets.length > 1) removeChips(targets, 'input'); else editChip(chip); }
    else if (event.key === 'F2') { event.preventDefault(); editChip(chip); }
    else if (event.key === 'Escape') { event.preventDefault(); event.stopPropagation(); setSelected(new Set<number>()); input.focus(); }
    else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') { event.preventDefault(); selectAll(); }
    else if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey && event.key !== ' ') { setSelected(new Set<number>()); input.focus(); }
  }

  function onChipClick(chip: Chip, event: MouseEvent): void {
    event.preventDefault(); event.stopPropagation();
    if (!chip.valid) { editChip(chip); return; }
    props.openCard?.(chip.email, chip.name ?? props.nameOf?.(chip.email) ?? '', event.currentTarget as HTMLElement);
  }

  function onBoxFocusOut(event: FocusEvent): void {
    const next = event.relatedTarget as Element | null;
    if (!next || box.contains(next)) return;
    if (typeof next.closest === 'function' && next.closest('.address-card-backdrop, [role="listbox"]')) return;
    setSelected(new Set<number>());
    props.closeCard?.();
  }
  onCleanup(() => props.closeCard?.());

  const chipName = (chip: Chip): string => chip.valid
    ? t('mail-recipient-chip-name', { name: isolate(labelOf(chip)), address: isolate(chip.email) })
    : t('mail-recipient-chip-name-invalid', { address: isolate(chip.text) });

  return <div class="compose-rf" data-testid={`compose-rf-${props.field}`}>
    <div ref={box} class="compose-rf__box" role="group" aria-label={props.groupLabel} data-chips={props.autoChip ? 'on' : 'off'}
      onClick={event => { if (event.target === box || (event.target as HTMLElement).classList.contains('compose-rf__chips')) input.focus(); }}
      onFocusOut={onBoxFocusOut}>
      <Show when={props.autoChip && chips().length > 0}>
        <ul class="compose-rf__chips" role="list" data-testid={`compose-chips-${props.field}`}>
          <For each={chips()}>{chip =>
            <li role="listitem" class={`compose-rf__chip${chip.valid ? '' : ' compose-rf__chip--invalid'}${selected().has(chip.id) ? ' compose-rf__chip--selected' : ''}`}
              data-recipient={chip.text} data-valid={chip.valid ? 'true' : 'false'} data-selected={selected().has(chip.id) ? 'true' : 'false'}>
              <button type="button" class="compose-rf__chip-main" tabindex={-1} ref={el => { buttons.set(chip.id, el); onCleanup(() => buttons.delete(chip.id)); }}
                aria-haspopup={chip.valid && props.openCard ? 'dialog' : undefined}
                aria-label={chipName(chip)}
                title={chip.valid ? chip.email : t('mail-recipient-invalid-title')}
                onClick={event => onChipClick(chip, event)}
                onDblClick={event => { event.preventDefault(); editChip(chip); }}
                onKeyDown={event => onChipKeyDown(chip, event)}>
                <Show when={!chip.valid}><Icon name="alert"/></Show>
                <span class="compose-rf__chip-label">{isolate(labelOf(chip))}</span>
              </button>
              <button type="button" class="compose-rf__chip-remove" tabindex={-1} aria-label={t('mail-recipient-chip-remove', { address: isolate(chip.text) })}
                onClick={event => { event.stopPropagation(); removeChips([chip], 'input'); }}>
                <span aria-hidden="true">×</span>
              </button>
            </li>}</For>
        </ul>
      </Show>
      <input ref={el => { input = el; props.inputRef?.(el); }} type="text" class="compose-rf__input"
        autocomplete="off" autocapitalize="off" spellcheck={false}
        {...props.inputAttrs}
        placeholder={chips().length === 0 ? props.placeholder : undefined}
        value={draft()}
        onInput={onInputInput}
        onPaste={onInputPaste}
        onKeyDown={onInputKeyDown}
        onKeyUp={event => props.onInputKeyUp?.(event)}
        onClick={event => props.onInputClick?.(event)}
        onFocus={() => { setSelected(new Set<number>()); props.onInputFocus?.(); }}
        onCompositionStart={() => props.onCompositionStart?.()}
        onCompositionEnd={event => props.onCompositionEnd?.(event)}
        onBlur={event => {
          props.onInputBlur?.(event);
          const next = event.relatedTarget as Element | null;
          if (props.autoChip && touched && !props.isComposing?.() && !(next && typeof next.closest === 'function' && next.closest('[role="listbox"]'))) commitDraft(true);
        }}/>
    </div>
    <span class="compose-rf__live" role="status" aria-live="polite">{live()}</span>
    {props.children}
  </div>;
}
