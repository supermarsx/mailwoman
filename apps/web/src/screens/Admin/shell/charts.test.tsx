import { describe, it, expect } from 'vitest';
import { render, screen, within } from '@solidjs/testing-library';
import { Sparkline } from './Sparkline.tsx';
import { BarChart } from './BarChart.tsx';

const pts = [
  { t: 0, value: 10 },
  { t: 60, value: 25 },
  { t: 120, value: 5 },
];
const time = (s: number): string => `t${s}`;

describe('Sparkline', () => {
  it('is a region named by its title, with a summary and a hidden table of every point', () => {
    render(() => <Sparkline testId="s" title="Requests per minute" points={pts} formatTime={time} />);
    const region = screen.getByRole('region', { name: 'Requests per minute' });
    expect(region).toHaveTextContent('Latest 5, highest 25');
    const table = within(region).getByRole('table', { name: 'Requests per minute' });
    const cells = within(table).getAllByRole('row').slice(1).map((r) => r.textContent);
    expect(cells).toEqual(['t010', 't6025', 't1205']);
    // The drawing is hidden from assistive technology: the table is the text alternative.
    expect(region.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
  });

  it('draws one 2px line through every point, scaled to the highest value', () => {
    render(() => <Sparkline testId="s" title="R" points={pts} />);
    const line = screen.getByTestId('s').querySelector('polyline')!;
    const coords = line.getAttribute('points')!.split(' ').map((p) => p.split(',').map(Number));
    expect(coords).toHaveLength(3);
    expect(coords[0]![0]).toBe(0);
    expect(coords[2]![0]).toBe(100);
    // The highest point is at the top, the lowest nearest the baseline.
    expect(coords[1]![1]).toBeLessThan(coords[0]![1]!);
    expect(coords[2]![1]).toBeGreaterThan(coords[0]![1]!);
  });

  it('hover shows the time and value of the nearest bucket', () => {
    render(() => <Sparkline testId="s" title="R" points={pts} formatTime={time} />);
    const svg = screen.getByTestId('s').querySelector('svg')!;
    svg.getBoundingClientRect = () => ({ left: 0, width: 200, top: 0, height: 48, right: 200, bottom: 48, x: 0, y: 0, toJSON: () => ({}) });
    // jsdom has no PointerEvent; a MouseEvent of the same type carries clientX.
    svg.dispatchEvent(new MouseEvent('pointermove', { bubbles: true, clientX: 199 }));
    expect(screen.getByTestId('s')).toHaveTextContent('t120: 5');
    svg.dispatchEvent(new MouseEvent('pointerleave', { bubbles: false }));
    expect(screen.getByTestId('s')).not.toHaveTextContent('t120: 5');
  });

  it('no points: says so instead of drawing a flat zero line', () => {
    render(() => <Sparkline testId="s" title="R" points={[]} emptyText="No requests were counted in this window." />);
    expect(screen.getByTestId('s')).toHaveTextContent('No requests were counted in this window.');
    expect(screen.getByTestId('s').querySelector('svg')).toBeNull();
  });
});

describe('BarChart', () => {
  it('prints label and value per bar and carries the same data in a hidden table', () => {
    render(() => <BarChart testId="b" title="Blocked accounts per domain" bars={[{ label: 'a.example', value: 4 }, { label: 'b.example', value: 1 }]} />);
    const region = screen.getByRole('region', { name: 'Blocked accounts per domain' });
    const table = within(region).getByRole('table');
    expect(within(table).getAllByRole('row').slice(1).map((r) => r.textContent)).toEqual(['a.example4', 'b.example1']);
    const visible = region.querySelector('ul')!;
    expect(visible).toHaveAttribute('aria-hidden', 'true');
    expect(visible).toHaveTextContent('a.example');
  });

  it('bar lengths are proportional to the largest value', () => {
    render(() => <BarChart testId="b" title="T" bars={[{ label: 'x', value: 4 }, { label: 'y', value: 1 }]} />);
    const fills = [...screen.getByTestId('b').querySelectorAll('li span span')] as HTMLElement[];
    expect(fills.map((f) => f.dataset['share'])).toEqual(['100', '25']);
    expect(fills.every((f) => !f.hasAttribute('style'))).toBe(true);
  });

  it('no bars: says so', () => {
    render(() => <BarChart testId="b" title="T" bars={[]} emptyText="Nothing to show." />);
    expect(screen.getByTestId('b')).toHaveTextContent('Nothing to show.');
    expect(screen.queryByRole('table')).toBeNull();
  });
});
