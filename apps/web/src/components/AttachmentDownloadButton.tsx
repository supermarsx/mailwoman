// The per-attachment download button of the reader's attachments bar. It is a
// sibling of the thumbnail button (ThumbnailStrip `renderAction`), so it never
// nests a button in a button, and its click never reaches the bar's row toggle
// (the row ignores clicks that land on a button; stopPropagation keeps any
// outer handler out too). A refused file keeps the button focusable with
// aria-disabled so the reason is still read out and shown as the tooltip.

import {createSignal,type JSX} from 'solid-js';
import type {AttachmentPolicy} from '../api/attachmentPolicy.ts';
import {formatBytes} from '../i18n/format.ts';
import {t} from '../i18n';
import {isolate} from '../i18n/index.ts';
import type {StripItem} from '../viewers/ThumbnailStrip.tsx';
import {Icon} from './Icon.tsx';
import {Spinner} from './spinner/Spinner.tsx';
import {downloadBarItem,itemDownloadBlock,type BarApp} from './attachmentsBarDownload.ts';

export function AttachmentDownloadButton(props:{app:BarApp;item:StripItem;emailId:string;policy:AttachmentPolicy|undefined;allowDownload:boolean}):JSX.Element {
 const [busy,setBusy]=createSignal(false);
 const reason=():string|undefined=>itemDownloadBlock(props.policy,props.item,props.allowDownload);
 const name=():string=>isolate(props.item.name);
 const tip=():string=>{const why=reason();return why?t('reader-attachments-download-blocked',{name:name(),reason:why}):t('reader-attachments-download',{name:name()});};
 const label=():string=>{const why=reason();return why?tip():t('reader-attachments-download-label',{name:name(),size:formatBytes(props.item.size)});};
 const click=(event:MouseEvent):void=>{
  event.stopPropagation();
  if(reason()||busy())return;
  setBusy(true);
  void downloadBarItem(props.app,props.emailId,props.item,props.policy).finally(()=>setBusy(false));
 };
 return (
  <button type="button" class="mw-thumb__download" data-testid="attachment-download" data-no-toggle=""
   aria-disabled={reason()?true:undefined} aria-busy={busy()?true:undefined} data-busy={busy()?'':undefined}
   aria-label={label()} data-tooltip={tip()} onClick={click} onDblClick={e=>e.stopPropagation()}>
   {busy()?<Spinner />:<Icon name="download" />}
  </button>
 );
}
