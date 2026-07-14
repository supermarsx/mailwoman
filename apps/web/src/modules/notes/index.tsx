// Notes module (plan §2.5, §3 e6). A two-pane personal-notes surface mounted into
// the app shell beside Mail: a searchable/filterable list (pinned first, color
// chips, tag filter) on the left, and a detail editor (title, color, tags, pin,
// rich-text body, `mailwoman:` cross-links) on the right. It reads the frozen
// `Note/*` surface through the notes store slice (`state/slices/notes.ts`) via
// `useApp()`; the slice is mock-backed until e10 swaps in the real engine.
//
// Bodies are plaintext HTML on the client, allowlist-sanitized here before they
// leave the editor (see `sanitize.ts`); they are sealed only at rest server-side
// (plan §1.6). Reuses the V2 design tokens (`--bg`/`--surface`/… in app.css).

import { For, Show, createMemo, createSignal, onMount, type JSX } from 'solid-js';
import { t, loadCatalog } from '../../i18n';
import { useApp } from '../../state/context.ts';
import { NOTE_COLORS, noteBodyText } from '../../state/slices/notes.ts';
import type { Note, NoteLink } from '../../api/pim-types.ts';
import { NoteEditor } from './Editor.tsx';
import { LinkPicker, linkMeta } from './LinkPicker.tsx';
import { htmlToText } from './sanitize.ts';
import './notes.css';

/** A short plain-text snippet for the list row. */
function snippet(note: Note): string {
  const text = noteBodyText(note).replace(/<[^>]*>/g, ' ').trim();
  return text.length > 120 ? `${text.slice(0, 120)}…` : text;
}

export function NotesModule(): JSX.Element {
  const app = useApp();
  const [selectedId, setSelectedId] = createSignal<string | null>(null);

  onMount(() => {
    void loadCatalog('notes');
    void app.loadNotes();
  });

  const selected = createMemo(() => app.notes().find((n) => n.id === selectedId()) ?? null);

  async function newNote(): Promise<void> {
    const note = await app.createNote({ title: t('notes-untitled') });
    setSelectedId(note.id);
  }

  return (
    <section class="notes" aria-label={t('notes-title')} data-module="notes">
      <NotesList
        selectedId={selectedId()}
        onSelect={(id) => setSelectedId(id)}
        onNew={() => void newNote()}
      />
      <Show when={selected()} fallback={<NotesEmpty />}>
        {(note) => <NoteDetail note={note()} onDeleted={() => setSelectedId(null)} />}
      </Show>
    </section>
  );
}

// ── List pane ────────────────────────────────────────────────────────────────

