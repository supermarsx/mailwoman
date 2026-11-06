import { afterEach, describe, expect, it } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { ComposeHeader } from './ComposeHeader.tsx';
import { ccbccGroup, ccbccToggle, densityCompact, densityComfortable } from './composeHeader.css.ts';
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

describe('ComposeHeader density and Cc/Bcc toggle styling', () => {
  it('uses the comfortable density by default and compact only when collapsed', () => {
    mount('auto');
    expect(screen.getByTestId('compose-header')).toHaveAttribute('data-density', 'comfortable');
    expect(screen.getByTestId('compose-header')).toHaveClass(densityComfortable);
  });

  it('uses the compact density in collapsed mode', () => {
    mount('collapsed');
    expect(screen.getByTestId('compose-header')).toHaveAttribute('data-density', 'compact');
    expect(screen.getByTestId('compose-header')).toHaveClass(densityCompact);
  });

  it('defines distinct densities on space tokens that never reach zero', () => {
    expect(densityComfortable).not.toBe(densityCompact);
    const src = readFileSync(resolve(__dirname, 'composeHeader.css.ts'), 'utf8');
    expect(src).toMatch(/\[headRowGap\]: vars\.space\[2\]/);
    expect(src).toMatch(/\[headRowGap\]: vars\.space\[1\]/);
    expect(src).toMatch(/\[headSubjectGap\]: vars\.space\[3\]/);
    const css = readFileSync(resolve(__dirname, 'compose-chrome.css'), 'utf8');
    expect(css).toMatch(/\.compose-head__body \{[^}]*row-gap:var\(--mw-head-row-gap/);
    expect(css).not.toMatch(/margin-block:\.1rem/);
  });

  it('themes the Cc/Bcc toggle: accent link colour, pressed state, focus ring and forced colours', () => {
    const src = readFileSync(resolve(__dirname, 'composeHeader.css.ts'), 'utf8');
    expect(src).toMatch(/color: linkColor/);
    expect(src).toContain(`'&[aria-pressed="true"]'`);
    expect(src).toContain('&:hover');
    expect(src).toContain('&:active');
    expect(src).toContain('&:focus-visible');
    expect(src).toContain('vars.a11y.focusRingWidth');
    expect(src).toContain('forced-colors: active');
    expect(ccbccToggle).toBeTruthy();
    expect(ccbccGroup).toBeTruthy();
  });
});
