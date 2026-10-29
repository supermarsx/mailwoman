import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { Settings } from './Settings.tsx';
import { AppContext } from '../state/context.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppState } from '../state/store.ts';
import { clearSettings, loadSettings } from '../api/settings.ts';

// Settings only touches the theme slice; provide just that (cast to AppState) so
// the test stays isolated from the other executors' slices.
function renderSettings(onClose = vi.fn()) {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const app = {...createThemeSlice(ctx), accountId:()=>null} as unknown as AppState;
  const utils = render(() => (
    <AppContext.Provider value={app}>
      <Settings onClose={onClose} />
    </AppContext.Provider>
  ));
  return { app, onClose, ...utils };
}

describe('Settings', () => {
  beforeEach(() => {
    clearSettings();
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
  });

  it('renders the appearance controls', () => {
    renderSettings();
    expect(screen.getByRole('dialog', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByText('Theme')).toBeInTheDocument();
    expect(screen.getByText('Density')).toBeInTheDocument();
    expect(screen.getByText('Accent')).toBeInTheDocument();
    expect(screen.getByText('Interface font')).toBeInTheDocument();
    expect(screen.getByText('Layout')).toBeInTheDocument();
  });

  it('switching theme drives the slice + :root attribute', () => {
    const { app } = renderSettings();
    fireEvent.click(screen.getByRole('button', { name: 'Grove Dark' }));
    expect(app.theme()).toBe('grove-dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('grove-dark');
  });

  it('marks the active option with aria-pressed', () => {
    const { app } = renderSettings();
    app.setDensity('compact');
    const btn = screen.getByRole('button', { name: 'Compact' });
    expect(btn).toHaveAttribute('aria-pressed', 'true');
  });

  it('picking an accent swatch sets the override', () => {
    const { app } = renderSettings();
    fireEvent.click(screen.getByRole('button', { name: 'Green' }));
    expect(app.accentId()).toBe('green');
  });

  it('picking a badge colour sets data-badge, and the theme default clears it (t40)', () => {
    const { app } = renderSettings();
    const group = screen.getByRole('group', { name: 'Unread badge colour' });
    expect(app.badgeColor()).toBe('default');
    expect(group.querySelector('[aria-pressed="true"]')).toHaveTextContent('Theme default');
    fireEvent.click(screen.getByRole('button', { name: 'Green badges' }));
    expect(app.badgeColor()).toBe('green');
    expect(document.documentElement).toHaveAttribute('data-badge', 'green');
    expect(app.appearancePrefs().badgeColor).toBe('green');
    fireEvent.click(screen.getByRole('button', { name: 'Same as accent' }));
    expect(document.documentElement).toHaveAttribute('data-badge', 'accent');
    fireEvent.click(group.querySelector('button')!);
    expect(document.documentElement).not.toHaveAttribute('data-badge');
  });

  it('lighter/darker accent badges set derived colours on :root and follow the accent live (t85)', () => {
    const { app } = renderSettings();
    const root = document.documentElement;
    const lighter = screen.getByRole('button', { name: 'Lighter accent' });
    const darker = screen.getByRole('button', { name: 'Darker accent' });
    fireEvent.click(lighter);
    expect(app.badgeColor()).toBe('accent-lighter');
    expect(root).toHaveAttribute('data-badge', 'accent-lighter');
    const first = root.style.getPropertyValue('--mw-badge-derived');
    expect(first).toMatch(/^#[0-9a-f]{6}$/);
    expect(lighter.style.background).not.toBe('');
    app.setAccentId('green');
    const second = root.style.getPropertyValue('--mw-badge-derived');
    expect(second).not.toBe(first);
    fireEvent.click(darker);
    expect(root).toHaveAttribute('data-badge', 'accent-darker');
    expect(root.style.getPropertyValue('--mw-badge-derived')).not.toBe(second);
    expect(root.style.getPropertyValue('--mw-badge-derived-text')).toMatch(/^#(ffffff|000000)$/);
    fireEvent.click(screen.getByRole('button', { name: 'Same as accent' }));
    expect(root.style.getPropertyValue('--mw-badge-derived')).toBe('');
  });

  it('applies an administrator badge colour lock and disables the choices', async () => {
    await loadSettings({fetcher: async () => new Response(JSON.stringify({settings:{'appearance.badgeColor':{value:'teal',source:'global',locked:true,lockedBy:'global'}}}), {status:200})});
    const { app } = renderSettings();
    await vi.waitFor(() => expect(app.badgeColor()).toBe('teal'));
    const group = screen.getByRole('group', {name:'Unread badge colour'});
    expect(group.querySelectorAll('button')).toHaveLength(14);
    for (const button of group.querySelectorAll('button')) expect(button).toBeDisabled();
    expect(group).toHaveAttribute('aria-describedby','settings-badge-color-lock');
  });

  it('applies an administrator font and disables every font choice', async () => {
    await loadSettings({fetcher: async () => new Response(JSON.stringify({settings:{'appearance.font':{value:'inter',source:'domain',locked:true,lockedBy:'domain'}}}), {status:200})});
    const { app } = renderSettings();
    await vi.waitFor(() => expect(app.uiFont()).toBe('inter'));
    const group = screen.getByRole('group', {name:'Interface font'});
    expect(group.querySelectorAll('button')).toHaveLength(8);
    for (const button of group.querySelectorAll('button')) expect(button).toBeDisabled();
    expect(group).toHaveAttribute('aria-describedby','settings-font-lock');
  });

  it('the close button and backdrop click both invoke onClose', () => {
    const { onClose } = renderSettings();
    fireEvent.click(screen.getByRole('button', { name: 'Close settings' }));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole('dialog', { name: 'Settings' }));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('moves focus into the dialog on open (focus trap)', async () => {
    renderSettings();
    const dialog = screen.getByRole('dialog', { name: 'Settings' });
    await vi.waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));
  });

  it('Esc from within the dialog closes it', () => {
    const { onClose } = renderSettings();
    fireEvent.keyDown(screen.getByRole('button', { name: 'Close settings' }), { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
