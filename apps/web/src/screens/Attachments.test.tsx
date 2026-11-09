import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { Attachments } from './Attachments.tsx';
import type { AttachmentItem } from '../viewers/attachments.ts';
import { Suspense } from 'solid-js';

const items: AttachmentItem[] = [
  { emailId: 'e1', blobId: 'b1', name: 'Q3-report.pdf', mime: 'application/pdf', size: 2_000_000, from: 'Alice <alice@example.org>', subject: 'Q3', receivedAt: '2026-03-01T00:00:00Z' },
  { emailId: 'e2', blobId: 'b2', name: 'logo.png', mime: 'image/png', size: 40_000, from: 'Bob <bob@corp.com>', subject: 'Logo', receivedAt: '2026-01-15T00:00:00Z' },
  { emailId: 'e3', blobId: 'b3', name: 'demo.mp4', mime: 'video/mp4', size: 8_000_000, from: 'Alice <alice@example.org>', subject: 'Demo', receivedAt: '2026-02-10T00:00:00Z' },
];

describe('Attachments module', () => {
  it('sorts numeric sizes and exposes filename warnings without downloading files', async () => {
    render(()=><Attachments items={[...items,{...items[0]!,name:'invoice.pdf.exe',blobId:'exe'}]}/>);
    await screen.findByText('invoice.pdf.exe');
    expect(screen.getByRole('table',{name:'Attachments'}).parentElement).toHaveAttribute('tabindex','0');
    fireEvent.click(screen.getByRole('button',{name:'Size'}));
    expect(screen.getByRole('columnheader',{name:'Size'})).toHaveAttribute('aria-sort','ascending');
    expect(screen.getAllByRole('row')[1]).toHaveTextContent('logo.png');
    expect(screen.getByLabelText(/may execute programs/i)).toBeInTheDocument();
  });
  it('searches older pages explicitly and clears account results when filters change', async () => {
    const load = vi.fn(async (position:number) => ({items:position===0?[items[0]!]:[items[1]!],position,nextPosition:position===0?50:null}));
    render(()=><Attachments loadPage={load}/>);
    await screen.findByText('Q3-report.pdf');
    fireEvent.input(screen.getByLabelText('Search attachments'),{target:{value:'logo'}});
    expect(screen.queryByText('logo.png')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button',{name:'Search entire account'}));
    await screen.findByText('logo.png');
    await screen.findByText(/Search complete/);
    expect(screen.queryByRole('navigation',{name:'Attachment pages'})).not.toBeInTheDocument();
    load.mockClear();
    fireEvent.click(screen.getByRole('button',{name:'Refresh attachments'}));
    await screen.findByText(/Search complete/);
    expect(load.mock.calls.map(call=>call[0])).toEqual([0,50]);
    fireEvent.input(screen.getByLabelText('Search attachments'),{target:{value:'report'}});
    await screen.findByText('Q3-report.pdf');
    expect(screen.queryByText(/Search complete/)).not.toBeInTheDocument();
  });
  it('opens the file independently from its parent message', async () => {
    const open = vi.fn(), parent = vi.fn();
    render(() => <Attachments items={items} onOpen={open} onOpenMessage={parent}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Q3-report.pdf'}));
    expect(open).toHaveBeenCalledWith(items[0]);
    expect(parent).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Q3'}));
    expect(parent).toHaveBeenCalledWith(items[0]);
  });
  it('pages by server message positions and permits returning to the previous page', async () => {
    const load = vi.fn(async (position:number) => ({items:position===0?[items[0]!]:[items[1]!],position,nextPosition:position===0?50:null}));
    render(()=><Attachments loadPage={load}/>);
    await screen.findByText('Q3-report.pdf');
    fireEvent.click(screen.getByRole('button',{name:'Next'}));
    await screen.findByText('logo.png');
    expect(load).toHaveBeenLastCalledWith(50);
    expect(screen.getByRole('button',{name:'Next'})).toBeDisabled();
    fireEvent.click(screen.getByRole('button',{name:'Previous'}));
    await screen.findByText('Q3-report.pdf');
    expect(load).toHaveBeenLastCalledWith(0);
  });
  it('keeps the surrounding app visible while the attachment query is pending', async () => {
    let finish!: (value: AttachmentItem[]) => void;
    const pending = new Promise<AttachmentItem[]>(resolve => { finish = resolve; });
    render(() => <Suspense fallback={<p>Whole app loading</p>}><aside>Mailbox navigation</aside><Attachments load={() => pending} /></Suspense>);
    expect(screen.getByText('Mailbox navigation')).toBeVisible();
    expect(screen.queryByText('Whole app loading')).not.toBeInTheDocument();
    finish(items);
    await screen.findByText('Q3-report.pdf');
    expect(screen.getByText('Mailbox navigation')).toBeVisible();
  });
  it('renders a table row per attachment', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('Q3-report.pdf')).toBeInTheDocument());
    expect(screen.getByText('logo.png')).toBeInTheDocument();
    expect(screen.getByText('demo.mp4')).toBeInTheDocument();
  });

  it('filters via the shared search operators', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('Q3-report.pdf')).toBeInTheDocument());

    fireEvent.input(screen.getByLabelText('Search attachments'), {
      target: { value: 'type:video from:alice' },
    });

    await waitFor(() => {
      expect(screen.getByText('demo.mp4')).toBeInTheDocument();
      expect(screen.queryByText('Q3-report.pdf')).not.toBeInTheDocument();
      expect(screen.queryByText('logo.png')).not.toBeInTheDocument();
    });
  });

  it('filters via the type dropdown', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('logo.png')).toBeInTheDocument());

    fireEvent.change(screen.getByLabelText('Filter by type'), { target: { value: 'image' } });

    await waitFor(() => {
      expect(screen.getByText('logo.png')).toBeInTheDocument();
      expect(screen.queryByText('Q3-report.pdf')).not.toBeInTheDocument();
    });
  });

  it('shows an empty state when nothing matches', async () => {
    render(() => <Attachments items={items} />);
    await waitFor(() => expect(screen.getByText('logo.png')).toBeInTheDocument());
    fireEvent.input(screen.getByLabelText('Search attachments'), {
      target: { value: 'filename:nonexistent-zzz' },
    });
    await waitFor(() => expect(screen.getByText('No attachments match.')).toBeInTheDocument());
  });
});

