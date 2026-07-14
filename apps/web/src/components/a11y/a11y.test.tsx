import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { createFocusTrap } from './focusTrap.ts';
import { createRovingTabindex } from './rovingTabindex.ts';
import { focusableWithin, firstFocusable, isFocusable } from './focusable.ts';

describe('focusable utilities', () => {
  it('lists enabled, visible, focusable descendants in DOM order', () => {
    render(() => (
      <div data-testid="box">
        <button>one</button>
        <button disabled>skip-disabled</button>
        <a href="#x">two</a>
        <span tabindex="-1">skip-negative</span>
        <input />
      </div>
    ));
    const box = screen.getByTestId('box');
    const names = focusableWithin(box).map((el) => el.textContent || el.tagName);
    expect(names).toEqual(['one', 'two', 'INPUT']);
    expect(firstFocusable(box)?.textContent).toBe('one');
  });

  it('isFocusable rejects tabindex=-1 and disabled', () => {
    render(() => (
      <div data-testid="b">
        <button data-testid="ok">ok</button>
        <button data-testid="dis" disabled>
          no
        </button>
      </div>
    ));
    expect(isFocusable(screen.getByTestId('ok'))).toBe(true);
    expect(isFocusable(screen.getByTestId('dis'))).toBe(false);
  });
});

describe('createFocusTrap', () => {
  function Dialog(props: { onEscape?: () => void }) {
    let el!: HTMLDivElement;
    createFocusTrap(() => el, props.onEscape ? { onEscape: props.onEscape } : {});
    return (
      <div ref={el} role="dialog" aria-modal="true" tabindex="-1" data-testid="dlg">
        <button data-testid="first">first</button>
        <button data-testid="mid">mid</button>
        <button data-testid="last">last</button>
      </div>
    );
  }

  it('moves focus to the first focusable on activation', async () => {
    render(() => <Dialog />);
    await vi.waitFor(() => expect(document.activeElement).toBe(screen.getByTestId('first')));
  });

  it('wraps Tab from last back to first', async () => {
    render(() => <Dialog />);
    const first = screen.getByTestId('first');
    const last = screen.getByTestId('last');
    await vi.waitFor(() => expect(document.activeElement).toBe(first));
    last.focus();
    fireEvent.keyDown(screen.getByTestId('dlg'), { key: 'Tab' });
    expect(document.activeElement).toBe(first);
  });

  it('wraps Shift+Tab from first back to last', async () => {
    render(() => <Dialog />);
    const first = screen.getByTestId('first');
    const last = screen.getByTestId('last');
    await vi.waitFor(() => expect(document.activeElement).toBe(first));
    fireEvent.keyDown(screen.getByTestId('dlg'), { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it('invokes onEscape on Esc', async () => {
    const onEscape = vi.fn();
    render(() => <Dialog onEscape={onEscape} />);
    await vi.waitFor(() => expect(document.activeElement).toBe(screen.getByTestId('first')));
    fireEvent.keyDown(screen.getByTestId('dlg'), { key: 'Escape' });
    expect(onEscape).toHaveBeenCalledTimes(1);
  });

  it('restores focus to the opener when it deactivates', async () => {
    const [open, setOpen] = createSignal(false);
    function Harness() {
      let el!: HTMLDivElement;
      createFocusTrap(() => el, { active: open });
      return (
        <>
          <button data-testid="opener" onClick={() => setOpen(true)}>
            open
          </button>
          {open() && (
            <div ref={el} role="dialog" tabindex="-1">
              <button data-testid="inner">inner</button>
            </div>
          )}
        </>
      );
    }
    render(() => <Harness />);
    const opener = screen.getByTestId('opener');
    opener.focus();
    fireEvent.click(opener);
    await vi.waitFor(() => expect(document.activeElement).toBe(screen.getByTestId('inner')));
    setOpen(false);
    await vi.waitFor(() => expect(document.activeElement).toBe(opener));
  });
});

describe('createRovingTabindex', () => {
  function Menu(props: { orientation?: 'horizontal' | 'vertical' }) {
    let el!: HTMLDivElement;
    createRovingTabindex(() => el, props.orientation ? { orientation: props.orientation } : {});
    return (
      <div ref={el} role="menu" data-testid="menu">
        <button data-roving-item data-testid="a">
          a
        </button>
        <button data-roving-item data-testid="b">
          b
        </button>
        <button data-roving-item data-testid="c">
          c
        </button>
      </div>
    );
  }

  it('seeds a single tab stop', () => {
    render(() => <Menu />);
    expect(screen.getByTestId('a').tabIndex).toBe(0);
    expect(screen.getByTestId('b').tabIndex).toBe(-1);
    expect(screen.getByTestId('c').tabIndex).toBe(-1);
  });

  it('ArrowRight/ArrowLeft move the roving focus and wrap', () => {
    render(() => <Menu />);
    const a = screen.getByTestId('a');
    const b = screen.getByTestId('b');
    a.focus();
    fireEvent.keyDown(screen.getByTestId('menu'), { key: 'ArrowRight' });
    expect(document.activeElement).toBe(b);
    expect(b.tabIndex).toBe(0);
    expect(a.tabIndex).toBe(-1);
    // wrap left from a
    a.focus();
    fireEvent.keyDown(screen.getByTestId('menu'), { key: 'ArrowLeft' });
    expect(document.activeElement).toBe(screen.getByTestId('c'));
  });

  it('Home/End jump to the ends', () => {
    render(() => <Menu />);
    const b = screen.getByTestId('b');
    b.focus();
    fireEvent.keyDown(screen.getByTestId('menu'), { key: 'End' });
    expect(document.activeElement).toBe(screen.getByTestId('c'));
    fireEvent.keyDown(screen.getByTestId('menu'), { key: 'Home' });
    expect(document.activeElement).toBe(screen.getByTestId('a'));
  });

  it('vertical orientation responds to ArrowDown/ArrowUp', () => {
    render(() => <Menu orientation="vertical" />);
    const a = screen.getByTestId('a');
    a.focus();
    fireEvent.keyDown(screen.getByTestId('menu'), { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getByTestId('b'));
  });
});
