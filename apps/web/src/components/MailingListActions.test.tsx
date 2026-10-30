import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent,within} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {MailingListActions} from './MailingListActions.tsx';
import {mailingListName} from '../api/mailingList.ts';
const headers='List-Id: "Example news" <news.example.org>\r\nList-Unsubscribe: <https://list.example.org/leave?token=opaque>, <mailto:leave@example.org?subject=unsubscribe&body=please>\r\nList-Archive: <https://list.example.org/archive>\r\nList-Help: <mailto:help@example.org>\r\nList-Post: NO';
const dialog=()=>screen.getByRole('dialog');

it('is one notice bar: the list name, Unsubscribe, and the other commands in a menu',async()=>{
 render(()=><MailingListActions headers={headers} scope="a:m" active/>);
 const bar=await screen.findByTestId('mailing-list-bar');
 expect(bar).toHaveAttribute('data-notice-bar','');expect(bar).toHaveTextContent(/^Mailing list · \u2068Example news\u2069/);
 expect(within(bar).getByRole('button',{name:'Unsubscribe'})).toHaveAttribute('aria-haspopup','dialog');
 // The explanation and the no-post note live in the summary tooltip, not the row.
 const tip=bar.querySelector('[data-tooltip]')!.getAttribute('data-tooltip')!;expect(tip).toContain('not independently verified');expect(tip).toContain('does not accept posts by email');
 fireEvent.click(within(bar).getByRole('button',{name:'More list actions'}));
 const items=screen.getAllByRole('menuitem').map(item=>item.textContent);expect(items).toEqual(['Help','List archive']);
});
it('requires destination review before showing a no-referrer external link',async()=>{
 render(()=><MailingListActions headers={headers} scope="a:m" active/>);
 fireEvent.click(await screen.findByRole('button',{name:'Unsubscribe'}));
 expect(within(dialog()).getByRole('heading',{name:'Unsubscribe'})).toBeVisible();expect(screen.queryByRole('link')).not.toBeInTheDocument();
 fireEvent.click(within(dialog()).getByRole('button',{name:'list.example.org'}));
 const link=screen.getByRole('link',{name:'Open in a new tab'});
 expect(link).toHaveAttribute('rel','noopener noreferrer');expect(link).toHaveAttribute('referrerpolicy','no-referrer');expect(link).toHaveAttribute('href','https://list.example.org/leave?token=opaque');
});
it('opens an ordinary user-reviewed draft only after explicit action',async()=>{
 const compose=vi.fn();render(()=><MailingListActions headers={headers} scope="a:m" active onCompose={compose}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Unsubscribe'}));
 fireEvent.click(within(dialog()).getByRole('button',{name:'leave@example.org'}));expect(compose).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Review a new message'}));expect(compose).toHaveBeenCalledWith(expect.objectContaining({mode:'new',to:'leave@example.org',cc:'',subject:'unsubscribe',bodyText:'please',attachments:[],references:[],inReplyTo:[]}));
 expect(compose.mock.calls[0]![0]).not.toHaveProperty('source');
});
it('clears a reviewed destination when account/message scope changes',async()=>{
 const [scope,setScope]=createSignal('a:m');render(()=><MailingListActions headers={headers} scope={scope()} active/>);
 fireEvent.click(await screen.findByRole('button',{name:'Unsubscribe'}));expect(screen.getByRole('dialog')).toBeInTheDocument();setScope('b:m');expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
});
it('shows nothing for a message without list commands',()=>{
 render(()=><MailingListActions headers={'List-Id: <x.example.org>'} scope="a:m" active/>);expect(screen.queryByTestId('mailing-list-bar')).toBeNull();
});
it('reads the List-Id phrase, else the id, and refuses repeated or control-laden values',()=>{
 expect(mailingListName('List-Id: "Example news" <news.example.org>')).toBe('Example news');
 expect(mailingListName('List-Id: <news.example.org>')).toBe('news.example.org');
 expect(mailingListName('List-Id: <a.example.org>\r\nList-Id: <b.example.org>')).toBeNull();
 expect(mailingListName('Subject: x')).toBeNull();
});
