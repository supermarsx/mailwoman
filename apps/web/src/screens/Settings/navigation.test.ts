import { describe, expect, it, vi } from 'vitest';
import { createSettingsNavigation } from './navigation.tsx';

describe('retained settings navigation', () => {
  it('keeps an editor when its dirty guard refuses and unregisters on disposal', async () => {
    const nav=createSettingsNavigation({section:'signatures',editor:{kind:'edit',id:'work'}});
    const guard=vi.fn(() => false);const dispose=nav.beforeLeave(guard);
    expect(await nav.navigate({section:'appearance'})).toBe(false);
    expect(nav.route().editor?.id).toBe('work');
    dispose();expect(await nav.navigate({section:'appearance'})).toBe(true);
  });
  it('shares a pending confirmation and only applies the latest navigation', async () => {
    const nav=createSettingsNavigation();let resolve!:(yes:boolean)=>void;
    const guard=vi.fn(() => new Promise<boolean>(done => {resolve=done;}));nav.beforeLeave(guard);
    const first=nav.navigate({section:'signatures'});const second=nav.navigate({section:'identities'});
    expect(guard).toHaveBeenCalledTimes(1);resolve(true);
    expect(await first).toBe(false);expect(await second).toBe(true);expect(nav.route().section).toBe('identities');
  });
});
