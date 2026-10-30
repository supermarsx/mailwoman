// The filter chips under the top-bar search input (t88). The chips are a view of
// `parseQuery(text)`; every edit, toggle or removal builds the next query and
// hands its canonical text (`serializeQuery`) to `onChange`, which writes it
// into the input and runs the search. Nothing here keeps its own copy of the
// filters, so the input and the chips cannot disagree.

import { createMemo, createSignal, For, Index, onCleanup, onMount, Show, type Accessor, type JSX } from 'solid-js';
import { Portal } from 'solid-js/web';
import { t } from '../../i18n/index.ts';
import { Icon, type IconName } from '../Icon.tsx';
import { createContextMenu, type ContextAction } from '../ContextMenu.tsx';
import { placePopover } from '../color/placement.ts';
import * as a11y from '../mailA11y.css.ts';
import * as css from './searchBar.css.ts';
import {
  addTerm, chipsOf, isDateValue, isSizeValue, parseQuery, removeTerms, replaceTerm, resolveScope, serializeQuery,
  serializeTerm, setRange, setScope, toggleTerm, valueIn, type Chip, type ChipKind, type Operator, type ScopeMailbox,
  type SearchQuery, type Term,
} from '../../search/searchQuery.ts';

export interface SearchFilterBarProps {
  /** The input's current text (the chips follow it as it is typed). */
  text: Accessor<string>;
  /** Write this text into the input and search with it. */
  onChange: (text: string) => void;
  mailboxes: Accessor<readonly ScopeMailbox[]>;
  currentMailboxId: Accessor<string | null>;
}

/** Text-valued chips and the operator each writes. */
const TEXT_OPS: Partial<Record<ChipKind, Operator>> = {
  from: 'from', to: 'to', cc: 'cc', subject: 'subject', body: 'body', filename: 'filename', tag: 'tag',
};

const FIELD_LABEL: Partial<Record<ChipKind, string>> = {
  from: 'search-field-from', to: 'search-field-to', cc: 'search-field-cc', subject: 'search-field-subject',
  body: 'search-field-body', text: 'search-field-text', filename: 'search-field-filename', tag: 'search-field-tag',
  date: 'search-field-date', size: 'search-field-size', scope: 'search-field-scope', expr: 'search-field-expression',
  other: 'search-field-other',
};

const CHIP_ICON: Partial<Record<ChipKind, IconName>> = {
  attachment: 'attachment', event: 'calendar', unread: 'mailRead', flagged: 'flag', pinned: 'pin', date: 'clock',
  scope: 'folder', from: 'users', to: 'sent', subject: 'mail', body: 'file', text: 'search', size: 'archive',
};

const TOGGLES: readonly ChipKind[] = ['attachment', 'event', 'unread', 'flagged', 'pinned'];

/** What the popover is editing: an existing chip, or a new one of `kind`. */
interface Editing { kind: ChipKind; chip: Chip | null; anchor: HTMLElement }

function termAt(q: SearchQuery, chip: Chip): Term | undefined {
  return q.terms[chip.terms[0]!];
}

/** The two-state label of a toggle chip. */
function toggleLabel(kind: ChipKind, term: Term | undefined): string {
  const negated = term?.kind === 'field' && term.negated;
  const value = term?.kind === 'field' ? term.value : '';
  switch (kind) {
    case 'attachment': return t(negated ? 'search-chip-no-attachment' : 'search-chip-attachment');
    case 'event': return t(negated ? 'search-chip-no-event' : 'search-chip-event');
    case 'unread': return t(value === 'read' || negated ? 'search-chip-read' : 'search-chip-unread');
    case 'flagged': return t(value === 'unflagged' || negated ? 'search-chip-unflagged' : 'search-chip-flagged');
    default: return t(negated || (term?.kind === 'field' && term.op === 'pinned' && /^(false|0)$/iu.test(value)) ? 'search-chip-unpinned' : 'search-chip-pinned');
  }
}

