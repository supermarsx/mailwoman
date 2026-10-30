import {describe,it,expect} from 'vitest';
import {render} from '@solidjs/testing-library';
import {quoteForForward} from './quote.ts';
import {RichTextEditor,type RichTextApi} from './RichTextEditor.tsx';
import {inertBody,docFromHtml,htmlFromDoc} from './richtext.ts';
import {applyEmailStyles} from './safeEmailStyles.ts';
import {tablesWithinLimits} from './richCommands.ts';

const original='<div style="font-family:Arial;font-size:14px;color:#123456;line-height:1.4;text-align:center;width:600px"><p>Newsletter opening</p><table width="600" border="0" cellpadding="12" style="border-collapse:separate"><tr><td valign="middle" bgcolor="#eeeeee"><h2>Heading</h2><ul><li>First</li><li>Second</li></ul><table style="width:240px;border-collapse:collapse"><tr><td style="padding:8px;border:2px solid #112233">Nested content</td></tr></table></td></tr></table><div style="text-align:right">Closing line</div><img src="https://tracker.invalid/pixel"><script>bad()</script></div>';

describe('forwarded formatting through the real editor',()=>{
 it('rejects forged attribute keys and bounds nested table depth and total cells without losing text',()=>{
  const element=document.createElement('div');
  expect(()=>applyEmailStyles(element,JSON.parse('{"constructor":"x","toString":"x","background-image":"url(https://evil.invalid)","color":"#123456"}'))).not.toThrow();
  expect(element.style.cssText).toBe('color: rgb(18, 52, 86);');
  let nested='<p>Deep text</p>';for(let index=0;index<6;index++)nested=`<table><tr><td>${nested}</td></tr></table>`;
  const doc=docFromHtml(nested);expect(tablesWithinLimits(doc)).toBe(true);expect(doc.textContent).toContain('Deep text');expect(inertBody(htmlFromDoc(doc)).querySelectorAll('table')).toHaveLength(3);
 });
 it('retains safe containers, inherited typography, nested tables and block content through outgoing serialization',async()=>{
  const quote=await quoteForForward({html:original},['Forwarded message','Subject: Newsletter']);
  let outgoing='';let api:RichTextApi|undefined;
  render(()=><RichTextEditor initialHtml={quote.html} ariaLabel="Forward body" onReady={value=>{api=value;}} onChange={html=>{outgoing=html;}}/>);
  api!.appendHtml('<p>My forwarding note</p>');
  const body=inertBody(outgoing);
  const container=body.querySelector('div')!;expect(container).not.toBeNull();
  expect(container.style.fontFamily).toBe('Arial');expect(container.style.fontSize).toBe('14px');expect(container.style.color).toBe('rgb(18, 52, 86)');expect(container.style.lineHeight).toBe('1.4');
  expect(body.querySelectorAll('table')).toHaveLength(2);
  expect((body.querySelector('table') as HTMLTableElement).style.width).toBe('600px');
  expect((body.querySelector('table table') as HTMLTableElement).style.width).toBe('240px');
  const cell=body.querySelector('td')!;expect(cell.querySelector('h2')?.textContent).toBe('Heading');expect(cell.querySelectorAll('li')).toHaveLength(2);expect(cell.style.verticalAlign).toBe('middle');expect(cell.style.paddingTop).toBe('12px');
  expect(cell.style.borderWidth).not.toBe('1px');
  expect((body.querySelector('table table td') as HTMLElement).style.borderTopWidth).toBe('2px');
  expect(outgoing).toContain('My forwarding note');expect(outgoing).not.toMatch(/<img|<script|tracker\.invalid|onclick|url\(/i);
  expect(htmlFromDoc(docFromHtml(outgoing))).toBe(outgoing);
 });
 it('keeps safe block boundaries while rejecting active CSS, arbitrary positioning and remote resources',async()=>{
  const quote=await quoteForForward({html:'<div style="text-align:right;position:fixed;z-index:99999;background:url(https://evil.invalid/x);color:#123456" onclick="bad()">Line one</div><div>Line two</div><table style="width:40%;background-image:url(https://evil.invalid/t)"><tr><td>Cell</td></tr></table>'},['Forward']);
  const body=inertBody(quote.html);expect(body.querySelectorAll('div')).toHaveLength(2);expect((body.querySelector('div') as HTMLElement).style.textAlign).toBe('right');expect((body.querySelector('table') as HTMLElement).style.width).toBe('40%');
  expect(quote.html).not.toMatch(/position|z-index|background-image|evil\.invalid|onclick|url\(/i);
 });
});
