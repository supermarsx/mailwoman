import { describe, it, expect, vi } from 'vitest';
import { render } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { anyDirty, createLeaveCheck, useDirtyGuard } from './useDirtyGuard.ts';

function Form(props: { dirty: () => boolean }) {
  useDirtyGuard(props.dirty);
  return <form aria-label="f" />;
}

describe('useDirtyGuard', () => {
  it('reports dirty only while a mounted form says so', () => {
    const [dirty, setDirty] = createSignal(false);
    const { unmount } = render(() => <Form dirty={dirty} />);
    expect(anyDirty()).toBe(false);
    setDirty(true);
    expect(anyDirty()).toBe(true);
    unmount();
    expect(anyDirty()).toBe(false);
  });

  it('beforeunload is cancelled while dirty and left alone otherwise', () => {
    const [dirty, setDirty] = createSignal(true);
    const { unmount } = render(() => <Form dirty={dirty} />);
    const e1 = new Event('beforeunload', { cancelable: true });
    window.dispatchEvent(e1);
    expect(e1.defaultPrevented).toBe(true);
    setDirty(false);
    const e2 = new Event('beforeunload', { cancelable: true });
    window.dispatchEvent(e2);
    expect(e2.defaultPrevented).toBe(false);
    unmount();
  });

  it('the leave check asks only when something is dirty, and keeps the page on "stay"', async () => {
    const ask = vi.fn(async () => false);
    const check = createLeaveCheck(ask);
    expect(await check()).toBe(true);
    expect(ask).not.toHaveBeenCalled();
    const [dirty] = createSignal(true);
    const { unmount } = render(() => <Form dirty={dirty} />);
    expect(await check()).toBe(false);
    expect(ask).toHaveBeenCalledTimes(1);
    unmount();
  });
});
