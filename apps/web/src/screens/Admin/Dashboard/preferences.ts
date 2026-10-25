export const WIDGETS = ['checklist', 'overview', 'security', 'traffic', 'health', 'storage', 'audit', 'server'] as const;
export type DashboardWidgetId = typeof WIDGETS[number];
export interface DashboardPreferences { order: DashboardWidgetId[]; hidden: DashboardWidgetId[]; refreshSecs: 0 | 30 | 60 | 300 }
export function dashboardPreferences(value: unknown): DashboardPreferences {
  const source = value !== null && typeof value === 'object' ? value as Record<string, unknown> : {};
  const ids = (value: unknown): DashboardWidgetId[] => Array.isArray(value) ? [...new Set(value.filter((v): v is DashboardWidgetId => typeof v === 'string' && WIDGETS.includes(v as DashboardWidgetId)))] : [];
  const order = ids(source['order']);
  return { order: [...order, ...WIDGETS.filter(v => !order.includes(v))], hidden: ids(source['hidden']), refreshSecs: [30, 60, 300].includes(Number(source['refreshSecs'])) ? Number(source['refreshSecs']) as 30 | 60 | 300 : 0 };
}
