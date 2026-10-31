import {For} from 'solid-js';
import {t} from '../i18n/index.ts';
import {readingPane,setReadingPane,READING_PANE_OPTIONS,type ReadingPane} from './readingPane.ts';
import {setAccountUiPreferences} from '../api/accountPreferences.ts';

export function ReadingPaneControl() {
  return <label class="list-reading-pane-controls workspace-layout-modes"><span class="workspace-control-label">{t('mail-reading-pane')}</span>
    <select aria-label={t('mail-reading-pane')} value={readingPane()} onChange={event=>{const mode=event.currentTarget.value as ReadingPane;setAccountUiPreferences({readerCollapsed:false});setReadingPane(mode);}}>
      <For each={READING_PANE_OPTIONS}>{mode=><option value={mode}>{t(`mail-reading-pane-${mode}`)}</option>}</For>
    </select>
  </label>;
}
