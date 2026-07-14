import { describe, it, expect } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AdminPlugins, UNSIGNED_BANNER } from './index.tsx';
import { anyUnsignedEnabled, type PluginInfo, type PluginsApi } from '../../../state/slices/plugins.ts';

function plugin(over: Partial<PluginInfo>): PluginInfo {
  return {
    id: 'p',
    name: 'Plugin',
    version: '1.0.0',
    signed: true,
    approved: true,
    enabled: false,
    allowUnsigned: false,
    capabilities: ['account-backend'],
    netAllowlist: [],
    limits: { memoryMb: 64, deadlineMs: 500, fuel: null },
    ...over,
  };
}

function mockApi(list: PluginInfo[]): PluginsApi & { calls: string[] } {
  const calls: string[] = [];
  let current = list;
  return {
    calls,
    async list() {
      return current;
    },
    async approve(id) {
      calls.push(`approve:${id}`);
      current = current.map((p) => (p.id === id ? { ...p, approved: true } : p));
    },
    async enable(id) {
      calls.push(`enable:${id}`);
      current = current.map((p) => (p.id === id ? { ...p, enabled: true } : p));
    },
    async disable(id) {
      calls.push(`disable:${id}`);
      current = current.map((p) => (p.id === id ? { ...p, enabled: false } : p));
    },
    async grant(id, input) {
      calls.push(`grant:${id}:${input.capability}`);
    },
    async setAllowUnsigned(id, allow) {
      calls.push(`allow:${id}:${allow}`);
      current = current.map((p) => (p.id === id ? { ...p, allowUnsigned: allow } : p));
    },
  };
}

describe('unsigned-plugin detection', () => {
  it('flags an enabled unsigned plugin', () => {
    expect(anyUnsignedEnabled([plugin({ signed: false, enabled: true })])).toBe(true);
    expect(anyUnsignedEnabled([plugin({ signed: false, enabled: false })])).toBe(false);
    expect(anyUnsignedEnabled([plugin({ signed: true, enabled: true })])).toBe(false);
  });
});

describe('Admin → Plugins: signed vs unsigned banner', () => {
  it('shows the PERMANENT unsigned banner when an unsigned plugin is enabled', async () => {
    const api = mockApi([plugin({ id: 'lt', name: 'LanguageTool', signed: false, enabled: true, allowUnsigned: true })]);
    render(() => <AdminPlugins api={api} />);

    await waitFor(() => expect(screen.getByTestId('unsigned-banner')).toBeInTheDocument());
    expect(screen.getByTestId('unsigned-banner')).toHaveTextContent(UNSIGNED_BANNER);
    expect(screen.getByTestId('sig-chip')).toHaveTextContent('Unsigned');
  });

  it('shows NO banner when every enabled plugin is signed', async () => {
    const api = mockApi([plugin({ id: 'graph', name: 'Graph bridge', signed: true, enabled: true })]);
    render(() => <AdminPlugins api={api} />);

    await waitFor(() => expect(screen.getByTestId('plugin-card')).toBeInTheDocument());
    expect(screen.queryByTestId('unsigned-banner')).not.toBeInTheDocument();
    expect(screen.getByTestId('sig-chip')).toHaveTextContent('Signed');
  });

  it('refuses to enable an unsigned plugin until allow-unsigned is set', async () => {
    const api = mockApi([plugin({ id: 'x', signed: false, approved: true, enabled: false, allowUnsigned: false })]);
    render(() => <AdminPlugins api={api} />);

    await waitFor(() => expect(screen.getByTestId('plugin-card')).toBeInTheDocument());
    const enableBtn = screen.getByRole('button', { name: 'Enable' }) as HTMLButtonElement;
    expect(enableBtn.disabled).toBe(true);

    fireEvent.click(screen.getByTestId('allow-unsigned').querySelector('input')!);
    await waitFor(() => expect(api.calls).toContain('allow:x:true'));
  });
});

describe('Admin → Plugins: registry actions', () => {
  it('approves then enables a pending signed plugin', async () => {
    const api = mockApi([plugin({ id: 'g', signed: true, approved: false, enabled: false })]);
    render(() => <AdminPlugins api={api} />);

    await waitFor(() => expect(screen.getByRole('button', { name: 'Approve' })).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Enable' })).toBeInTheDocument());
    fireEvent.click(screen.getByRole('button', { name: 'Enable' }));
    await waitFor(() => expect(api.calls).toEqual(['approve:g', 'enable:g']));
  });
});
