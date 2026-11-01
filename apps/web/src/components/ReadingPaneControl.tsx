import {For} from 'solid-js';
import {t} from '../i18n/index.ts';
import {readingPane,setReadingPane,READING_PANE_OPTIONS,type ReadingPane} from './readingPane.ts';
import {setAccountUiPreferences} from '../api/accountPreferences.ts';
import {IconSelect} from './IconSelect.tsx';
import type {IconName} from './Icon.tsx';

const icons:Record<ReadingPane,IconName>={right:'paneRight',bottom:'paneBottom',off:'eyeOff'};

export function ReadingPaneControl() {
  return <IconSelect label={t('mail-reading-pane')} currentLabel={t(`mail-reading-pane-${readingPane()}`)} icon={icons[readingPane()]} value={readingPane()} onChange={value=>{setAccountUiPreferences({readerCollapsed:false});setReadingPane(value as ReadingPane);}}>
      <For each={READING_PANE_OPTIONS}>{mode=><option value={mode}>{t(`mail-reading-pane-${mode}`)}</option>}</For>
  </IconSelect>;
}
