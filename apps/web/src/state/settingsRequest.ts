// A request, from anywhere in the mail screen, to open a Settings section. The
// mail screen (screens/Mailbox.tsx) owns the panes, so it watches this signal and
// opens the Settings tab on the section; menus only say what they want.
import {createSignal} from 'solid-js';

export interface SettingsRequest {section:string;seq:number;card?:string}
const [request,setRequest]=createSignal<SettingsRequest|null>(null);
let seq=0;
export const settingsRequest=request;
export function requestSettings(section:string,card?:string):void {setRequest({section,seq:++seq,...(card?{card}:{})});}
