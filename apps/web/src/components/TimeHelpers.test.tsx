import { beforeEach, expect, it, vi } from 'vitest';
import { render, screen } from '@solidjs/testing-library';
import { EventTimeComparison, MessageTimes } from './TimeHelpers.tsx';
const data=vi.hoisted(()=>({values:{} as Record<string,{value:unknown}>}));
vi.mock('../api/settings.ts',()=>({settings:()=>data.values}));
beforeEach(()=>{data.values={'time.displayZone':{value:'UTC'},'time.comparisonZone':{value:'America/New_York'}};});
it('shows explicit comparison with no inferred location',()=>{
 render(()=><MessageTimes sent="2026-07-01T12:00:00Z" received="2026-07-01T12:01:00Z"/>);
 expect(screen.getByText('Time in America/New_York')).toBeInTheDocument();
 expect(screen.getByText(/timestamps do not identify/)).toBeInTheDocument();
 expect(document.querySelector('time')).toHaveAttribute('datetime','2026-07-01T12:00:00Z');
});
it('honors the effective helper policy',()=>{
 data.values['time.showHelpers']={value:false};
 const {container}=render(()=><MessageTimes received="2026-07-01T12:00:00Z"/>);
 expect(container).toBeEmptyDOMElement();
});
it('exposes both clock-change instants instead of silently choosing one',()=>{
 render(()=><EventTimeComparison wallTime="2026-10-25T01:30" zone="Europe/London"/>);
 expect(screen.getByRole('status')).toHaveTextContent('occurs twice');
 expect(document.querySelectorAll('dl')).toHaveLength(2);
});
