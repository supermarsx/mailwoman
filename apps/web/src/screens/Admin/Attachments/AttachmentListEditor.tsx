// Chip-style editor of one attachment rule list (extensions or MIME patterns).
// The registry draft stays JSON text; this editor reads and writes that text.
// Entries are normalised and checked with the server's own rule shape; repeats,
// bad entries, the 100-rule limit and rules that sit in another list are said
// in words under the field. Paste many, import and export are plain text, one
// rule per line.

import { For, Show, createSignal, type JSX } from 'solid-js';
import { t } from '../../../i18n';
import type { InlineEditorContext } from '../Settings/InlineSettingEditor.tsx';
import { MAX_RULES, addBulk, conflictsFor, exportText, normaliseRule, type ListName } from './rules.ts';
import * as css from './attachments.css.ts';

export interface AttachmentListEditorProps {
  ctx: InlineEditorContext;
  list: ListName;
  mime: boolean;
  /** The other lists' current rules, for conflict detection. */
  others: () => Record<ListName, readonly string[]>;
}

function parse(draft: unknown): string[] {
  if (typeof draft !== 'string') return [];
  try {
    const v: unknown = JSON.parse(draft);
    return Array.isArray(v) ? v.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

export function AttachmentListEditor(props: AttachmentListEditorProps): JSX.Element {
  const rules = (): string[] => parse(props.ctx.draft);
  const write = (next: string[]): void => props.ctx.onDraft(JSON.stringify(next, null, 2));
  const [input, setInput] = createSignal('');
  const [bulk, setBulk] = createSignal('');
  const [filter, setFilter] = createSignal('');
  const [notes, setNotes] = createSignal<string[]>([]);
  const kind = (): 'mime' | 'extension' => (props.mime ? 'mime' : 'extension');

  function merge(text: string): void {
    const r = addBulk(rules(), text, props.mime);
    const out: string[] = [];
    if (r.invalid.length > 0) out.push(t('admin-attachments-invalid', { entries: r.invalid.map((i) => i.input.trim()).join(', ') }));
    if (r.duplicates.length > 0) out.push(t('admin-attachments-duplicates', { rules: r.duplicates.join(', ') }));
    if (r.overflow.length > 0) out.push(t('admin-attachments-overflow', { max: MAX_RULES }));
    setNotes(out);
    if (r.rules.length !== rules().length) write(r.rules);
    if (r.invalid.length === 0 && r.overflow.length === 0) {
      setInput('');
      setBulk('');
    }
  }
  function add(): void {
    const raw = input().trim();
    if (raw === '') return;
    if (!/[\s,;]/.test(raw)) {
      const n = normaliseRule(raw, props.mime);
      if (!n.ok) {
        setNotes([t(`admin-attachments-err-${n.error === 'empty' ? kind() : n.error === 'long' ? 'long' : kind()}`)]);
        return;
      }
    }
    merge(raw);
  }
  async function importFile(file: File | undefined): Promise<void> {
    if (file) merge(await file.text());
  }
  function exportFile(): void {
    try {
      const url = URL.createObjectURL(new Blob([exportText(rules())], { type: 'text/plain' }));
      const a = document.createElement('a');
      a.href = url;
      a.download = `${props.list}-${props.mime ? 'mime-types' : 'extensions'}.txt`;
      a.click();
      URL.revokeObjectURL(url);
    } catch {
      /* no object URLs in this browser */
    }
  }
  const visible = (): string[] => rules().filter((r) => r.includes(filter().trim().toLowerCase()));
  const conflicts = () => conflictsFor(props.list, rules(), props.others(), props.mime);
  const notesId = `${props.ctx.wiring.id}-notes`;

  return (
    <div class={css.listEditor} data-testid={`attachment-list-${props.list}-${kind()}`}>
      <Show when={!props.ctx.disabled}>
        <div class={css.addRow}>
          <input
            id={props.ctx.wiring.id}
            type="text"
            value={input()}
            placeholder={t(props.mime ? 'admin-attachments-add-mime' : 'admin-attachments-add-extension')}
            aria-describedby={`${notesId} ${props.ctx.wiring.describedBy ?? ''}`.trim()}
            autocapitalize="none"
            spellcheck={false}
            onInput={(e) => setInput(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                add();
              }
            }}
            onPaste={(e) => {
              const text = e.clipboardData?.getData('text') ?? '';
              if (/[\s,;]/.test(text.trim())) {
                e.preventDefault();
                merge(text);
              }
            }}
          />
          <button type="button" class="btn" onClick={add}>{t('admin-attachments-add')}</button>
        </div>
      </Show>
      <Show when={props.ctx.disabled}>
        <span id={props.ctx.wiring.id} class={css.readonlyLabel}>{t('admin-attachments-count', { count: rules().length, max: MAX_RULES })}</span>
      </Show>
      <div id={notesId} aria-live="polite" class={css.notes}>
        <For each={notes()}>{(n) => <p role="alert" class={css.warning}>{n}</p>}</For>
        <For each={conflicts()}>{(c) => <p class={css.warning} data-testid="attachment-conflict">{t(`admin-attachments-conflict-${c.kind}`, { rule: c.rule, other: c.other })}</p>}</For>
      </div>
      <Show when={rules().length > 8}>
        <input type="search" value={filter()} aria-label={t('admin-attachments-filter')} placeholder={t('admin-attachments-filter')} onInput={(e) => setFilter(e.currentTarget.value)} />
      </Show>
      <Show when={rules().length > 0} fallback={<p class={css.muted}>{t('admin-attachments-empty-list')}</p>}>
        <ul class={css.chips} aria-label={t('admin-attachments-list-label')}>
          <For each={visible()} fallback={<li class={css.muted}>{t('admin-attachments-none-match')}</li>}>{(rule) => (
            <li class={css.chip}>
              <code>{rule}</code>
              <Show when={!props.ctx.disabled}>
                <button type="button" class={css.chipRemove} aria-label={t('admin-attachments-remove', { rule })} onClick={() => write(rules().filter((r) => r !== rule))}>×</button>
              </Show>
            </li>
          )}</For>
        </ul>
      </Show>
      <p class={css.muted}>{t('admin-attachments-count', { count: rules().length, max: MAX_RULES })}</p>
      <div class={css.tools}>
        <Show when={!props.ctx.disabled}>
          <details>
            <summary>{t('admin-attachments-bulk')}</summary>
            <textarea rows="4" value={bulk()} aria-label={t('admin-attachments-bulk')} onInput={(e) => setBulk(e.currentTarget.value)} />
            <button type="button" class="btn" disabled={bulk().trim() === ''} onClick={() => merge(bulk())}>{t('admin-attachments-bulk-add')}</button>
          </details>
          <label class="btn btn--ghost">
            {t('admin-attachments-import')}
            <input type="file" accept=".txt,.csv,text/plain" hidden onChange={(e) => { void importFile(e.currentTarget.files?.[0]); e.currentTarget.value = ''; }} />
          </label>
        </Show>
        <button type="button" class="btn btn--ghost" disabled={rules().length === 0} onClick={exportFile}>{t('admin-attachments-export')}</button>
      </div>
    </div>
  );
}
