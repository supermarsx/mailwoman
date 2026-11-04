// Session-watch readout (password spray, instance spike, session binding, theft
// and country-change alerts). Server: `crates/mw-server/src/security_alerts.rs`.
//
//   GET {console}/security/session-watch → SessionWatch   (capability security.read)

import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';

export interface SpikeMetric {
  metric: string;
  baseline: number;
  current: number;
  floor: number;
}
export interface SessionWatch {
  spray: {
    enabled: boolean;
    trackedSources: number;
    sourcesAtChallengeOrAbove: number;
    windowSecs: number;
    challengeAt: number;
    paceAt: number;
    lockAt: number;
  };
  spike: {
    enabled: boolean;
    warmedUp: boolean;
    observedBuckets: number;
    warmupBuckets: number;
    bucketSecs: number;
    heightened: boolean;
    heightenedUntil: number | null;
    metrics: SpikeMetric[];
    recent: { at: number; signals: number | string[]; observed?: number; baseline?: number }[];
  };
  sessionBinding: {
    enabled?: boolean;
    attributes?: string[];
    strictness?: string;
    webmailAction?: string;
    adminAction?: string;
  };
  theft: {
    enabled?: boolean;
    minTravelSecs?: number;
    concurrentUse?: string;
    webmailAction?: string;
    adminAction?: string;
  };
  countryChange: {
    enabled?: boolean;
    userMuteAllowed?: boolean;
    cooldownSecs?: number;
    emailEnabled?: boolean;
  };
  recentAlerts: {
    ts: number;
    kind: string;
    reason?: string;
    fromCountry?: string | null;
    toCountry?: string | null;
    action: string;
    session?: string;
    surface?: string;
    account?: string;
  }[];
  caveat?: string;
}

export interface SessionWatchApi {
  get(): Promise<SessionWatch>;
}

export const sessionWatchApi: SessionWatchApi = {
  async get() {
    return (await adminRequest<SessionWatch>('/security/session-watch')).data;
  },
};
