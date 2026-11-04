import {describe,expect,it} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {SecurityPanel} from './SecurityPanel.tsx';
import {overallTone} from './security/model.ts';
import type {SecurityVerdict} from '../api/security-types.ts';

const verdict={
 emailId:'M1',plainLanguage:'DMARC failed for this message.',
 auth:{dkim:{result:'fail',domain:'example.com',selector:'s'},spf:{result:'fail',domain:'example.com'},dmarc:{result:'fail',policy:'reject',aligned:false},arc:{result:'none',chainLength:0}},
 received:[],signature:null,encryption:{kind:'none',isEncrypted:false,decryptsClientSide:false},attachments:[],anomalies:[],
} as unknown as SecurityVerdict;

describe('SecurityPanel with the authentication status hidden (mail.showAuthStatus off)',()=>{
 it('shows the DMARC result and the SPF, DKIM and DMARC rows by default',()=>{
  render(()=><SecurityPanel verdict={verdict} inline initiallyExpanded/>);
  expect(screen.getByRole('button',{name:/DMARC failed/})).toBeInTheDocument();
  expect(screen.getByText('SPF')).toBeInTheDocument();expect(screen.getByText('DKIM')).toBeInTheDocument();
 });
 it('shows no authentication result, row or summary of one',()=>{
  render(()=><SecurityPanel verdict={verdict} inline hideAuth initiallyExpanded/>);
  expect(screen.queryByText('SPF')).toBeNull();expect(screen.queryByText('DKIM')).toBeNull();expect(screen.queryByText('DMARC')).toBeNull();
  expect(screen.queryByText(/DMARC failed/)).toBeNull();
  expect(screen.getByRole('button',{name:'Message security details'})).toBeInTheDocument();
 });
 it('leaves the authentication results out of the chip tone',()=>{
  expect(overallTone(verdict)).toBe('bad');
  expect(overallTone(verdict,false)).toBe('neutral');
 });
});
