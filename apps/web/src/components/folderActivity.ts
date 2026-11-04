import {createSignal} from 'solid-js';
import type {Client} from '../api/client.ts';
import {lastActivity} from '../api/folderBulk.ts';
/** Newest message time per folder, fetched only once a folder list is ordered by last activity. */
const [activity,setActivity]=createSignal<{account:string;times:ReadonlyMap<string,number>}|null>(null);
export function folderActivity(account:string|null|undefined):ReadonlyMap<string,number>|undefined {
 const value=activity();return value&&value.account===account?value.times:undefined;
}
export async function loadFolderActivity(client:Pick<Client,'jmap'>,accountId:string,ids:readonly string[],current:()=>boolean):Promise<void> {
 try{const times=await lastActivity(client,accountId,ids,{current});if(current())setActivity({account:accountId,times});}catch{/* the order stays as it was */}
}
