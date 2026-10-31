import {createEffect, onCleanup} from 'solid-js';
import {settings} from '../api/settings.ts';

/** Scoped to the app lifecycle, reset when settings/account ownership changes. */
export function useInterfacePreferences(): void {
  const root = document.documentElement;
  const keys = ['--mw-ui-font-size', '--mw-ui-scale', '--mw-ui-font-weight', '--mw-ui-medium-weight', '--mw-ui-bold-weight', '--mw-ui-heavy-weight', '--mw-ui-text-color', '--mw-ui-text-mix'];
  const clear = () => keys.forEach(key => root.style.removeProperty(key));
  createEffect(() => {
    const map = settings();
    clear();
    const size = map['appearance.fontSize']?.value;
    if (typeof size === 'number' && size >= 12 && size <= 24) {root.style.setProperty('--mw-ui-scale', String(size / 14));root.style.setProperty('--mw-ui-font-size', `${size}px`);}
    const weight = map['appearance.fontWeight']?.value;
    if (typeof weight === 'string' && ['300','400','500','600'].includes(weight)) {
      const base = Number(weight);
      root.style.setProperty('--mw-ui-font-weight', weight);
      root.style.setProperty('--mw-ui-medium-weight', String(Math.min(900, base + 100)));
      root.style.setProperty('--mw-ui-bold-weight', String(Math.min(900, base + 300)));
      root.style.setProperty('--mw-ui-heavy-weight', String(Math.min(900, base + 400)));
    }
    const color = map['appearance.fontColor']?.value;
    if (typeof color === 'string' && /^#[0-9a-f]{6}$/i.test(color)) root.style.setProperty('--mw-ui-text-color', color);
    if (map['appearance.lowContrast']?.value === true) root.style.setProperty('--mw-ui-text-mix', '82%');
  });
  onCleanup(clear);
}
