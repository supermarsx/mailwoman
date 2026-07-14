import { describe, it, expect } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { UiPluginTier, UnsignedBanner } from './Tier.tsx';
import { EMPTY_REGISTRY, type UiPluginRegistration, type UiPluginRegistry } from './types';

function reg(over: Partial<UiPluginRegistration> = {}): UiPluginRegistration {
  return {
    manifest: {
      id: 'snooze',
      name: 'Snooze',
      version: '1.0.0',
      signature: 'c2ln', // base64-ish, signed
      extensionPoints: ['message-toolbar'],
      capabilities: ['net:host-allowlist'],
      csp: "default-src 'none'",
    },
    grants: [{ capability: 'net:host-allowlist', params: { hosts: ['api.example.com'] } }],
    enabled: true,
    approved: true,
    ...over,
  };
}

describe('UiPluginTier — fail-soft baseline', () => {
  it('renders NOTHING when the registry is empty (mailbox path byte-unchanged)', () => {
    const { container } = render(() => <UiPluginTier registry={EMPTY_REGISTRY} />);
    expect(container.querySelector('[data-testid="ui-plugin-tier"]')).toBeNull();
    expect(container.textContent).toBe('');
  });

  it('renders nothing when a plugin exists but is not approved+enabled', () => {
    const registry: UiPluginRegistry = { plugins: [reg({ approved: false })], unsignedBanner: [] };
    const { container } = render(() => <UiPluginTier registry={registry} />);
    expect(container.querySelector('[data-testid="ui-plugin-tier"]')).toBeNull();
  });
});

describe('UiPluginTier — sandboxed frame', () => {
  it('renders each active plugin in an opaque-origin sandbox (allow-scripts, no allow-same-origin)', () => {
    const registry: UiPluginRegistry = { plugins: [reg()], unsignedBanner: [] };
    render(() => <UiPluginTier registry={registry} />);
    const frame = screen.getByTitle('plugin:snooze') as HTMLIFrameElement;
    expect(frame.tagName).toBe('IFRAME');
    expect(frame.getAttribute('sandbox')).toBe('allow-scripts');
    expect(frame.getAttribute('sandbox')).not.toContain('allow-same-origin');
    expect(frame.getAttribute('src')).toBeNull();
    expect(frame.getAttribute('srcdoc')).toContain("connect-src 'none'");
    // The slot is a labelled region (WCAG name/role/value).
    expect(screen.getByRole('region', { name: 'Plugin: Snooze' })).toBeInTheDocument();
  });
});

describe('UnsignedBanner', () => {
  it('renders a labelled, persistent trust banner listing unsigned plugin ids', () => {
    render(() => <UnsignedBanner ids={['snooze', 'weather']} />);
    const banner = screen.getByRole('note', { name: 'Unsigned UI plugin warning' });
    expect(banner).toBeInTheDocument();
    expect(banner).toHaveTextContent('snooze');
    expect(banner).toHaveTextContent('weather');
    // Non-dismissable by the plugin: there is no close control the guest could drive.
    expect(banner.querySelector('button')).toBeNull();
  });

  it('renders nothing when there are no unsigned plugins', () => {
    const { container } = render(() => <UnsignedBanner ids={[]} />);
    expect(container.querySelector('[data-testid="ui-plugin-unsigned-banner"]')).toBeNull();
  });

  it('the tier raises the banner from the registry unsignedBanner list', () => {
    const registry: UiPluginRegistry = { plugins: [reg({ manifest: { ...reg().manifest, signature: null } })], unsignedBanner: ['snooze'] };
    render(() => <UiPluginTier registry={registry} />);
    expect(screen.getByRole('note', { name: 'Unsigned UI plugin warning' })).toBeInTheDocument();
  });
});
