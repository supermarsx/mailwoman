import {activeFolderViewId,folderViewAllowed,folderViewValue,folderViewLocked,setFolderView} from './folderViewPreferences.ts';
// Reading-pane position is account-synchronized; signed-out contexts retain a
// device fallback. The root attribute follows remote hydration as well as clicks.

import { createEffect, createRoot, createSignal, type Accessor } from 'solid-js';

import { setAccountUiPreferences } from '../api/accountPreferences.ts';
import { preferenceAccount } from '../api/accountPreferenceLifecycle.ts';

export type ReadingPane = 'right' | 'bottom' | 'off';

export const READING_PANE_OPTIONS: readonly ReadingPane[] = ['right', 'bottom', 'off'];
const STORAGE_KEY = 'mw.mail.readingPane';

function load(): ReadingPane {
  if (typeof localStorage === 'undefined') return 'right';
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return READING_PANE_OPTIONS.includes(raw as ReadingPane) ? (raw as ReadingPane) : 'right';
  } catch {
    return 'right';
  }
}

/** Reflect the choice onto :root so the CSS layout override applies. */
function apply(mode: ReadingPane): void {
  if (typeof document === 'undefined') return;
  document.documentElement.setAttribute('data-reading-pane', mode);
}

const [pane, setPaneSig] = createSignal<ReadingPane>(load());
apply(pane());

/** Current reading-pane position (reactive). */
export const readingPane: Accessor<ReadingPane> = () => {
  return preferenceAccount() ? folderViewValue('readingPane') : pane();
};
createRoot(() => createEffect(() => apply(readingPane())));

/** Set + persist + reflect the reading-pane position. */
export function setReadingPane(mode: ReadingPane): void {
  if(folderViewLocked('readingPane'))return;
  const folder=activeFolderViewId();if(folder&&folderViewAllowed()){setFolderView(folder,{readingPane:mode});apply(mode);return;}
  if (preferenceAccount()) { setAccountUiPreferences({readingPane: mode}); apply(mode); return; }
  setPaneSig(mode);
  apply(mode);
  try {
    localStorage?.setItem(STORAGE_KEY, mode);
  } catch {
    /* private mode / quota — best-effort, the attribute is already applied */
  }
}
