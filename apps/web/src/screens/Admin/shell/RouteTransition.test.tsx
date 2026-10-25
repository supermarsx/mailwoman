import { afterEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { RouteTransition } from './RouteTransition.tsx';

afterEach(() => { vi.unstubAllGlobals(); document.documentElement.removeAttribute('data-reduced-motion'); });
it('replays entry without remounting the page, losing drafts or changing focus', () => {
  let frame!: FrameRequestCallback; let mounted = 0;
  vi.stubGlobal('requestAnimationFrame', (callback: FrameRequestCallback) => { frame = callback; return 1; });
  vi.stubGlobal('cancelAnimationFrame', vi.fn());
  vi.stubGlobal('matchMedia', () => ({ matches: false }));
  const [key, setKey] = createSignal('domains');
  function Draft() { mounted++; return <input aria-label="Retained draft" />; }
  render(() => <RouteTransition routeKey={key()}><Draft /></RouteTransition>);
  const input = screen.getByLabelText('Retained draft'); input.focus(); fireEvent.input(input, { target: { value: 'unsaved' } });
  frame(0); expect(screen.getByTestId('admin-route-transition')).toHaveAttribute('data-mw-entering', 'true');
  setKey('domains/example.org');
  expect(screen.getByTestId('admin-route-transition')).toHaveAttribute('data-mw-entering', 'false');
  frame(1);
  expect(screen.getByLabelText('Retained draft')).toBe(input); expect(input).toHaveValue('unsaved');
  expect(document.activeElement).toBe(input); expect(mounted).toBe(1);
});
it('does not schedule motion for system or application reduced-motion preference', () => {
  const raf = vi.fn(); vi.stubGlobal('requestAnimationFrame', raf);
  vi.stubGlobal('matchMedia', () => ({ matches: true }));
  const first = render(() => <RouteTransition routeKey="one"><p>One</p></RouteTransition>);
  expect(raf).not.toHaveBeenCalled(); first.unmount();
  vi.stubGlobal('matchMedia', () => ({ matches: false })); document.documentElement.setAttribute('data-reduced-motion', '');
  render(() => <RouteTransition routeKey="two"><p>Two</p></RouteTransition>);
  expect(raf).not.toHaveBeenCalled();
});
