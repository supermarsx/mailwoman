// SettingsFormPage additions used by the rebuilt legacy screens (f33): `when` on
// fields and groups, `frame: 'none'`, `revert.value`, and a custom control that
// keeps its focus while its value changes.

import { describe, it, expect, vi } from 'vitest';
import { screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { SettingsFormPage, type SettingsGroup } from './SettingsFormPage.tsx';
import { renderInShell } from '../testkit.tsx';

const GROUPS: SettingsGroup[] = [
  {
    id: 'kind',
    heading: 'Kind',
    fields: [
      {
        id: 'kind',
        label: 'Kind',
        help: 'Which kind.',
        kind: 'select',
        options: [
          { value: 'a', label: 'A' },
          { value: 'b', label: 'B' },
        ],
      },
      { id: 'onlyA', label: 'Only for A', help: 'Shown for A.', kind: 'text', required: true, when: (v) => v.kind === 'a' },
      {
        id: 'note',
        label: 'Note',
        help: 'A custom control.',
        kind: 'text',
        revert: { defaultDisplay: 'empty', value: '' },
        render: (ctx) => <input id={ctx.id} aria-describedby={ctx.describedBy} value={String(ctx.value ?? '')} onInput={(e) => ctx.setValue(e.currentTarget.value)} />,
      },
    ],
  },
  { id: 'b-only', heading: 'B settings', when: (v) => v.kind === 'b', fields: [{ id: 'bField', label: 'B field', help: 'For B.', kind: 'text' }] },
];

function setup(frame: 'page' | 'none' = 'page') {
  const save = vi.fn(async () => ({}));
  const utils = renderInShell(() => (
    <>
      <h3 id="detail-h">Detail</h3>
      <SettingsFormPage
        title="Things"
        groups={GROUPS}
        frame={frame}
        labelledBy="detail-h"
        load={async () => ({ values: { kind: 'a', onlyA: '', note: 'x', bField: '' } })}
        save={save}
      />
    </>
  ));
  return { ...utils, save };
}

describe('SettingsFormPage (f33 additions)', () => {
  it('shows a field or group only while its `when` holds, and does not validate hidden fields', async () => {
    const { save } = setup();
    expect(await screen.findByRole('textbox', { name: 'Only for A' })).toBeInTheDocument();
    expect(screen.queryByRole('group', { name: 'B settings' })).toBeNull();
    fireEvent.change(screen.getByRole('combobox', { name: 'Kind' }), { target: { value: 'b' } });
    expect(screen.queryByRole('textbox', { name: 'Only for A' })).toBeNull();
    expect(screen.getByRole('group', { name: 'B settings' })).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect((save.mock.calls[0] as unknown[])[0]).toEqual({ kind: 'b' });
  });

  it('a custom control keeps focus while typing; revert.value puts the default back as a change', async () => {
    setup();
    const note = await screen.findByRole('textbox', { name: 'Note' });
    note.focus();
    fireEvent.input(note, { target: { value: 'xy' } });
    expect(document.activeElement).toBe(note);
    expect(screen.getByRole('textbox', { name: 'Note' })).toBe(note);
    fireEvent.click(screen.getByRole('button', { name: /Revert to default/ }));
    await waitFor(() => expect(note).toHaveValue(''));
    expect(screen.getByTestId('save-bar')).toHaveTextContent('1 unsaved change');
  });

  it("frame 'none' draws no page heading and names the form by the outside heading", async () => {
    setup('none');
    await screen.findByRole('combobox', { name: 'Kind' });
    expect(screen.queryByRole('heading', { name: 'Things' })).toBeNull();
    expect(screen.getByRole('form', { name: 'Detail' })).toBeInTheDocument();
  });
});
