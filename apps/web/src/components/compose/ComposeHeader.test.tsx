import { afterEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { ComposeHeader } from './ComposeHeader.tsx';
import type { HeaderCollapseMode } from './headerCollapse.ts';

afterEach(() => localStorage.clear());

function mount(mode: HeaderCollapseMode, empty = false) {
  const [attention, setAttention] = createSignal(0);
  render(() => (
    <ComposeHeader mode={() => mode} summary={() => 'To: Ana +2 · Cc: 1 · Subject: Re: Invoice'} empty={() => empty} attention={attention}>
      <input aria-label="To" />
    </ComposeHeader>
  ));
  return { setAttention, toggle: () => screen.getByTestId('compose-header-toggle'), body: () => screen.getByTestId('compose-header-body'), input: () => screen.getByLabelText('To', { selector: 'input' }) };
}

describe('ComposeHeader', () => {
  it('is open in expanded mode and folds to the summary line on the toggle', () => {
    const { toggle, body } = mount('expanded');
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
    expect(body()).not.toHaveAttribute('hidden');
    fireEvent.click(toggle());
    expect(toggle()).toHaveAttribute('aria-expanded', 'false');
    expect(body()).toHaveAttribute('hidden');
    expect(screen.getByTestId('compose-header-summary')).toHaveTextContent('To: Ana +2 · Cc: 1 · Subject: Re: Invoice');
    expect(toggle()).toHaveAttribute('aria-controls', body().id);
  });

  it('keeps the fields mounted while folded', () => {
    const { toggle, input } = mount('collapsed');
    expect(input()).toBeInTheDocument();
    expect(toggle()).toHaveAttribute('aria-expanded', 'false');
  });

  it('remembers the user choice in this browser', () => {
    const { toggle } = mount('expanded');
    fireEvent.click(toggle());
    expect(localStorage.getItem('mw.compose.headerOpen')).toBe('0');
  });

  it('opens a folded header while focus is inside it, and folds again when focus leaves', () => {
    const { toggle, input } = mount('collapsed');
    fireEvent.focusIn(input());
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
    fireEvent.focusOut(input(), { relatedTarget: document.body });
    expect(toggle()).toHaveAttribute('aria-expanded', 'false');
  });

  it('opens when the host asks for attention', () => {
    const { toggle, setAttention } = mount('collapsed');
    setAttention(1);
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
  });

  it('stays open while there is nothing to summarise', () => {
    const { toggle } = mount('collapsed', true);
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
  });

  it('auto mode folds, and a click peeks without remembering', () => {
    const { toggle } = mount('auto');
    expect(toggle()).toHaveAttribute('aria-expanded', 'false');
    fireEvent.click(toggle());
    expect(toggle()).toHaveAttribute('aria-expanded', 'true');
    expect(localStorage.getItem('mw.compose.headerOpen')).toBeNull();
  });
});
