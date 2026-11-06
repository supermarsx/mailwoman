// Parsing for the message Properties dialog: the raw header block as a list,
// the spam verdict headers, and the hop-to-hop delays of the Received chain.
import {parseSpamScores,type SpamScore} from '../security/spamScore.ts';

export interface HeaderLine {name:string;value:string}

/** The header block as ordered name/value pairs, folded lines joined. Bounded: 512 headers, 8 KB a value. */
export function headerList(raw:string):HeaderLine[] {
 const head=raw.split(/\r?\n\r?\n/,1)[0]!.replace(/\r?\n[ \t]+/g,' ');
 const out:HeaderLine[]=[];
 for(const line of head.split(/\r?\n/)){
  const match=/^([!-9;-~]+):[ \t]*(.*)$/.exec(line);
  if(!match)continue;
  out.push({name:match[1]!,value:match[2]!.slice(0,8192)});
  if(out.length>=512)break;
 }
 return out;
}

/** Headers whose name contains `filter` or whose value does (case-insensitive); an empty filter keeps all. */
export function filterHeaders(list:readonly HeaderLine[],filter:string):HeaderLine[] {
 const needle=filter.trim().toLowerCase();
 return needle===''?[...list]:list.filter(h=>h.name.toLowerCase().includes(needle)||h.value.toLowerCase().includes(needle));
}

export interface SpamVerdict {score:SpamScore[];status:string|null;flag:boolean|null;level:string|null}
/** X-Spam-Status / X-Spam-Flag / X-Spam-Level beside the scanners' scores (SpamAssassin, Rspamd). */
export function spamVerdict(raw:string):SpamVerdict {
 const list=headerList(raw),first=(name:string)=>list.find(h=>h.name.toLowerCase()===name)?.value??null;
 const flag=first('x-spam-flag');
 return {score:parseSpamScores(raw),status:first('x-spam-status'),flag:flag===null?null:/^yes$/i.test(flag.trim()),level:first('x-spam-level')};
}

/** The header block of a message as bytes of a `.eml` download of its headers only. */
export function headersOnly(raw:string):string {return `${raw.split(/\r?\n\r?\n/,1)[0]!.replace(/\r?\n/g,'\r\n')}\r\n\r\n`;}

/** `1.2 s`, `3 min`, `2 h 5 min`; null for an unknown delay. */
export function hopDelayText(ms:number|null):string|null {
 if(ms===null||!Number.isFinite(ms))return null;
 if(ms<1000)return `${Math.max(0,Math.round(ms))} ms`;
 if(ms<60_000)return `${(ms/1000).toFixed(ms<10_000?1:0)} s`;
 const minutes=Math.round(ms/60_000);
 return minutes<60?`${minutes} min`:`${Math.floor(minutes/60)} h ${minutes%60} min`;
}

/** Bar widths (0..100) for a Received chain, relative to the slowest hop. */
export function hopBars(delays:ReadonlyArray<number|null>):number[] {
 const max=Math.max(0,...delays.map(d=>d??0));
 return delays.map(d=>max===0||d===null?0:Math.max(2,Math.round((d/max)*100)));
}
