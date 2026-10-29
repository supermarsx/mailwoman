import { fireEvent, render, screen } from '@solidjs/testing-library';
import { afterEach, beforeEach, expect, it, onTestFinished, vi } from 'vitest';
import { TooltipLayer, tooltipDescription } from './TooltipLayer.tsx';

beforeEach(() => { vi.useFakeTimers(); });
afterEach(() => { vi.useRealTimers(); });
const settle = async (ms = 0) => { await vi.advanceTimersByTimeAsync(ms); };

it('adopts titles on focus, preserves existing descriptions and accessible names, and dismisses on Escape', async () => {
  render(() => <><button title="Restore the inherited value" aria-label="Reset" aria-describedby="existing" /><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Reset' });
  fireEvent.focusIn(button);
  await settle();
  const tooltip = screen.getByRole('tooltip');
  expect(tooltip).toHaveTextContent('Restore the inherited value');
  expect(button.getAttribute('aria-describedby')).toContain(tooltip.id);
  expect(button.getAttribute('aria-describedby')).toContain('existing');
  expect(button).not.toHaveAttribute('title');
  fireEvent.keyDown(button, { key: 'Escape' });
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  expect(button).toHaveAttribute('aria-describedby', 'existing');
  expect(button).toHaveAccessibleName('Reset');
  expect(button).not.toHaveAttribute('title');
  fireEvent.pointerOut(button, { relatedTarget: document.body });
  expect(button).not.toHaveAttribute('title');
  expect(button).toHaveAttribute('data-tooltip-title', 'Restore the inherited value');
});

it('shows hover immediately, prefers explicit descriptions, and cleans up removed targets', async () => {
  render(() => <><button title="Refresh" data-tooltip="Fetch the latest messages from this account">Refresh</button><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Refresh' });
  fireEvent.pointerOver(button, { pointerType: 'mouse' });
  expect(screen.getByRole('tooltip')).toHaveTextContent('Fetch the latest messages from this account');
  button.remove();
  await settle(200);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
});

it('adopts non-control and newly added titles without native duplicates and updates visible text',async()=>{
  render(()=><><span title="Unread messages">12</span><TooltipLayer/></>);
  const badge=screen.getByText('12');
  expect(badge).not.toHaveAttribute('title');
  fireEvent.pointerOver(badge,{pointerType:'mouse'});
  expect(screen.getByRole('tooltip')).toHaveTextContent('Unread messages');
  badge.setAttribute('title','Updated unread count');await settle();
  expect(screen.getByRole('tooltip')).toHaveTextContent('Updated unread count');expect(badge).not.toHaveAttribute('title');
  const disabled=document.createElement('button');disabled.disabled=true;disabled.title='Unavailable until connected';document.body.append(disabled);await settle();
  fireEvent.pointerOver(disabled,{pointerType:'mouse'});
  expect(screen.getByRole('tooltip')).toHaveTextContent('Unavailable until connected');expect(disabled).not.toHaveAttribute('title');disabled.remove();
});

it('shows a different hovered control immediately after pointer activation',()=>{
  render(()=><><button aria-label="First"/><button aria-label="Second"/><TooltipLayer/></>);
  const first=screen.getByRole('button',{name:'First'}),second=screen.getByRole('button',{name:'Second'});
  fireEvent.pointerOver(first,{pointerType:'mouse',clientX:10,clientY:10});fireEvent.pointerDown(first,{pointerType:'mouse',clientX:10,clientY:10});fireEvent.focusIn(first);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  fireEvent.pointerOver(second,{pointerType:'mouse',clientX:60,clientY:10});expect(screen.getByRole('tooltip')).toHaveTextContent('Second');
});

it('opens no help for a control that the layout moves under a pointer that has not moved since the press (t83)',()=>{
  // The reading pane opens beside the row just clicked and puts its Pin button
  // under the still pointer; Chromium then fires pointerover at the press point.
  // jsdom has no PointerEvent, so fired pointer events would carry no coordinates.
  class TestPointerEvent extends MouseEvent { readonly pointerType:string; constructor(type:string,init:PointerEventInit={}){super(type,init);this.pointerType=init.pointerType??'';} }
  vi.stubGlobal('PointerEvent',TestPointerEvent);
  onTestFinished(()=>{vi.unstubAllGlobals();});
  render(()=><><div role="row" aria-label="Message"/><button aria-label="Pin"/><TooltipLayer/></>);
  const row=screen.getByRole('row',{name:'Message'}),pin=screen.getByRole('button',{name:'Pin'});
  fireEvent.pointerDown(row,{pointerType:'mouse',clientX:752,clientY:200});
  fireEvent.pointerOver(pin,{pointerType:'mouse',clientX:752,clientY:200});
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  // The same spot after the pointer has moved away and back is real pointing.
  fireEvent.pointerMove(document.body,{pointerType:'mouse',clientX:700,clientY:260});
  fireEvent.pointerOver(pin,{pointerType:'mouse',clientX:752,clientY:200});
  expect(screen.getByRole('tooltip')).toHaveTextContent('Pin');
});

it('retains the accessible name of title-only icon controls',()=>{
  render(()=><><button title="Restore defaults"><svg aria-hidden="true"/></button><TooltipLayer/></>);
  const button=screen.getByRole('button',{name:'Restore defaults'});
  expect(button).not.toHaveAttribute('title');fireEvent.pointerOver(button,{pointerType:'mouse'});
  expect(screen.getByRole('tooltip')).toHaveTextContent('Restore defaults');
});

it('keeps touch activation unobstructed and ignores editable user content', async () => {
  const click = vi.fn();
  render(() => <><button aria-label="Archive" onClick={click} /><div contenteditable="true"><button title="User supplied title">Content</button></div><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Archive' });
  const touch = new Event('pointerdown', { bubbles: true });
  Object.defineProperty(touch, 'pointerType', { value: 'touch' });
  button.dispatchEvent(touch);
  fireEvent.focusIn(button); fireEvent.click(button);
  await settle(400);
  expect(click).toHaveBeenCalledOnce();
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  expect(tooltipDescription(screen.getByRole('button', { name: 'Content' }))).toBeNull();
});

it('clamps placement and dismisses an anchor scrolled out of the viewport', async () => {
  render(() => <><button aria-label="Close" /><TooltipLayer /></>);
  const button = screen.getByRole('button', { name: 'Close' });
  const rect = vi.spyOn(button, 'getBoundingClientRect').mockReturnValue({ left: 0, right: 20, top: 0, bottom: 20, width: 20, height: 20 } as DOMRect);
  fireEvent.focusIn(button); await settle();
  expect(screen.getByRole('tooltip').style.left).toBe('10px');
  expect(Number.parseFloat(screen.getByRole('tooltip').style.top)).toBeGreaterThanOrEqual(8);
  rect.mockReturnValue({ left: 0, right: 20, top: -200, bottom: -180, width: 20, height: 20 } as DOMRect);
  fireEvent.scroll(document); await settle(20);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
});

it('does not reopen over navigation after mouse activation, but retains keyboard help', async () => {
  render(() => <><button aria-label="Open folders and apps" /><button aria-label="Compose" /><TooltipLayer /></>);
  const drawer = screen.getByRole('button', { name: 'Open folders and apps' });
  fireEvent.pointerDown(drawer, { pointerType: 'mouse' });
  fireEvent.focusIn(drawer);
  await settle(400);
  expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();
  fireEvent.keyDown(drawer, { key: 'Tab' });
  fireEvent.focusIn(screen.getByRole('button', { name: 'Compose' }));
  await settle();
  expect(screen.getByRole('tooltip')).toHaveTextContent('Compose');
});

it('cancels every pending dismissal when returning from the tooltip to its anchor', async () => {
  render(() => <><button aria-label="Archive"/><TooltipLayer/></>);
  const button=screen.getByRole('button',{name:'Archive'});
  fireEvent.pointerOver(button,{pointerType:'mouse'});
  const tooltip=screen.getByRole('tooltip');
  fireEvent.pointerOut(tooltip,{relatedTarget:document.body,pointerType:'mouse'});
  fireEvent.pointerLeave(tooltip,{relatedTarget:document.body,pointerType:'mouse'});
  await settle(50);
  fireEvent.pointerOver(button,{pointerType:'mouse'});
  await settle(150);
  expect(screen.getByRole('tooltip')).toBe(tooltip);
});
it('does not dismiss help when pointer movement happens outside its anchor and bubble', async () => {
  render(() => <><button aria-label="Archive"/><span>Other content</span><TooltipLayer/></>);
  fireEvent.pointerOver(screen.getByRole('button',{name:'Archive'}),{pointerType:'mouse'});
  fireEvent.pointerOut(screen.getByText('Other content'),{relatedTarget:document.body,pointerType:'mouse'});
  await settle(150);
  expect(screen.getByRole('tooltip')).toBeInTheDocument();
});

it('uses the detached document for hover, native title adoption, and cleanup', async()=>{
  const frame=document.createElement('iframe');document.body.append(frame);
  const child=frame.contentDocument!;
  const button=child.createElement('button');button.title='Detached action';child.body.append(button);
  const adopted=document.createElement('button');adopted.setAttribute('data-tooltip','Adopted action');child.body.append(adopted);
  const mounted=render(()=><TooltipLayer document={child}/>);
  expect(button).not.toHaveAttribute('title');
  fireEvent.pointerOver(button,{pointerType:'mouse'});await settle();
  expect(child.querySelector('[role="tooltip"]')?.textContent).toBe('Detached action');
  expect(document.querySelector('[role="tooltip"]')).toBeNull();
  fireEvent.pointerOver(adopted,{pointerType:'mouse'});
  expect(child.querySelector('[role="tooltip"]')?.textContent).toBe('Adopted action');
  mounted.unmount();
  expect(child.querySelector('[role="tooltip"]')).toBeNull();
  expect(button).toHaveAttribute('title','Detached action');
  frame.remove();
});

it('leaves a frame its title, which is its accessible name, while other titles are still adopted', async () => {
  render(() => <><iframe title="Message body" /><span title="Unread">3</span><TooltipLayer /></>);
  await settle();
  const frame = document.querySelector('iframe')!;
  expect(frame).toHaveAttribute('title', 'Message body');
  expect(frame).not.toHaveAttribute('data-tooltip-title');
  const badge = screen.getByText('3');
  expect(badge).not.toHaveAttribute('title');
  expect(badge).toHaveAttribute('data-tooltip-title', 'Unread');
  // A title set later (the reader re-rendering its frame) is not stripped either.
  frame.setAttribute('title', 'Message body (updated)');
  await settle();
  expect(frame).toHaveAttribute('title', 'Message body (updated)');
});

it('opens below its anchor inside a data-tooltip-placement="below" row, so it never covers the controls above (t68)', async () => {
  vi.stubGlobal('innerHeight', 800);
  render(() => <><button aria-label="More actions" /><section data-tooltip-placement="below"><span tabIndex={0} data-tooltip="2 remote images blocked" /></section><span tabIndex={0} data-tooltip="Elsewhere" /><TooltipLayer /></>);
  const [summary, other] = [...document.querySelectorAll<HTMLElement>('[data-tooltip]')];
  const box = { left: 100, right: 300, top: 200, bottom: 230, width: 200, height: 30 } as DOMRect;
  vi.spyOn(summary!, 'getBoundingClientRect').mockReturnValue(box);
  vi.spyOn(other!, 'getBoundingClientRect').mockReturnValue(box);
  fireEvent.focusIn(summary!); await settle();
  expect(Number.parseFloat(screen.getByRole('tooltip').style.top)).toBe(238);
  fireEvent.focusOut(summary!); fireEvent.focusIn(other!); await settle();
  // Without the hint the bubble still prefers the space above.
  expect(Number.parseFloat(screen.getByRole('tooltip').style.top)).toBeLessThan(200);
  vi.unstubAllGlobals();
});
