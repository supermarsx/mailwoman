import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { Settings } from './Settings.tsx';
import { AppContext } from '../state/context.ts';
import { createThemeSlice } from '../state/slices/theme.ts';
import type { SliceContext } from '../state/slices/context.ts';
import type { AppState } from '../state/store.ts';

// Settings only touches the theme slice; provide just that (cast to AppState) so
// the test stays isolated from the other executors' slices.
function renderSettings(onClose = vi.fn()) {
  const ctx = { client: {}, showToast: vi.fn() } as unknown as SliceContext;
  const app = createThemeSlice(ctx) as unknown as AppState;
  const utils = render(() => (
    <AppContext.Provider value={app}>
      <Settings onClose={onClose} />
    </AppContext.Provider>
  ));
  return { app, onClose, ...utils };
}

describe('Settings', () => {
  beforeEach(() => {
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
    fireEvent.click(screen.getByRole('button', { name: 'Moss' }));
    expect(app.accent()).toBe('#6d8a4e');
  });

  it('the close button and backdrop click both invoke onClose', () => {
    const { onClose } = renderSettings();
    fireEvent.click(screen.getByRole('button', { name: 'Close settings' }));
    expect(onClose).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole('dialog', { name: 'Settings' }));
    expect(onClose).toHaveBeenCalledTimes(2);
  });
});
