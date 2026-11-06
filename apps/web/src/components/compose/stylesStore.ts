// The Styles gallery's per-user state (FORMAT): the style set and the custom styles,
// kept in the settings registry (`compose.styles.defaultSet`, `compose.styles.custom`,
// limit `compose.styles.maxCustom`). A write that fails (signed out, locked) still
// changes what this editor shows; it just is not kept.

import { createSignal } from 'solid-js';
import { useSetting } from '../../screens/Settings/useSetting.ts';
import { isStyleSet, parseCustomStyles, serializeCustomStyles, type StyleDef, type StyleSet } from './styleModel.ts';

const isString = (value: unknown): value is string => typeof value === 'string';
const isCount = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);

export function createStyleStore() {
  const setting = useSetting<string>('compose.styles.defaultSet', 'default', isString);
  const customSetting = useSetting<string>('compose.styles.custom', '', isString);
  const maxSetting = useSetting<number>('compose.styles.maxCustom', 20, isCount);
  const [localSet, setLocalSet] = createSignal<StyleSet | null>(null);
  const [localCustom, setLocalCustom] = createSignal<StyleDef[] | null>(null);
  const set = (): StyleSet => localSet() ?? (isStyleSet(setting.value()) ? (setting.value() as StyleSet) : 'default');
  return {
    set,
    chooseSet(next: StyleSet): void {
      setLocalSet(next);
      void setting.set(next).catch(() => false);
    },
    custom: (): StyleDef[] => localCustom() ?? parseCustomStyles(customSetting.value()),
    saveCustom(list: StyleDef[]): void {
      setLocalCustom(list);
      void customSetting.set(serializeCustomStyles(list)).catch(() => false);
    },
    max: (): number => Math.max(0, Math.min(100, Math.floor(maxSetting.value()))),
    setLocked: (): boolean => setting.locked(),
  };
}

export type StyleStore = ReturnType<typeof createStyleStore>;
