// t95: the shared trail: ARIA, placeholder while a name loads, tooltip only when cut.
import { fireEvent, render, screen, within } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { Breadcrumbs } from './Breadcrumbs.tsx';

it('labels the landmark, marks the current page and shows a placeholder, not an id, while loading', () => {
  render(() => <Breadcrumbs label="Breadcrumb" loadingLabel="Loading name" items={[{ label: 'Dashboard', href: '/c', onNavigate: () => {} }, { label: 'Users', href: '/c#/users' }, { label: null }]} />);
  const nav = screen.getByRole('navigation', { name: 'Breadcrumb' });
  const items = within(nav).getAllByRole('listitem');
  expect(items).toHaveLength(3);
  const loading = items[2]!.querySelector('[data-crumb-loading]')!;
  expect(loading).toHaveAttribute('aria-current', 'page');
  expect(loading).toHaveAttribute('aria-busy', 'true');
  expect(loading).toHaveTextContent('Loading name');
});

it('a level with a page but no address is a button', () => {
  const go = vi.fn();
  render(() => <Breadcrumbs label="Settings path" loadingLabel="Loading" items={[{ label: 'Mail' }, { label: 'Labels', onNavigate: go }, { label: 'Edit' }]} />);
  fireEvent.click(screen.getByRole('button', { name: 'Labels' }));
  expect(go).toHaveBeenCalled();
  expect(screen.getByText('Edit')).toHaveAttribute('aria-current', 'page');
  expect(screen.getByText('Mail')).not.toHaveAttribute('aria-current');
});

it('offers the full name as a tooltip only when the ellipsis cuts it', () => {
  const long = 'A very long display name for an identity provider client that will not fit';
  render(() => <Breadcrumbs label="Breadcrumb" loadingLabel="Loading" items={[{ label: 'Short', href: '/c' }, { label: long }]} />);
  const current = screen.getByText(long);
  expect(current).toHaveAttribute('dir', 'auto');
  Object.defineProperty(current, 'scrollWidth', { configurable: true, value: 400 });
  Object.defineProperty(current, 'clientWidth', { configurable: true, value: 200 });
  fireEvent.pointerOver(current);
  expect(current).toHaveAttribute('data-tooltip', long);
  Object.defineProperty(current, 'scrollWidth', { configurable: true, value: 200 });
  fireEvent.pointerOver(current);
  expect(current).not.toHaveAttribute('data-tooltip');
  const link = screen.getByRole('link', { name: 'Short' });
  fireEvent.pointerOver(link);
  expect(link).not.toHaveAttribute('data-tooltip');
});