export function SearchFilterBar(props: SearchFilterBarProps): JSX.Element {
  const query = createMemo(() => parseQuery(props.text()));
  const scopeChip = createMemo(() => chipsOf(query()).find((c) => c.kind === 'scope') ?? null);
  // The scope chip is always shown (first), even with no `in:` (= all folders).
  const chips = createMemo(() => chipsOf(query()).filter((c) => c.kind !== 'scope'));
  const [editing, setEditing] = createSignal<Editing | null>(null);
  let addButton!: HTMLButtonElement;

  const commit = (next: SearchQuery): void => {
    setEditing(null);
    props.onChange(serializeQuery(next));
  };

  const scopeName = (): string => {
    const chip = scopeChip();
    if (chip === null) return t('search-scope-all');
    const value = valueIn(query(), chip.terms, 'in');
    return resolveScope(value, props.mailboxes())?.name ?? value;
  };

  const chipValue = (chip: Chip): string => {
    const q = query();
    const term = termAt(q, chip);
    if (chip.kind === 'date') {
      const after = valueIn(q, chip.terms, 'after');
      const before = valueIn(q, chip.terms, 'before');
      if (after && before) return t('search-date-between', { after, before });
      return after ? t('search-date-since', { date: after }) : t('search-date-until', { date: before });
    }
    if (chip.kind === 'size') {
      const larger = valueIn(q, chip.terms, 'larger');
      const smaller = valueIn(q, chip.terms, 'smaller');
      if (larger && smaller) return t('search-size-between', { larger, smaller });
      return larger ? t('search-size-over', { size: larger }) : t('search-size-under', { size: smaller });
    }
    if (term === undefined) return '';
    if (term.kind === 'expr' || chip.kind === 'other') return serializeTerm(term);
    return term.negated ? t('search-chip-negated', { value: term.value }) : term.value;
  };

  /** The chip's whole visible text, which also names it to assistive tech. */
  const chipLabel = (chip: Chip): string => {
    if (TOGGLES.includes(chip.kind)) return toggleLabel(chip.kind, termAt(query(), chip));
    if (chip.kind === 'scope') return `${t('search-field-scope')}: ${scopeName()}`;
    return `${t(FIELD_LABEL[chip.kind] ?? 'search-field-other')}: ${chipValue(chip)}`;
  };

  const press = (chip: Chip, anchor: HTMLElement): void => {
    if (TOGGLES.includes(chip.kind)) {
      const term = termAt(query(), chip);
      if (term) commit(replaceTerm(query(), chip.terms[0]!, toggleTerm(term)));
      return;
    }
    setEditing({ kind: chip.kind, chip, anchor });
  };

  /** Add a two-state filter, or switch an existing one back on. */
  const addFlag = (op: Operator, value: string, alike: readonly string[]): void => {
    const q = query();
    const at = q.terms.findIndex((term) => term.kind === 'field' && term.op === op && alike.includes(term.value));
    const term: Term = { kind: 'field', op, value, negated: false, quoted: false };
    commit(at < 0 ? addTerm(q, term) : replaceTerm(q, at, term));
  };

  const addMenu = createContextMenu({
    label: () => t('search-add-filter'),
    scope: () => 'search-add',
    restoreFocus: () => addButton?.focus(),
    actions: (): ContextAction[] => {
      const edit = (kind: ChipKind, icon: IconName, label: string): ContextAction => ({
        id: kind, label: t(label), icon, run: () => { setEditing({ kind, chip: chips().find((c) => c.kind === kind && (kind === 'date' || kind === 'size')) ?? null, anchor: addButton }); },
      });
      return [
        edit('from', 'users', 'search-field-from'),
        edit('to', 'sent', 'search-field-to'),
        edit('subject', 'mail', 'search-field-subject'),
        edit('body', 'file', 'search-field-body'),
        { id: 'attachment', label: t('search-chip-attachment'), icon: 'attachment', separatorBefore: true, run: () => addFlag('has', 'attachment', ['attachment']) },
        { id: 'event', label: t('search-chip-event'), icon: 'calendar', run: () => addFlag('has', 'calendar', ['calendar']) },
        { id: 'unread', label: t('search-chip-unread'), icon: 'mailRead', run: () => addFlag('is', 'unread', ['unread', 'read']) },
        { id: 'flagged', label: t('search-chip-flagged'), icon: 'flag', run: () => addFlag('is', 'flagged', ['flagged', 'unflagged']) },
        { ...edit('date', 'clock', 'search-field-date'), separatorBefore: true },
        edit('size', 'archive', 'search-field-size'),
      ];
    },
  });

  return (
    <div class={css.bar} role="toolbar" aria-label={t('search-filters-label')} data-testid="search-filter-bar">
      <span class={css.chip} data-kind="scope">
        <button
          type="button" class={`${css.chipLabel} ${a11y.focusable}`} aria-haspopup="dialog"
          aria-expanded={editing()?.kind === 'scope'} title={t('search-chip-edit', { label: `${t('search-field-scope')}: ${scopeName()}` })}
          onClick={(e) => setEditing({ kind: 'scope', chip: scopeChip(), anchor: e.currentTarget })}
        >
          <Icon name="folder" /><span class={css.chipText}><span class={css.chipField}>{t('search-field-scope')}:</span> {scopeName()}</span>
        </button>
      </span>
      <Index each={chips()}>
        {(chip) => {
          const toggle = (): boolean => TOGGLES.includes(chip().kind);
          const term = (): Term | undefined => termAt(query(), chip());
          return (
            <span class={css.chip} data-kind={chip().kind} data-negated={term()?.kind !== 'expr' && term() !== undefined && (term() as { negated: boolean }).negated && !toggle()}>
              <button
                type="button" class={`${css.chipLabel} ${a11y.focusable}`}
                aria-haspopup={toggle() ? undefined : 'dialog'}
                aria-expanded={toggle() ? undefined : editing()?.chip?.key === chip().key}
                title={t(toggle() ? 'search-chip-switch' : 'search-chip-edit', { label: chipLabel(chip()) })}
                onClick={(e) => press(chip(), e.currentTarget)}
              >
                <Show when={CHIP_ICON[chip().kind]}>{(icon) => <Icon name={icon()} />}</Show>
                <Show when={!toggle()} fallback={<span class={css.chipText}>{chipLabel(chip())}</span>}>
                  <span class={css.chipText}><span class={css.chipField}>{t(FIELD_LABEL[chip().kind] ?? 'search-field-other')}:</span> {chipValue(chip())}</span>
                </Show>
              </button>
              <button
                type="button" class={`${css.chipRemove} ${a11y.focusable}`}
                aria-label={t('search-chip-remove', { label: chipLabel(chip()) })}
                title={t('search-chip-remove', { label: chipLabel(chip()) })}
                onClick={(e) => {
                  // Focus stays in this slot (now the next chip's remove button), or goes to Add filter.
                  const bar = e.currentTarget.closest('[role="toolbar"]');
                  const slot = Array.from(bar?.querySelectorAll(`.${css.chip}[data-kind]:not([data-kind="scope"])`) ?? []).indexOf(e.currentTarget.parentElement!);
                  commit(removeTerms(query(), chip().terms));
                  queueMicrotask(() => {
                    const left = Array.from(bar?.querySelectorAll<HTMLElement>(`.${css.chip}[data-kind]:not([data-kind="scope"]) button:last-child`) ?? []);
                    (left[slot] ?? addButton).focus();
                  });
                }}
              ><Icon name="close" /></button>
            </span>
          );
        }}
      </Index>
      <button
        ref={addButton} type="button" class={`${css.add} ${a11y.focusable}`} aria-haspopup="menu" aria-expanded={addMenu.isOpen()}
        data-testid="search-add-filter" onClick={(e) => addMenu.open(e)}
      ><Icon name="plus" />{t('search-add-filter')}</button>
      {addMenu.menu()}
      <Show when={editing()}>
        {(state) => (
          <ChipEditor
            state={state()} query={query()} mailboxes={props.mailboxes()} currentMailboxId={props.currentMailboxId()}
            onApply={commit} onClose={() => { const anchor = state().anchor; setEditing(null); if (anchor.isConnected) anchor.focus(); else addButton.focus(); }}
          />
        )}
      </Show>
    </div>
  );
}

