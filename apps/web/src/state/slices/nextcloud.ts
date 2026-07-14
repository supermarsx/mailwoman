// Nextcloud files state slice (SPEC §18.4, plan §3 e7 / e14b). Owns the app-level
// `NextcloudService` handle + a one-shot "is Nextcloud linked?" probe so the composer
// only surfaces the attach-from-Nextcloud affordance when a Nextcloud account is
// actually configured. Disjoint file — no `store.ts` collision. Files + share links
// only (CalDAV/CardDAV/tasks are core `mw-dav`).
//
// e14b registers this slice; the components (NextcloudAttach / ShareLinkComposer /
// SaveToNextcloud) may also be used standalone with their own service.

import { createSignal, type Accessor } from 'solid-js';
import { NextcloudService, type Fetcher } from '../../modules/nextcloud/service.ts';

export interface NextcloudSlice {
  /** The shared typed client. */
  readonly service: NextcloudService;
  /** Whether a Nextcloud account is linked (hides the attach UI when false). */
  enabled: Accessor<boolean>;
  setEnabled(on: boolean): void;
  /**
   * Probe ONCE per session (idempotent): a root PROPFIND resolves when Nextcloud is
   * linked (⇒ enabled) and throws on `NotConfigured`/501 (⇒ the attach affordance is
   * never mounted), keeping the composer byte-unchanged when Nextcloud is unconfigured.
   */
  ensureEnabled(): Promise<void>;
}

/** Build the Nextcloud slice over an injectable transport (mockable in tests). */
export function createNextcloudSlice(fetcher?: Fetcher): NextcloudSlice {
  const service = new NextcloudService(fetcher);
  const [enabled, setEnabled] = createSignal(false);
  let probed = false;

  async function ensureEnabled(): Promise<void> {
    if (probed) return;
    probed = true;
    try {
      await service.list('/');
      setEnabled(true);
    } catch {
      setEnabled(false);
    }
  }

  return { service, enabled, setEnabled, ensureEnabled };
}