function NotesList(props: {
  selectedId: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
}): JSX.Element {
  const app = useApp();
  return (
    <div class="notes__list" aria-label={t('notes-list')}>
      <div class="notes__toolbar">
        <button type="button" class="notes__new" onClick={props.onNew}>
          {t('notes-new')}
        </button>
        <input
          class="notes__search"
          type="search"
          placeholder={t('notes-search-placeholder')}
          aria-label={t('notes-search')}
          value={app.noteSearch()}
          onInput={(e) => app.setNoteSearch(e.currentTarget.value)}
        />
      </div>

      <Show when={app.noteTags().length > 0}>
        <div class="notes__tagfilter" role="group" aria-label={t('notes-filter-tag')}>
          <button
            type="button"
            class="notes__tagchip"
            classList={{ 'notes__tagchip--active': app.noteTagFilter() === null }}
            onClick={() => app.setNoteTagFilter(null)}
          >
            {t('notes-all')}
          </button>
          <For each={app.noteTags()}>
            {(tag) => (
              <button
                type="button"
                class="notes__tagchip"
                classList={{ 'notes__tagchip--active': app.noteTagFilter() === tag }}
                onClick={() => app.setNoteTagFilter(app.noteTagFilter() === tag ? null : tag)}
              >
                #{tag}
              </button>
            )}
          </For>
        </div>
      </Show>

      <ul class="notes__items" role="listbox" aria-label={t('notes-listbox')}>
        <For each={app.filteredNotes()} fallback={<li class="notes__none">{t('notes-none')}</li>}>
          {(note) => (
            <li>
              <button
                type="button"
                class="notes__item"
                classList={{ 'notes__item--selected': note.id === props.selectedId }}
                role="option"
                aria-selected={note.id === props.selectedId}
                onClick={() => props.onSelect(note.id)}
              >
                <span class="notes__swatch" style={{ background: note.color }} aria-hidden="true" />
                <span class="notes__item-main">
                  <span class="notes__item-title">
                    <Show when={note.pinned}>
                      <span class="notes__pin" aria-label={t('notes-pinned')}>📌</span>{' '}
                    </Show>
                    <bdi>{note.title.length > 0 ? note.title : t('notes-untitled')}</bdi>
                  </span>
                  <span class="notes__item-snippet"><bdi>{snippet(note)}</bdi></span>
                  <Show when={note.tags.length > 0}>
                    <span class="notes__item-tags">
                      <For each={note.tags}>{(t) => <span class="notes__tag">#{t}</span>}</For>
                    </span>
                  </Show>
                </span>
              </button>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
}

function NotesEmpty(): JSX.Element {
  return (
    <div class="notes__detail notes__detail--empty" aria-label={t('notes-none-selected')}>
      <p>{t('notes-empty-hint')}</p>
    </div>
  );
}

// ── Detail pane ──────────────────────────────────────────────────────────────

function NoteDetail(props: { note: Note; onDeleted: () => void }): JSX.Element {
  const app = useApp();
  const [tagDraft, setTagDraft] = createSignal('');
  const note = (): Note => props.note;

  function setBody(html: string): void {
    void app.updateNote(note().id, { bodyHtml: html, bodyText: htmlToText(html) });
  }

  function addTag(e: Event): void {
    e.preventDefault();
    const t = tagDraft().trim().toLowerCase();
    if (t.length === 0 || note().tags.includes(t)) {
      setTagDraft('');
      return;
    }
    void app.updateNote(note().id, { tags: [...note().tags, t] });
    setTagDraft('');
  }

  function removeTag(tag: string): void {
    void app.updateNote(note().id, { tags: note().tags.filter((t) => t !== tag) });
  }

  async function del(): Promise<void> {
    await app.deleteNote(note().id);
    props.onDeleted();
  }

  return (
    <div class="notes__detail" aria-label={t('notes-editor')}>
      <div class="notes__detail-head">
        <input
          class="notes__title-input"
          type="text"
          placeholder={t('notes-title-placeholder')}
          aria-label={t('notes-title-label')}
          value={note().title}
          onInput={(e) => void app.updateNote(note().id, { title: e.currentTarget.value })}
        />
        <button
          type="button"
          class="notes__pin-btn"
          aria-label={note().pinned ? t('notes-unpin') : t('notes-pin')}
          aria-pressed={note().pinned}
          onClick={() => void app.toggleNotePin(note().id)}
        >
          {note().pinned ? '📌' : '📍'}
        </button>
        <button type="button" class="notes__delete" aria-label={t('notes-delete')} onClick={() => void del()}>
          🗑
        </button>
      </div>

      <div class="notes__colors" role="group" aria-label={t('notes-color-group')}>
        <For each={NOTE_COLORS}>
          {(c) => (
            <button
              type="button"
              class="notes__color"
              classList={{ 'notes__color--active': note().color === c }}
              style={{ background: c }}
              aria-label={t('notes-color', { color: c })}
              aria-pressed={note().color === c}
              onClick={() => void app.updateNote(note().id, { color: c })}
            />
          )}
        </For>
      </div>

      <div class="notes__tags-editor">
        <For each={note().tags}>
          {(tag) => (
            <span class="notes__tag notes__tag--editable">
              #<bdi>{tag}</bdi>
              <button
                type="button"
                class="notes__tag-remove"
                aria-label={t('notes-remove-tag', { tag })}
                onClick={() => removeTag(tag)}
              >
                ×
              </button>
            </span>
          )}
        </For>
        <form class="notes__tag-add" onSubmit={addTag}>
          <input
            type="text"
            placeholder={t('notes-add-tag-placeholder')}
            aria-label={t('notes-add-tag')}
            value={tagDraft()}
            onInput={(e) => setTagDraft(e.currentTarget.value)}
          />
        </form>
      </div>

      <NoteEditor noteId={note().id} html={note().bodyHtml} onInput={setBody} />

      <div class="notes__links">
        <h3 class="notes__links-head">{t('notes-links')}</h3>
        <ul class="notes__links-list" aria-label={t('notes-crosslinks')}>
          <For each={note().links} fallback={<li class="notes__none">{t('notes-no-links')}</li>}>
            {(link, i) => <LinkChip link={link} onRemove={() => void app.removeNoteLink(note().id, i())} />}
          </For>
        </ul>
        <LinkPicker onAdd={(link: NoteLink) => void app.addNoteLink(note().id, link)} />
      </div>
    </div>
  );
}

function LinkChip(props: { link: NoteLink; onRemove: () => void }): JSX.Element {
  const meta = linkMeta(props.link.type);
  return (
    <li class="notes__link-chip">
      <span class="notes__link-icon" aria-hidden="true">
        {meta.icon}
      </span>
      <span class="notes__link-label">
        {meta.label}: <bdi>{props.link.id}</bdi>
      </span>
      <button
        type="button"
        class="notes__link-remove"
        aria-label={t('notes-remove-link', { label: meta.label, id: props.link.id })}
        onClick={props.onRemove}
      >
        ×
      </button>
    </li>
  );
}
