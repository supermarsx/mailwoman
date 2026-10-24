import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Preferences } from './Preferences.tsx';
import { DEFAULT_PREFS, type SettingsPrefs } from './prefs.ts';

// The "Off" keyboard preset (t28-e16): the way to turn single-key shortcuts off
// from Settings (WCAG 2.1.4). The key dispatcher binds nothing for it.

describe('Preferences: keyboard shortcuts off', () => {
  it('offers Off, selects it, and previews no bindings', async () => {
    const [prefs, setPrefs] = createSignal<SettingsPrefs>({ ...DEFAULT_PREFS, keyboardPreset: 'gmail' });
    const onChange = vi.fn((next: SettingsPrefs) => setPrefs(next));
    render(() => <Preferences prefs={prefs} onChange={onChange} />);

    const preview = screen.getByTestId('kbd-preview');
    // Precondition: the Gmail preset lists its bindings.
    expect(preview.querySelectorAll('li').length).toBeGreaterThan(0);

    const off = await screen.findByRole('button', { name: 'Off (no shortcuts)' });
    fireEvent.click(off);
    expect(onChange.mock.calls.at(-1)?.[0]).toMatchObject({ keyboardPreset: 'off' });
    expect(off.getAttribute('aria-pressed')).toBe('true');
    await waitFor(() => expect(preview.querySelectorAll('li').length).toBe(0));
  });
});
