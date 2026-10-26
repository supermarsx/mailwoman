import { describe, it, expect } from 'vitest';
import { dashboardPreferences, WIDGETS } from './preferences.ts';
describe('dashboard preferences', () => {
  it('defaults to manual refresh and includes each supported widget once', () => {
    expect(dashboardPreferences(null)).toEqual({ order: [...WIDGETS], hidden: [], refreshSecs: 0, keepCompletedChecklistProminent:false });
    expect(dashboardPreferences({ order: ['traffic', 'traffic', 'unknown'], hidden: ['server', 'unknown'], refreshSecs: 1 })).toEqual({ order: ['traffic', ...WIDGETS.filter(v => v !== 'traffic')], hidden: ['server'], refreshSecs: 0, keepCompletedChecklistProminent:false });
  });
  it('accepts only bounded refresh intervals', () => {
    for (const refreshSecs of [30, 60, 300]) expect(dashboardPreferences({ refreshSecs }).refreshSecs).toBe(refreshSecs);
    expect(dashboardPreferences({ refreshSecs: -1 }).refreshSecs).toBe(0);
  });
});
