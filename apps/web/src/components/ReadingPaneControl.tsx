import {t} from '../i18n/index.ts';
import {readingPane,setReadingPane,READING_PANE_OPTIONS,type ReadingPane} from './readingPane.ts';
import {setAccountUiPreferences} from '../api/accountPreferences.ts';
import {IconSelect} from './IconSelect.tsx';
import type {IconName} from './Icon.tsx';

const icons:Record<ReadingPane,IconName>={right:'paneRight',bottom:'paneBottom',off:'eyeOff'};

export function ReadingPaneControl() {
  return <IconSelect label={t('mail-reading-pane')} value={readingPane()} options={READING_PANE_OPTIONS.map(mode=>({value:mode,label:t(`mail-reading-pane-${mode}`),icon:icons[mode]}))} onChange={value=>{setAccountUiPreferences({readerCollapsed:false});setReadingPane(value as ReadingPane);}}/>;
}
