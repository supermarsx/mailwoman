import {Show,type JSX} from 'solid-js';
import {t} from '../i18n/index.ts';
import {RemoteContentBar,type RemoteContentBarProps} from './RemoteContentBar.tsx';
import {LinkedFilesBar,type LinkedFilesBarProps} from './LinkedFilesBar.tsx';
import {noticeCss} from './ReaderNoticeBar.tsx';

/** Both controls retain independent consent and one compact shared frame. */
export function MessagePrivacyBar(props:{images?:RemoteContentBarProps|undefined;links?:LinkedFilesBarProps|undefined}):JSX.Element{
 return <Show when={props.images||props.links}><section class={noticeCss.root} aria-label={t('mail-privacy-bar')} data-testid="message-privacy-bar" data-notice-bar=""><div class={noticeCss.combined}><Show when={props.images}>{images=><RemoteContentBar {...images()} compact/>}</Show><Show when={props.links}>{links=><LinkedFilesBar {...links()} compact/>}</Show></div></section></Show>;
}