describe('friendly attachment filters',()=>{
 it('combines sender, subject and type filters, and removes an individual chip',async()=>{
  render(()=><Attachments items={items}/>);await screen.findByText('Q3-report.pdf');
  fireEvent.input(screen.getByLabelText('Sender'),{target:{value:'Alice'}});fireEvent.input(screen.getByLabelText('Message subject'),{target:{value:'Demo'}});
  expect(screen.getByText('demo.mp4')).toBeVisible();expect(screen.queryByText('Q3-report.pdf')).toBeNull();
  fireEvent.change(screen.getByLabelText('Filter by type'),{target:{value:'pdf'}});expect(screen.getByText('No attachments match.')).toBeVisible();
  fireEvent.click(screen.getByRole('button',{name:'Remove filter: PDFs'}));expect(screen.getByText('demo.mp4')).toBeVisible();
  fireEvent.click(screen.getByRole('button',{name:'Clear filters'}));expect(screen.getByText('Q3-report.pdf')).toBeVisible();expect(screen.getByText('logo.png')).toBeVisible();
 });
 it('searches filename, sender and subject with ordinary words and keeps advanced constraints',async()=>{
  render(()=><Attachments items={items}/>);await screen.findByText('Q3-report.pdf');
  fireEvent.input(screen.getByLabelText('Search attachments'),{target:{value:'Alice report'}});expect(screen.getByText('Q3-report.pdf')).toBeVisible();expect(screen.queryByText('demo.mp4')).toBeNull();
  fireEvent.input(screen.getByLabelText('Search attachments'),{target:{value:'from:bob'}});fireEvent.input(screen.getByLabelText('Sender'),{target:{value:'Alice'}});expect(screen.getByText('No attachments match.')).toBeVisible();
 });
 it('includes the full end date and validates reversed date and size ranges',async()=>{
  render(()=><Attachments items={[...items,{...items[1]!,name:'late.png',blobId:'late',receivedAt:'2026-01-15T23:59:59Z'}]}/>);await screen.findByText('late.png');
  fireEvent.change(screen.getByLabelText('Received'),{target:{value:'custom'}});fireEvent.input(screen.getByLabelText('Received from'),{target:{value:'2026-01-15'}});fireEvent.input(screen.getByLabelText('Received through'),{target:{value:'2026-01-15'}});
  expect(screen.getByText('late.png')).toBeVisible();expect(screen.getByText('logo.png')).toBeVisible();expect(screen.queryByText('Q3-report.pdf')).toBeNull();
  fireEvent.input(screen.getByLabelText('Received through'),{target:{value:'2026-01-14'}});expect(screen.getByRole('alert')).toHaveTextContent('end date');expect(screen.getByRole('button',{name:'Apply filters'})).toBeDisabled();
  fireEvent.click(screen.getByRole('button',{name:'Clear filters'}));fireEvent.change(screen.getByLabelText('File size'),{target:{value:'custom'}});fireEvent.input(screen.getByLabelText('Minimum size (MB)'),{target:{value:'3'}});fireEvent.input(screen.getByLabelText('Maximum size (MB)'),{target:{value:'9'}});
  expect(screen.getByText('demo.mp4')).toBeVisible();expect(screen.queryByText('Q3-report.pdf')).toBeNull();
  fireEvent.input(screen.getByLabelText('Maximum size (MB)'),{target:{value:'2'}});expect(screen.getByRole('alert')).toHaveTextContent('minimum');
 });
 it('filters by folder paths and extensions without loading attachment bytes',async()=>{
  render(()=><Attachments items={items.map((item,index)=>({...item,mailboxIds:{[index===1?'work':'inbox']:true}}))} mailboxes={[{id:'inbox',name:'Inbox'},{id:'work',name:'Reports',parentId:'inbox'}]}/>);await screen.findByText('logo.png');
  expect(screen.getByRole('option',{name:'Inbox / Reports'})).toBeInTheDocument();fireEvent.change(screen.getByLabelText('Folder'),{target:{value:'work'}});expect(screen.getByText('logo.png')).toBeVisible();expect(screen.queryByText('Q3-report.pdf')).toBeNull();
  fireEvent.click(screen.getByText('More filters'));fireEvent.input(screen.getByLabelText('File extensions'),{target:{value:'pdf'}});expect(screen.getByText('No attachments match.')).toBeVisible();
  fireEvent.click(screen.getByRole('button',{name:'Clear filters'}));expect(screen.getByText('Q3-report.pdf')).toBeVisible();
 });
 it('automatically searches older pages with friendly filters and supports page-only browsing',async()=>{
  const load=vi.fn(async(position:number)=>({items:position===0?[items[0]!]:[items[1]!],position,nextPosition:position===0?50:null}));render(()=><Attachments loadPage={load}/>);await screen.findByText('Q3-report.pdf');
  fireEvent.input(screen.getByLabelText('Sender'),{target:{value:'Bob'}});await screen.findByText('logo.png',{}, {timeout:3000});await screen.findByText(/Search complete/);
  expect(load.mock.calls.map(call=>call[0])).toContain(50);fireEvent.click(screen.getByRole('button',{name:'Browse pages'}));expect(screen.getByLabelText('Search in')).toHaveValue('page');expect(screen.getByText('No attachments match.')).toBeVisible();
  fireEvent.change(screen.getByLabelText('Search in'),{target:{value:'all'}});await screen.findByText('logo.png',{}, {timeout:3000});
 });
});
