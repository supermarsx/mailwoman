import { mailHeaderValues } from '../../api/mailHeaders.ts';
/** One rule or symbol as the header reports it; `score` is null when the header names the rule without its points. */
export interface SpamRule { name: string; score: number | null; description: string }
export interface SpamScore { source: 'SpamAssassin'|'Rspamd'; score: number; threshold: number | null; spam: boolean | null; rules: SpamRule[] }
/** The colour band of a score: spam at or over the threshold (or a spam verdict), ham at or under zero, neutral between. */
export type SpamBand = 'ham' | 'neutral' | 'spam';
const MAX_RULES = 40;
const number = (text: string | undefined): number | null => { if (text === undefined || !/^-?\d+(?:\.\d+)?$/.test(text)) return null; const value=Number(text);return Number.isFinite(value)&&Math.abs(value)<=100000 ? value : null; };
const clean = (text: string): string => text.replace(/\s+/g, ' ').trim().slice(0, 512);
export function spamBand(score: SpamScore): SpamBand {
  if (score.spam === true || (score.threshold !== null && score.score >= score.threshold)) return 'spam';
  return score.score <= 0 ? 'ham' : 'neutral';
}
/** Largest contribution first; rules reported without points last, by name. */
export function sortRules(rules: readonly SpamRule[]): SpamRule[] {
  return [...rules].sort((a, b) => a.score === null || b.score === null ? (a.score === null ? 1 : 0) - (b.score === null ? 1 : 0) || a.name.localeCompare(b.name) : b.score - a.score || a.name.localeCompare(b.name));
}
/** `X-Spam-Report` lines: `* 3.5 RULE_NAME Description` with `*   more` continuations. */
function reportRules(report: string | undefined, known: Set<string>): Map<string, SpamRule> {
  const out = new Map<string, SpamRule>();
  if (!report) return out;
  const starts = [...report.matchAll(/(?:^|\s)(?:\*\s+)?(-?\d+(?:\.\d+)?)\s+([A-Z][A-Z0-9_]{1,63})(?=\s|$)/g)].filter(match => known.size === 0 || known.has(match[2]!));
  starts.forEach((match, index) => {
    const end = starts[index + 1]?.index ?? report.length;
    const description = clean(report.slice(match.index! + match[0].length, end).replace(/(?:^|\s)\*(?=\s|$)/g, ' '));
    const score = number(match[1]);
    if (!out.has(match[2]!)) out.set(match[2]!, { name: match[2]!, score, description });
  });
  return out;
}
export function parseSpamScores(raw: string): SpamScore[] {
  const headers=mailHeaderValues(raw);const out:SpamScore[]=[];
  const status=headers.get('x-spam-status')?.[0];
  if(status){const score=number(/\bscore\s*=\s*(-?\d+(?:\.\d+)?)/i.exec(status)?.[1]);if(score!==null){
    const threshold=number(/\brequired\s*=\s*(\d+(?:\.\d+)?)/i.exec(status)?.[1]);const verdict=/^\s*(yes|no)\b/i.exec(status)?.[1]?.toLowerCase();
    const tests=(/\btests\s*=\s*(.*?)(?:\s+\w+=(?![-\d.])|$)/i.exec(status)?.[1]??'').split(/[\s,]+/).filter(rule=>/^[A-Z][A-Za-z0-9_]{0,127}(?:=-?\d+(?:\.\d+)?)?$/.test(rule)).slice(0,MAX_RULES);
    const named=tests.map(test=>{const [name,points]=test.split('=',2);return {name:name!,score:number(points?.trim())};}).filter(rule=>rule.name);
    const report=reportRules(headers.get('x-spam-report')?.[0],new Set(named.map(rule=>rule.name)));
    const rules=named.length?named.map(rule=>{const detail=report.get(rule.name);return {name:rule.name,score:rule.score??detail?.score??null,description:detail?.description??''};}):[...report.values()].slice(0,MAX_RULES);
    out.push({source:'SpamAssassin',score,threshold,spam:verdict==='yes'?true:verdict==='no'?false:null,rules:sortRules(rules)});}}
  const result=headers.get('x-spamd-result')?.[0]??headers.get('x-rspamd-result')?.[0];
  if(result){const match=/\[\s*(-?\d+(?:\.\d+)?)\s*\/\s*(\d+(?:\.\d+)?)\s*\]/.exec(result);const score=number(match?.[1]);if(score!==null){
    const threshold=number(match?.[2]);const verdict=/\b(True|False)\s*\[/i.exec(result)?.[1]?.toLowerCase();
    const rules=[...result.matchAll(/\b([A-Z][A-Z0-9_]{0,63})\((-?\d+(?:\.\d+)?)\)(?:\[([^\]]*)\])?/g)].slice(0,MAX_RULES).map(rule=>({name:rule[1]!,score:number(rule[2]),description:clean((rule[3]??'').replace(/,(?=\S)/g,', '))}));
    out.push({source:'Rspamd',score,threshold,spam:verdict==='true'?true:verdict==='false'?false:null,rules:sortRules(rules)});}}
  if(!out.some(item=>item.source==='Rspamd')){const score=number(headers.get('x-rspamd-score')?.[0]?.trim());if(score!==null)out.push({source:'Rspamd',score,threshold:null,spam:null,rules:[]});}
  return out;
}
