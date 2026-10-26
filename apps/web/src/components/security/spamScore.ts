import { mailHeaderValues } from '../../api/mailHeaders.ts';
export interface SpamScore { source: 'SpamAssassin'|'Rspamd'; score: number; threshold: number | null; spam: boolean | null; rules: string[] }
const number = (text: string | undefined): number | null => { if (text === undefined || !/^-?\d+(?:\.\d+)?$/.test(text)) return null; const value=Number(text);return Number.isFinite(value)&&Math.abs(value)<=100000 ? value : null; };
export function parseSpamScores(raw: string): SpamScore[] {
  const headers=mailHeaderValues(raw);const out:SpamScore[]=[];
  const status=headers.get('x-spam-status')?.[0];
  if(status){const score=number(/\bscore\s*=\s*(-?\d+(?:\.\d+)?)/i.exec(status)?.[1]);if(score!==null){const threshold=number(/\brequired\s*=\s*(\d+(?:\.\d+)?)/i.exec(status)?.[1]);const verdict=/^\s*(yes|no)\b/i.exec(status)?.[1]?.toLowerCase();const rules=/\btests\s*=\s*(.*?)(?:\s+\w+=|$)/i.exec(status)?.[1]??'';out.push({source:'SpamAssassin',score,threshold,spam:verdict==='yes'?true:verdict==='no'?false:null,rules:rules.split(',').map(rule=>rule.trim().slice(0,128)).filter(Boolean).slice(0,20)});}}
  const result=headers.get('x-spamd-result')?.[0]??headers.get('x-rspamd-result')?.[0];
  if(result){const match=/\[\s*(-?\d+(?:\.\d+)?)\s*\/\s*(\d+(?:\.\d+)?)\s*\]/.exec(result);const score=number(match?.[1]);if(score!==null){const threshold=number(match?.[2]);const verdict=/\b(True|False)\s*\[/i.exec(result)?.[1]?.toLowerCase();const rules=[...result.matchAll(/\b([A-Z][A-Z0-9_]{0,63})\((-?\d+(?:\.\d+)?)\)/g)].slice(0,20).map(rule=>`${rule[1]} (${rule[2]})`);out.push({source:'Rspamd',score,threshold,spam:verdict==='true'?true:verdict==='false'?false:null,rules});}}
  if(!out.some(item=>item.source==='Rspamd')){const score=number(headers.get('x-rspamd-score')?.[0]?.trim());if(score!==null)out.push({source:'Rspamd',score,threshold:null,spam:null,rules:[]});}
  return out;
}
