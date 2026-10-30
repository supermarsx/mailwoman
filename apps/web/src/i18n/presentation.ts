import {createSignal, type Accessor} from 'solid-js';
import {leetspeakText} from '../viewers/leetspeak.ts';

const [mailSource, setMailSource] = createSignal<Accessor<boolean>>(()=>false);
const [adminMode, setAdminMode] = createSignal<boolean|null>(null);

/** The policy store supplies the current effective account value; clearing that
 * store on logout/account switch automatically restores ordinary text. */
export function configureMailLeetspeak(source:Accessor<boolean>):void {
  setMailSource(()=>source);
}
/** Console preferences are independent of the mailbox that may also be open. */
export function setAdminLeetspeak(value:boolean|null):void { setAdminMode(value); }
export const leetspeakEnabled = ():boolean => adminMode() ?? mailSource()();
// Sparse message previews and older cached tabs can carry null despite the wire
// model's string annotation. Preserve Solid's empty-text behavior in both modes.
export const displayText = (text:string|null|undefined):string => {
  const value=text??'';
  return leetspeakEnabled() ? leetspeakText(value) : value;
};
