import { createEffect, onCleanup, type Accessor } from 'solid-js';
import { loadCustomization, type AppearanceOwner } from '../api/appearanceCustomization.ts';
import { mountAppearanceCss, runAppearanceScript, type AppearanceUpdate } from '../theme/appearanceSandbox.ts';
import { appearanceScriptProperties } from '../theme/appearanceScriptPolicy.ts';
import { isThemeName } from '../theme/registry.ts';

export const CUSTOMIZATION_CHANGED = 'mw-appearance-customization-changed';
export function AppearanceCustomizationRuntime(props: {owner: Accessor<AppearanceOwner | null>}): null {
  createEffect(() => {
    const owner = props.owner();
    if (!owner || new URLSearchParams(location.search).get('appearanceSafe') === '1') return;
    const lifetime = new AbortController();
    let execution = new AbortController();
    let removeCss = () => {};
    let removeScript = () => {};
    let lastTag = '';
    let loading = false;
    const clear = () => { execution.abort(); removeCss(); removeScript(); removeCss = removeScript = () => {}; };
    const refresh = async () => {
      if (loading || lifetime.signal.aborted) return;
      loading = true;
      const request = new AbortController();
      const cancel = () => request.abort();
      lifetime.signal.addEventListener('abort', cancel, {once: true});
      const deadline = setTimeout(cancel, 5000);
      try {
        const snapshot = await loadCustomization(owner, request.signal);
        const fingerprint = JSON.stringify([snapshot.etag, snapshot.value.cssEnabled, snapshot.value.scriptEnabled, snapshot.value.cssAllowed, snapshot.value.lockedFields, document.documentElement.dataset.theme]);
        if (lifetime.signal.aborted || fingerprint === lastTag) return;
        clear(); lastTag = fingerprint;
        execution = new AbortController();
        const value = snapshot.value;
        if (value.cssEnabled && value.cssAllowed) removeCss = mountAppearanceCss(value.compiledCss);
        if (value.scriptEnabled && value.document.script.trim()) {
          let combined: AppearanceUpdate = {};
          runAppearanceScript({code: value.document.script, signal: execution.signal,
            context: {dark: document.documentElement.dataset.appearance === 'dark', reducedMotion: matchMedia('(prefers-reduced-motion: reduce)').matches},
            onError: () => { removeScript(); removeScript = () => {}; },
            onUpdate: update => {
              try {
                combined = {...combined, ...update};
                const selected = document.documentElement.dataset.theme;
                const entries = appearanceScriptProperties(combined, value.lockedFields, isThemeName(selected) ? selected : 'dark');
                removeScript();
                removeScript = mountAppearanceCss(`.${value.scopeClass}{${entries.map(([name, item]) => `${name}:${item}`).join(';')}}`);
              } catch { execution.abort(); removeScript(); removeScript = () => {}; }
            },
          });
        }
      } catch { if (!lifetime.signal.aborted) {clear(); lastTag = '';} }
      finally { clearTimeout(deadline); lifetime.signal.removeEventListener('abort', cancel); loading = false; }
    };
    void refresh();
    const changed = () => { lastTag = ''; void refresh(); };
    const visible = () => { if (!document.hidden) void refresh(); };
    const timer = setInterval(() => { if (!document.hidden) void refresh(); }, 30_000);
    window.addEventListener(CUSTOMIZATION_CHANGED, changed);
    document.addEventListener('visibilitychange', visible);
    onCleanup(() => {lifetime.abort(); clear(); clearInterval(timer); window.removeEventListener(CUSTOMIZATION_CHANGED, changed); document.removeEventListener('visibilitychange', visible);});
  });
  return null;
}
