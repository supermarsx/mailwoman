// Cross-link picker (plan §3 e6, §2.1 `Note.links`). Attaches a `mailwoman:`
// cross-link from the open note to a message / event / contact. The link is
// stored structurally on the note (`links:[{type,id}]`) — NOT injected into the
// body HTML — so it survives sanitization and stays a first-class, removable
// reference. Until the sibling modules expose a live object picker (e10), the id
// is entered directly; the type selector mirrors the frozen `NoteLink.type` set.

import { createSignal, type JSX } from 'solid-js';
import type { NoteLink } from '../../api/pim-types.ts';

/** The three cross-link targets (frozen `NoteLink.type`, plan §2.1). */
export const LINK_TYPES: ReadonlyArray<{ type: NoteLink['type']; label: string; icon: string }> = [
  { type: 'email', label: 'Message', icon: '✉️' },
  { type: 'event', label: 'Event', icon: '📅' },
  { type: 'contact', label: 'Contact', icon: '👤' },
];

/** The `mailwoman:` URI form of a cross-link (plan §2.5 picker → URI). */
export function linkUri(link: NoteLink): string {
  return `mailwoman:${link.type}/${link.id}`;
}

/** Human label + glyph for a stored link (chip rendering). */
export function linkMeta(type: NoteLink['type']): { label: string; icon: string } {
  return LINK_TYPES.find((t) => t.type === type) ?? { label: type, icon: '🔗' };
}

export interface LinkPickerProps {
  /** Add the chosen cross-link to the open note (deduped by the slice). */
  onAdd: (link: NoteLink) => void;
}

export function LinkPicker(props: LinkPickerProps): JSX.Element {
  const [type, setType] = createSignal<NoteLink['type']>('email');
  const [id, setId] = createSignal('');

  function add(e: Event): void {
    e.preventDefault();
    const trimmed = id().trim();
    if (trimmed.length === 0) return;
    props.onAdd({ type: type(), id: trimmed });
    setId('');
  }

  return (
    <form class="note-linkpicker" onSubmit={add} aria-label="Add cross-link">
      <select
        class="note-linkpicker__type"
        aria-label="Link type"
        value={type()}
        onChange={(e) => setType(e.currentTarget.value as NoteLink['type'])}
      >
        {LINK_TYPES.map((t) => (
          <option value={t.type}>
            {t.icon} {t.label}
          </option>
        ))}
      </select>
      <input
        class="note-linkpicker__id"
        type="text"
        placeholder="id to link…"
        aria-label="Link target id"
        value={id()}
        onInput={(e) => setId(e.currentTarget.value)}
      />
      <button type="submit" class="note-linkpicker__add" disabled={id().trim().length === 0}>
        Link
      </button>
    </form>
  );
}
