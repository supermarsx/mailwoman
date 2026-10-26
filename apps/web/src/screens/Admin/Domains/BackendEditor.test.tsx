import { expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { BackendEditor } from './BackendEditor.tsx';
import { domainBackends, domainsApi, type DomainDetail } from '../../../api/admin/domains.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from '../shell/uxTestkit.ts';

const domain: DomainDetail = { name: 'example.org', enabled: true, preferred: false, upstream: { imap: { host: 'mail.example.org', port: 993, tls: 'implicit', auth: 'password' }, smtp: { host: 'mail.example.org', port: 465, tls: 'implicit', auth: 'password' } }, settings: {}, allowlist: [], blocklist: [], updatedAt: 1, mode: 'engine', aliases: [], hosts: [], etag: 'v1', mappingEtag: 'm1', preferenceEtag: 'p1' };

it('saves a dedicated backend page with stable IDs, explicit priority and preserved sibling services', async () => {
  const save = vi.fn<typeof domainsApi.save>(async () => domain); const onSaved = vi.fn(async () => {});
  const { container } = renderInShell(() => <BackendEditor domain={domain} api={{ ...domainsApi, save }} advancedReady={false} onSaved={onSaved} onBack={() => {}} />);
  const id = await screen.findByLabelText(/^Backend ID/);
  fireEvent.input(id, { target: { value: 'secondary-pop3' } });
  fireEvent.change(screen.getByLabelText(/^Protocol and service/), { target: { value: 'pop3' } });
  fireEvent.input(screen.getByLabelText(/^Hostname or IP address/), { target: { value: 'backup.example.org' } });
  fireEvent.input(screen.getByLabelText(/^Priority/), { target: { value: '10' } });
  fireEvent.click(screen.getByLabelText('Preferred for this service'));
  fireEvent.submit(screen.getByRole('button', { name: 'Save backend' }).closest('form')!);
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(save).toHaveBeenCalledWith(domain.name, { upstream: { version: 2, backends: expect.arrayContaining([
    expect.objectContaining({ id: 'legacy-imap', role: 'mail', preferred: false }),
    expect.objectContaining({ id: 'legacy-smtp', role: 'submission' }),
    expect.objectContaining({ id: 'secondary-pop3', protocol: 'pop3', preferred: true, priority: 10, endpoint: expect.objectContaining({ host: 'backup.example.org' }) }),
  ]) } }, 'v1');
  expect(onSaved).toHaveBeenCalledTimes(1);
  expect(screen.getByLabelText('Customize transport security')).toBeDisabled();
  await expectNoAxeViolations(container); expectNoInlineStyle(container);
});

it('previews exact backend username formatting without saving or sending credentials', async () => {
  const save = vi.fn<typeof domainsApi.save>(async () => domain);
  const usernamePreview = vi.fn(async () => ({ authenticationUsername: 'CORP\\Alice', credentialsSent: false as const }));
  renderInShell(() => <BackendEditor domain={domain} backend={domainBackends(domain.upstream)[0]} api={{ ...domainsApi, save, usernamePreview }} advancedReady={true} onSaved={async () => {}} onBack={() => {}} />);
  fireEvent.change(await screen.findByLabelText(/^Authentication username format/), { target: { value: 'domainUser' } });
  fireEvent.input(screen.getByLabelText(/^Authentication domain/), { target: { value: 'CORP' } });
  fireEvent.input(screen.getByLabelText(/^Example email identity/), { target: { value: 'Alice@alias.example' } });
  fireEvent.click(screen.getByRole('button', { name: 'Preview authentication username' }));
  await screen.findByText('CORP\\Alice');
  expect(usernamePreview).toHaveBeenCalledWith('Alice@alias.example', { format: 'domainUser', domain: 'CORP' });
  expect(save).not.toHaveBeenCalled();
  expect(screen.getByLabelText(/^Backend ID/)).toBeDisabled();
  fireEvent.input(screen.getByLabelText(/^Authentication domain/), { target: { value: 'OTHER' } });
  expect(screen.queryByText('CORP\\Alice')).toBeNull();
});

it('preserves drafts and the read ETag after a concurrent update refusal', async () => {
  const save = vi.fn<typeof domainsApi.save>(async () => { throw new Error('412: reload required'); });
  renderInShell(() => <BackendEditor domain={domain} backend={domainBackends(domain.upstream)[0]} api={{ ...domainsApi, save }} advancedReady={true} onSaved={async () => {}} onBack={() => {}} />);
  fireEvent.input(await screen.findByLabelText(/^Hostname or IP address/), { target: { value: 'changed.example.org' } });
  fireEvent.submit(screen.getByRole('button', { name: 'Save backend' }).closest('form')!);
  await screen.findByText('412: reload required');
  expect(screen.getByLabelText(/^Hostname or IP address/)).toHaveValue('changed.example.org');
  expect(save.mock.calls[0]?.[2]).toBe('v1');
});

it('saves ordered authentication methods and only offers the server protocol matrix', async () => {
  const save = vi.fn<typeof domainsApi.save>(async () => domain);
  renderInShell(() => <BackendEditor domain={domain} backend={domainBackends(domain.upstream)[0]} api={{ ...domainsApi, save }} advancedReady={true} authentication={{ imap: ['scram-sha-256-plus', 'scram-sha-256', 'plain', 'login', 'imap-login'], sieve: ['scram-sha-256', 'plain', 'login'] }} onSaved={async () => {}} onBack={() => {}} />);
  fireEvent.click(await screen.findByLabelText('Customize authentication methods'));
  fireEvent.change(screen.getByLabelText(/^Authentication selection/), { target: { value: 'auto' } });
  fireEvent.change(screen.getByLabelText(/^Authentication priority 1/), { target: { value: 'plain' } });
  fireEvent.change(screen.getByLabelText(/^Authentication priority 2/), { target: { value: 'login' } });
  fireEvent.submit(screen.getByRole('button', { name: 'Save backend' }).closest('form')!);
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0]![1].upstream?.backends).toEqual(expect.arrayContaining([expect.objectContaining({ id: 'legacy-imap', endpoint: expect.objectContaining({ authentication: { mode: 'auto', mechanisms: ['plain', 'login'] } }) })]));
  fireEvent.change(screen.getByLabelText(/^Protocol and service/), { target: { value: 'sieve' } });
  fireEvent.click(screen.getByLabelText('Customize authentication methods'));
  expect(screen.queryByRole('option', { name: /^SCRAM-SHA-256-PLUS$/ })).toBeNull();
  expect(screen.queryByRole('option', { name: /^IMAP LOGIN command$/ })).toBeNull();
});