interface ChipEditorProps {
  state: Editing;
  query: SearchQuery;
  mailboxes: readonly ScopeMailbox[];
  currentMailboxId: string | null;
  onApply: (next: SearchQuery) => void;
  onClose: () => void;
}

/** The small popover that edits one chip (or a new one). */
function ChipEditor(props: ChipEditorProps): JSX.Element {
  const { kind, chip } = props.state;
  const q = props.query;
  const term = chip ? q.terms[chip.terms[0]!] : undefined;
  const initialText = term === undefined ? '' : term.kind === 'expr' ? term.text : kind === 'other' ? serializeTerm(term) : term.value;
  const [text, setText] = createSignal(initialText);
  const [negated, setNegated] = createSignal(term !== undefined && term.kind !== 'expr' && term.negated);
  const [low, setLow] = createSignal(chip ? valueIn(q, chip.terms, kind === 'size' ? 'larger' : 'after') : '');
  const [high, setHigh] = createSignal(chip ? valueIn(q, chip.terms, kind === 'size' ? 'smaller' : 'before') : '');
  const scopeNow = chip ? resolveScope(valueIn(q, chip.terms, 'in'), props.mailboxes) : null;
  const [scope, setScopeId] = createSignal(scopeNow?.id ?? (chip ? valueIn(q, chip.terms, 'in') : ''));
  const [error, setError] = createSignal('');
  const title = t(FIELD_LABEL[kind] ?? 'search-field-other');
  let panel!: HTMLDivElement;
  const [pos, setPos] = createSignal({ left: 8, top: 8, maxHeight: 400 });

  onMount(() => {
    const place = (): void => {
      const a = props.state.anchor.getBoundingClientRect();
      const vv = window.visualViewport;
      const view = { left: vv?.offsetLeft ?? 0, top: vv?.offsetTop ?? 0, width: vv?.width ?? window.innerWidth, height: vv?.height ?? window.innerHeight };
      const p = placePopover({ left: a.left, top: a.top, width: a.width, height: a.height }, { width: panel.offsetWidth, height: panel.offsetHeight }, view, { rtl: document.dir === 'rtl' });
      setPos({ left: p.left, top: p.top, maxHeight: p.maxHeight });
    };
    place();
    panel.querySelector<HTMLElement>('input,select')?.focus({ preventScroll: true });
    const outside = (e: PointerEvent): void => {
      if (e.target instanceof Node && !panel.contains(e.target) && !props.state.anchor.contains(e.target)) props.onClose();
    };
    document.addEventListener('pointerdown', outside, true);
    window.addEventListener('resize', place);
    onCleanup(() => { document.removeEventListener('pointerdown', outside, true); window.removeEventListener('resize', place); });
  });

  const apply = (e: Event): void => {
    e.preventDefault();
    const at = chip ? chip.terms[0]! : -1;
    const put = (next: Term | null): SearchQuery =>
      at < 0 ? (next === null ? q : addTerm(q, next)) : replaceTerm(q, at, next);
    if (kind === 'date' || kind === 'size') {
      const valid = kind === 'date' ? isDateValue : isSizeValue;
      if ((low() && !valid(low())) || (high() && !valid(high()))) { setError(t(kind === 'date' ? 'search-date-invalid' : 'search-size-invalid')); return; }
      const empty: Chip = { key: 'new', kind, terms: [] };
      props.onApply(kind === 'date'
        ? setRange(q, chip ?? empty, 'after', 'before', { low: low(), high: high() })
        : setRange(q, chip ?? empty, 'larger', 'smaller', { low: low(), high: high() }));
      return;
    }
    if (kind === 'scope') {
      const box = props.mailboxes.find((m) => m.id === scope()) ?? null;
      props.onApply(setScope(q, box, props.mailboxes));
      return;
    }
    const value = text().trim();
    if (kind === 'expr' || kind === 'other') {
      // Raw query text: its own terms take this chip's place.
      const parsed = parseQuery(value).terms;
      const terms = q.terms.slice();
      if (at < 0) terms.push(...parsed); else terms.splice(at, 1, ...parsed);
      props.onApply({ terms });
      return;
    }
    if (value === '') { props.onApply(put(null)); return; }
    const quoted = term !== undefined && term.kind !== 'expr' && term.quoted;
    const op = TEXT_OPS[kind];
    props.onApply(put(op === undefined
      ? { kind: 'text', value, negated: negated(), quoted }
      : { kind: 'field', op, value, negated: negated(), quoted }));
  };

  const remove = (): void => props.onApply(chip ? removeTerms(q, chip.terms) : q);

  return (
    <Portal>
      <div
        ref={panel} role="dialog" aria-label={t('search-chip-edit', { label: title })} class={css.popover}
        data-testid="search-chip-editor"
        style={{ left: `${pos().left}px`, top: `${pos().top}px`, 'max-height': `${pos().maxHeight}px` }}
        onKeyDown={(e) => { if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); props.onClose(); } }}
      >
        <form class={css.popoverForm} onSubmit={apply}>
          {/* Date and size name the chip above their two bound fields; the other
              editors' one field is labelled with the chip's name already. */}
          <Show when={kind === 'date' || kind === 'size'}><p class={css.popoverTitle}>{title}</p></Show>
          <Show when={kind === 'date' || kind === 'size'}>
            <label class={css.field}>{t(kind === 'date' ? 'search-date-after' : 'search-size-larger')}
              <input class={css.input} type={kind === 'date' ? 'date' : 'text'} value={low()} onInput={(e) => { setLow(e.currentTarget.value); setError(''); }} placeholder={kind === 'size' ? t('search-size-hint') : undefined} />
            </label>
            <label class={css.field}>{t(kind === 'date' ? 'search-date-before' : 'search-size-smaller')}
              <input class={css.input} type={kind === 'date' ? 'date' : 'text'} value={high()} onInput={(e) => { setHigh(e.currentTarget.value); setError(''); }} placeholder={kind === 'size' ? t('search-size-hint') : undefined} />
            </label>
          </Show>
          <Show when={kind === 'scope'}>
            <label class={css.field}>{t('search-field-scope')}
              <select class={css.input} value={scope()} onChange={(e) => setScopeId(e.currentTarget.value)}>
                <option value="">{t('search-scope-all')}</option>
                <Show when={props.currentMailboxId !== null && props.mailboxes.find((m) => m.id === props.currentMailboxId)}>
                  {(box) => <option value={box().id}>{t('search-scope-current', { name: box().name })}</option>}
                </Show>
                <For each={props.mailboxes.filter((m) => m.id !== props.currentMailboxId)}>{(box) => <option value={box.id}>{box.name}</option>}</For>
              </select>
            </label>
          </Show>
          <Show when={kind !== 'date' && kind !== 'size' && kind !== 'scope'}>
            <label class={css.field}>{title}
              <input class={css.input} type="text" value={text()} onInput={(e) => setText(e.currentTarget.value)} spellcheck={false} autocomplete="off" />
            </label>
            <Show when={kind !== 'expr' && kind !== 'other'}>
              <label class={css.check}><input type="checkbox" checked={negated()} onChange={(e) => setNegated(e.currentTarget.checked)} />{t('search-exclude')}</label>
            </Show>
          </Show>
          <Show when={error()}><p class={css.hint} role="alert">{error()}</p></Show>
          <div class={css.actions}>
            <Show when={chip !== null}>
              <button type="button" class={`btn btn--ghost ${css.removeAction}`} onClick={remove}>{t('search-remove')}</button>
            </Show>
            <button type="button" class="btn btn--ghost" onClick={() => props.onClose()}>{t('search-cancel')}</button>
            <button type="submit" class="btn btn--primary">{t('search-apply')}</button>
          </div>
        </form>
      </div>
    </Portal>
  );
}
