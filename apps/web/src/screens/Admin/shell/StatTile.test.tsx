import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@solidjs/testing-library';
import { StatTile } from './StatTile.tsx';

describe('StatTile', () => {
  it('shows a figure under its label', () => {
    render(() => <StatTile testId="t" label="Provisioned users" value={42} detail="3 disabled" />);
    const tile = screen.getByTestId('t');
    expect(screen.getByRole('figure', { name: 'Provisioned users' })).toBe(tile);
    expect(tile).toHaveTextContent('42');
    expect(tile).toHaveTextContent('3 disabled');
  });

  it('null is "Not available" with the reason, never a zero', () => {
    render(() => <StatTile testId="t" label="Free space" value={null} unavailableReason="The server cannot read free space on this platform." />);
    const tile = screen.getByTestId('t');
    expect(tile).toHaveTextContent('Not available');
    expect(tile).toHaveTextContent('The server cannot read free space on this platform.');
    expect(tile.textContent).not.toMatch(/\d/);
  });

  it('zero is a real figure, shown as 0', () => {
    render(() => <StatTile testId="t" label="Blocked" value={0} unavailableReason="never shown" />);
    expect(screen.getByTestId('t')).toHaveTextContent('0');
    expect(screen.getByTestId('t')).not.toHaveTextContent('Not available');
  });

  it('formats numbers and links to the section that explains it', () => {
    const onLink = vi.fn((e: MouseEvent) => e.preventDefault());
    render(() => (
      <StatTile label="Database" value={2048} format={(n) => `${n / 1024} KB`} href="/admin/console/observability" linkLabel="Details" onLink={onLink} />
    ));
    expect(screen.getByText('2 KB')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('link', { name: 'Details' }));
    expect(onLink).toHaveBeenCalled();
  });
});
