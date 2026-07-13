import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { MaxSecuritySwitch } from './MaxSecuritySwitch.tsx';
import type { SecurityMode } from './max-security.ts';

describe('MaxSecuritySwitch', () => {
  it('renders a radiogroup with the three positions', () => {
    const { getByTestId, getAllByRole } = render(() => (
      <MaxSecuritySwitch value="full-sanitized" onChange={() => {}} />
    ));
    const group = getByTestId('max-security-switch');
    expect(group.getAttribute('role')).toBe('radiogroup');
    expect(group.getAttribute('aria-label')).toBeTruthy();
    expect(getAllByRole('radio')).toHaveLength(3);
  });

  it('marks the selected position aria-checked and roving-tabindex 0', () => {
    const { getByTestId } = render(() => (
      <MaxSecuritySwitch value="sanitized-no-media" onChange={() => {}} />
    ));
    const sel = getByTestId('max-security-opt-sanitized-no-media');
    expect(sel.getAttribute('aria-checked')).toBe('true');
    expect(sel.getAttribute('tabindex')).toBe('0');
    const other = getByTestId('max-security-opt-full-sanitized');
    expect(other.getAttribute('aria-checked')).toBe('false');
    expect(other.getAttribute('tabindex')).toBe('-1');
  });

  it('reports the chosen mode on click', () => {
    const onChange = vi.fn();
    const { getByTestId } = render(() => (
      <MaxSecuritySwitch value="full-sanitized" onChange={onChange} />
    ));
    fireEvent.click(getByTestId('max-security-opt-plain-text'));
    expect(onChange).toHaveBeenCalledWith('plain-text');
  });

  it('disables positions below the admin floor and refuses to select them', () => {
    const onChange = vi.fn();
    const { getByTestId } = render(() => (
      <MaxSecuritySwitch value="sanitized-no-media" floor="sanitized-no-media" onChange={onChange} />
    ));
    const below = getByTestId('max-security-opt-full-sanitized');
    expect(below.getAttribute('aria-disabled')).toBe('true');
    expect((below as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(below);
    expect(onChange).not.toHaveBeenCalled();
  });

  it('arrow keys move between enabled positions and select', () => {
    const [value, setValue] = createSignal<SecurityMode>('full-sanitized');
    const { getByTestId } = render(() => (
      <MaxSecuritySwitch value={value()} onChange={setValue} />
    ));
    const group = getByTestId('max-security-switch');
    fireEvent.keyDown(group, { key: 'ArrowRight' });
    expect(value()).toBe('sanitized-no-media');
    fireEvent.keyDown(group, { key: 'ArrowRight' });
    expect(value()).toBe('plain-text');
    fireEvent.keyDown(group, { key: 'Home' });
    expect(value()).toBe('full-sanitized');
    fireEvent.keyDown(group, { key: 'End' });
    expect(value()).toBe('plain-text');
  });

  it('arrow-key navigation skips floor-disabled positions', () => {
    const [value, setValue] = createSignal<SecurityMode>('sanitized-no-media');
    const { getByTestId } = render(() => (
      <MaxSecuritySwitch value={value()} floor="sanitized-no-media" onChange={setValue} />
    ));
    const group = getByTestId('max-security-switch');
    // only no-media + plain-text are enabled; wrapping never lands on full
    fireEvent.keyDown(group, { key: 'ArrowLeft' });
    expect(value()).toBe('plain-text');
    fireEvent.keyDown(group, { key: 'ArrowRight' });
    expect(value()).toBe('sanitized-no-media');
  });
});
