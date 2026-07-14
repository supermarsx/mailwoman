// V7 Assist (AI) state slice (SPEC §14, plan §3 e6). Owns the reactive gateway
// config (which drives the HARD hide-when-disabled rule everywhere), the per-message
// "what left the device" disclosure log, and the auto-tag audit trail + mode.
//
// SAFETY: the slice holds a single `AssistService`, which has NO send/delete/accept
// method (see modules/assist/service.ts). Nothing here transmits mail. When the
// gateway is disabled the config's `availability` is 'disabled' and every Assist
// component renders nothing.

import { createSignal, type Accessor } from 'solid-js';
import type { SliceContext } from './context.ts';
import { AssistService } from '../../modules/assist/service.ts';
import {
  DISABLED_CONFIG,
  hasCapability,
  type AssistCapability,
  type AssistConfig,
  type AutoTagMode,
  type Disclosure,
  type TagAuditEntry,
} from '../../modules/assist/types.ts';

/** A per-message disclosure record: what left the device, when, and to where. */
export interface DisclosureLogEntry {
  readonly id: string;
  readonly capability: AssistCapability;
  readonly disclosure: Disclosure;
  readonly ts: string;
}

export interface AssistSlice {
  /** The shared gateway service (config + read-only invoke + transcribe). No send path. */
  readonly service: AssistService;
  /** The reactive gateway config; DISABLED_CONFIG until (and unless) enabled. */
  config: Accessor<AssistConfig>;
  /** Convenience: is the whole Assist UI available at all? */
  enabled: Accessor<boolean>;
  /** Is a specific capability granted (on an enabled gateway)? */
  can(cap: AssistCapability): boolean;
  /** Fetch `/api/assist/config` (called once at mount; failure ⇒ stays disabled). */
  loadConfig(): Promise<void>;

  /** The per-message "what left the device" log (newest last). */
  disclosureLog: Accessor<readonly DisclosureLogEntry[]>;
  /** Append a disclosure (called after any invoke that forwarded content). */
  recordDisclosure(capability: AssistCapability, disclosure: Disclosure): void;

  /** The auto-tag audit trail (suggested / applied / reverted, with attribution). */
  tagAudit: Accessor<readonly TagAuditEntry[]>;
  recordTagAudit(entry: TagAuditEntry): void;

  /** Auto-tag mode. 'suggest' is the default; 'auto' is opt-in and persisted. */
  autoTagMode: Accessor<AutoTagMode>;
  setAutoTagMode(mode: AutoTagMode): void;
}

const AUTOTAG_MODE_KEY = 'mw.assist.autotag.v1';

function loadAutoTagMode(): AutoTagMode {
  try {
    return globalThis.localStorage?.getItem(AUTOTAG_MODE_KEY) === 'auto' ? 'auto' : 'suggest';
  } catch {
    return 'suggest';
  }
}

let logSeq = 0;

/**
 * Build the Assist slice. `service` is injectable so component/slice tests run
 * without a live gateway; production defaults to the same-origin HTTP service.
 */
export function createAssistSlice(_ctx: SliceContext, service: AssistService = new AssistService()): AssistSlice {
  const [config, setConfig] = createSignal<AssistConfig>(DISABLED_CONFIG);
  const [disclosureLog, setDisclosureLog] = createSignal<readonly DisclosureLogEntry[]>([]);
  const [tagAudit, setTagAudit] = createSignal<readonly TagAuditEntry[]>([]);
  const [autoTagMode, setAutoTagModeSig] = createSignal<AutoTagMode>(loadAutoTagMode());

  return {
    service,
    config,
    enabled: () => config().availability === 'enabled',
    can: (cap) => hasCapability(config(), cap),
    async loadConfig() {
      try {
        setConfig(await service.getConfig());
      } catch {
        setConfig(DISABLED_CONFIG);
      }
    },

    disclosureLog,
    recordDisclosure(capability, disclosure) {
      logSeq += 1;
      const entry: DisclosureLogEntry = {
        id: `disc-${Date.now()}-${logSeq}`,
        capability,
        disclosure,
        ts: new Date().toISOString(),
      };
      setDisclosureLog((prev) => [...prev, entry]);
    },

    tagAudit,
    recordTagAudit(entry) {
      setTagAudit((prev) => [...prev, entry]);
    },

    autoTagMode,
    setAutoTagMode(mode) {
      setAutoTagModeSig(mode);
      try {
        globalThis.localStorage?.setItem(AUTOTAG_MODE_KEY, mode);
      } catch {
        // Non-fatal: the mode still applies for this session.
      }
    },
  };
}
