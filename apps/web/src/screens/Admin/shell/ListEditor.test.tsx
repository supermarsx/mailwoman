import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ListEditor } from './ListEditor.tsx';
import { StatusBadge } from './StatusBadge.tsx';
import { expectNoAxeViolations } from './uxTestkit.ts';

function setup(initial: string[] = [], validate?: (e: string) => string | null) {
  const onSubmit = vi.fn((e: Event) => e.preventDefault());
  const [items, setItems] = createSignal(initial);
  const utils = render(() => (
    <form onSubmit={onSubmit}>
      <label for="hosts">Hosts</label>
      <ListEditor id="hosts" label="Hosts" items={items()} onChange={setItems} lowerCase validate={validate} emptyText="No host." />
    </form>
  ));
  return { ...utils, items, onSubmit };
}

describe('ListEditor', () => {
  it('adds with Enter without submitting the form, splits pasted lists, lower-cases', () => {
    const { items, onSubmit } = setup();
    expect(screen.getByText('No host.')).toBeInTheDocument();
    const input = screen.getByRole('textbox', { name: 'Hosts' });
    fireEvent.input(input, { target: { value: 'A.example, b.example' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(items()).toEqual(['a.example', 'b.example']);
    expect(onSubmit).not.toHaveBeenCalled();
    expect(screen.getByRole('list', { name: 'Hosts' }).querySelectorAll('li')).toHaveLength(2);
  });

  it('refuses an empty entry, a duplicate and what validate rejects, with the reason tied to the input', () => {
    const { items } = setup(['a.example'], (e) => (e.includes('!') ? 'Not a host.' : null));
    const input = screen.getByRole('textbox', { name: 'Hosts' });
    fireEvent.click(screen.getByRole('button', { name: 'Add Hosts' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Type an entry first.');
    fireEvent.input(input, { target: { value: 'a.example' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.getByRole('alert')).toHaveTextContent('is already in the list.');
    fireEvent.input(input, { target: { value: 'x!' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.getByRole('alert')).toHaveTextContent('Not a host.');
    expect(input).toHaveAttribute('aria-invalid', 'true');
    expect(input).toHaveAccessibleDescription('Not a host.');
    expect(items()).toEqual(['a.example']);
  });

  it('Remove names its entry and moves focus to the next Remove, then to the input', async () => {
    setup(['a.example', 'b.example']);
    fireEvent.click(screen.getByRole('button', { name: /Remove .*a\.example/ }));
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('button', { name: /Remove .*b\.example/ })));
    fireEvent.click(screen.getByRole('button', { name: /Remove .*b\.example/ }));
    await waitFor(() => expect(document.activeElement).toBe(screen.getByRole('textbox', { name: 'Hosts' })));
  });

  it('has no axe violations', async () => {
    const { container } = setup(['a.example']);
    await expectNoAxeViolations(container);
  });
});

describe('StatusBadge', () => {
  it('carries an icon and text, the tone only as data', () => {
    render(() => (
      <StatusBadge tone="danger" status="expired" testId="b">
        Expired
      </StatusBadge>
    ));
    const b = screen.getByTestId('b');
    expect(b).toHaveTextContent('Expired');
    expect(b.querySelector('svg')).not.toBeNull();
    expect(b).toHaveAttribute('data-tone', 'danger');
  });
});