it('roundtrips EWS endpoint, Basic authentication, origin CA and username without socket fields', async () => {
  const ews = {id:'exchange',role:'mail' as const,protocol:'ews' as const,enabled:true,priority:4,ews:{endpointUrl:'https://exchange.example.org/EWS/Exchange.asmx',authentication:'basic' as const,caPem:'-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----'},username:{format:'domainUser' as const,domain:'CORP'}};
  const source = {...domain,upstream:{version:2 as const,backends:[ews]}};
  const save=vi.fn<typeof domainsApi.save>(async()=>source);
  const {container}=renderInShell(()=><BackendEditor domain={source} backend={ews} api={{...domainsApi,save}} advancedReady={true} onSaved={async()=>{}} onBack={()=>{}}/>);
  expect(await screen.findByLabelText(/^Exchange Web Services endpoint/)).toHaveValue(ews.ews.endpointUrl);
  expect(screen.queryByLabelText(/^Hostname or IP address/)).toBeNull();
  expect(screen.queryByLabelText('Customize authentication methods')).toBeNull();
  fireEvent.input(screen.getByLabelText(/^Priority/),{target:{value:'7'}});
  fireEvent.submit(screen.getByRole('button',{name:'Save backend'}).closest('form')!);
  await waitFor(()=>expect(save).toHaveBeenCalledOnce());
  expect(save.mock.calls[0]![1].upstream?.backends).toEqual([{...ews,priority:7}]);
  expect(save.mock.calls[0]![2]).toBe('v1');
  await expectNoAxeViolations(container);
  fireEvent.change(screen.getByLabelText(/^Protocol and service/),{target:{value:'imap'}});
  fireEvent.input(screen.getByLabelText(/^Hostname or IP address/),{target:{value:'mail.example.org'}});
  fireEvent.submit(screen.getByRole('button',{name:'Save backend'}).closest('form')!);
  await waitFor(()=>expect(save).toHaveBeenCalledTimes(2));
  expect(save.mock.calls[1]![1].upstream?.backends?.[0]).not.toHaveProperty('ews');
});

it('refuses selected EWS with active SMTP before any save', async()=>{
 const save=vi.fn<typeof domainsApi.save>(async()=>domain);
 renderInShell(()=><BackendEditor domain={domain} backend={domainBackends(domain.upstream)[0]} api={{...domainsApi,save}} advancedReady={true} onSaved={async()=>{}} onBack={()=>{}}/>);
 fireEvent.change(await screen.findByLabelText(/^Protocol and service/),{target:{value:'ews'}});
 fireEvent.input(screen.getByLabelText(/^Exchange Web Services endpoint/),{target:{value:'https://exchange.test/EWS/Exchange.asmx'}});
 fireEvent.submit(screen.getByRole('button',{name:'Save backend'}).closest('form')!);
 await screen.findByText(/The selected EWS backend supplies submission/);
 expect(save).not.toHaveBeenCalled();
});
