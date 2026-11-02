import {normalizeContactAddress} from '../modules/contacts/contactAddress.ts';

export interface ReaderAddressContext {address:string;x:number;y:number;ownerDocument:Document;trigger:HTMLIFrameElement}
/** Translate only a validated, current frame's viewport coordinates. */
export function readerAddressContext(value:unknown,frame:HTMLIFrameElement):ReaderAddressContext|null {
  if(!value||typeof value!=='object'||frame.closest('[hidden],[inert]'))return null;
  const data=value as {address?:unknown;x?:unknown;y?:unknown};
  if(typeof data.address!=='string'||typeof data.x!=='number'||typeof data.y!=='number'||!Number.isFinite(data.x)||!Number.isFinite(data.y))return null;
  const address=normalizeContactAddress(data.address),rect=frame.getBoundingClientRect(),width=frame.clientWidth||rect.width,height=frame.clientHeight||rect.height;
  if(!address||width<=0||height<=0||data.x<0||data.y<0||data.x>width||data.y>height)return null;
  return {address,x:rect.left+data.x*rect.width/width,y:rect.top+data.y*rect.height/height,ownerDocument:frame.ownerDocument,trigger:frame};
}
