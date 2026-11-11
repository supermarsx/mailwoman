import { expect, it } from 'vitest';
import { correlateReports, type ReceiptTarget } from './messageReceipts.ts';

const target: ReceiptTarget = {
  messageId: 'sent-1@example.test', recipients: ['alice@example.test', 'bob@example.test'],
  after: null, readRequested: true, deliveryRequested: true,
};
const receipt = { originalMessageId: '<sent-1@example.test>', finalRecipient: 'alice@EXAMPLE.TEST', disposition: 'displayed' };
const report = {
  reportingMta: 'mx.example.test', envelopeId: null, arrivalDate: null, originalMessageId: 'sent-1@example.test',
  recipients: [{ finalRecipient: 'forwarded@example.test', originalRecipient: 'bob@example.test', addressType: 'rfc822',
    action: 'delivered', status: '2.0.0', diagnostic: null, remoteMta: null, retryUntil: null,
    lastAttempt: 'Tue, 10 Nov 2026 12:30:00 +0000' }],
};

it('keeps read and delivery events separate and matches an original delivery recipient through forwarding', () => {
  const received = '2026-11-10T13:00:00Z';
  expect(correlateReports(target, received, receipt, report)).toEqual([
    { address: 'alice@example.test', kind: 'read', status: 'displayed', at: Date.parse(received) },
    { address: 'bob@example.test', kind: 'delivery', status: 'delivered', at: Date.parse('2026-11-10T12:30:00Z'), code: '2.0.0' },
  ]);
});

it('does not attach reports for a different Message-ID or unrelated recipient', () => {
  const date = '2026-11-10T13:00:00Z';
  expect(correlateReports(target, date, { ...receipt, finalRecipient: 'other@example.test' },
    { ...report, originalMessageId: 'sent-2@example.test' })).toEqual([]);
  expect(correlateReports(target, date, { ...receipt, originalMessageId: null },
    { ...report, originalMessageId: null })).toEqual([]);
});
