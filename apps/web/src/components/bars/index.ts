// Collapsible bars and 3-row ribbons (t38). See CollapsibleBar.tsx and RibbonPanel.tsx.
export {
  CollapsibleBar,
  useCollapsibleBar,
  HOVER_OPEN_MS,
  LEAVE_CLOSE_MS,
  TYPING_GUARD_MS,
  type CollapsibleBarApi,
  type CollapsibleBarProps,
  type RevealPolicy,
} from './CollapsibleBar.tsx';
export {
  useBarMode,
  barSettingKey,
  isBarMode,
  BAR_IDS,
  DEFAULT_BAR_MODE,
  type BarId,
  type BarMode,
  type BarModeHandle,
} from './useBarMode.ts';
export { RibbonPanel, EXPAND_HYSTERESIS_PX, type RibbonPanelProps } from './RibbonPanel.tsx';
export { RibbonGroup, RibbonItem, type RibbonGroupProps, type RibbonItemProps } from './RibbonGroup.tsx';
export { assertRibbonGeometry, type RibbonViolation, type RibbonGeometryOptions } from './geometry.ts';
export {
  RIBBON_ROW_FINE,
  RIBBON_ROW_COARSE,
  RIBBON_GAP,
  ribbonControlsHeight,
} from './ribbonMetrics.ts';
