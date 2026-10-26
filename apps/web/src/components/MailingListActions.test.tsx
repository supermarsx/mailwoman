import {it,expect,vi} from 'vitest';
import {render,screen,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {MailingListActions} from './MailingListActions.tsx';
const headers='List-Unsubscribe: <https://list.example.org/leave?token=opaque>, <mailto:leave@example.org?subject=unsubscribe&body=please>\r\nList-Post: NO';
it('requires destination review before showing a no-referrer external link',async()=>{
 render(()=><MailingListActions headers={headers} scope="a:m" active/>);
 fireEvent.click(await screen.findByText('Mailing list actions'));expect(screen.queryByRole('link')).not.toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'list.example.org'}));const link=screen.getByRole('link',{name:'Open web page'});
 expect(link).toHaveAttribute('rel','noopener noreferrer');expect(link).toHaveAttribute('referrerpolicy','no-referrer');expect(link).toHaveAttribute('href','https://list.example.org/leave?token=opaque');
 expect(screen.getByText('This list says it does not accept posts by email.')).toBeVisible();
});
it('opens an ordinary user-reviewed draft only after explicit action',async()=>{
 const compose=vi.fn();render(()=><MailingListActions headers={headers} scope="a:m" active onCompose={compose}/>);
 fireEvent.click(await screen.findByText('Mailing list actions'));fireEvent.click(screen.getByRole('button',{name:'leave@example.org'}));expect(compose).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:'Review email draft'}));expect(compose).toHaveBeenCalledWith(expect.objectContaining({mode:'new',to:'leave@example.org',cc:'',subject:'unsubscribe',bodyText:'please',attachments:[],references:[],inReplyTo:[]}));
 expect(compose.mock.calls[0]![0]).not.toHaveProperty('source');
});
it('clears a reviewed destination when account/message scope changes',async()=>{
 const [scope,setScope]=createSignal('a:m');render(()=><MailingListActions headers={headers} scope={scope()} active/>);
 fireEvent.click(await screen.findByText('Mailing list actions'));fireEvent.click(screen.getByRole('button',{name:'list.example.org'}));setScope('b:m');expect(screen.queryByRole('link')).not.toBeInTheDocument();
});
