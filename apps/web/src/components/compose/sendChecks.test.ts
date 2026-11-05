import { describe, expect, it } from 'vitest';
import { externalDomains, invalidRecipients, mentionsAttachment, planSend, resolvePolicy, type CheckId, type CheckPolicy } from './sendChecks.ts';

const all = (policy: CheckPolicy): Record<CheckId, CheckPolicy> => ({ invalidRecipients: policy, missingAttachment: policy, emptySubject: policy, externalRecipients: policy });

describe('invalidRecipients', () => {
  it('lists only entries that are not mailboxes, across fields', () => {
    expect(invalidRecipients(['a@b.example, nope', 'Ann <ann@b.example>; c@', ''])).toEqual(['nope', 'c@']);
  });
  it('accepts nothing invalid for an empty field', () => {
    expect(invalidRecipients(['', '  '])).toEqual([]);
  });
});

describe('mentionsAttachment', () => {
  it.each([
    'Please see the attached report.',
    'Segue em anexo o relatório',
    'Adjunto el archivo',
    'Voir la pièce jointe',
    'Die Datei ist angehängt',
    'In allegato il file',
    'Zie de bijlage',
    'I have enclosed the form',
  ])('detects %s', text => expect(mentionsAttachment(text)).toBe(true));
  it('ignores words that merely contain a keyword', () => {
    expect(mentionsAttachment('The detachment moves at noon; anexos2 ok')).toBe(false);
  });
  it('ignores quoted lines, the quoted original and the signature', () => {
    expect(mentionsAttachment('Thanks!\n\n> see attached\n')).toBe(false);
    expect(mentionsAttachment('Thanks!\n\nOn Mon, Ann wrote:\nsee the attached file')).toBe(false);
    expect(mentionsAttachment('Thanks!\n\nEm seg, Ann escreveu:\nsegue em anexo')).toBe(false);
    expect(mentionsAttachment('Thanks!\n-- \nSent with an attachment-friendly client')).toBe(false);
    expect(mentionsAttachment('Thanks\n\nFrom: Ann\nSent: Monday\nSubject: x\n\nsee attached')).toBe(false);
  });
  it('still sees mentions above the quote', () => {
    expect(mentionsAttachment('Attached is the file.\n\nOn Mon, Ann wrote:\n> hi')).toBe(true);
  });
});

describe('policy resolution', () => {
  it('maps the older external-recipient values and defaults to warn', () => {
    expect(resolvePolicy('off')).toBe('allow');
    expect(resolvePolicy('confirm')).toBe('allow');
    expect(resolvePolicy('deny')).toBe('deny');
    expect(resolvePolicy('warn-remember')).toBe('warn-remember');
    expect(resolvePolicy(undefined)).toBe('warn');
    expect(resolvePolicy('bogus')).toBe('warn');
  });
  const issues = [
    { id: 'externalRecipients' as const, details: ['x.example'] },
    { id: 'emptySubject' as const, details: [] },
    { id: 'invalidRecipients' as const, details: ['nope'] },
  ];
  it('allow adds nothing, warn asks in check order, never blocks', () => {
    expect(planSend(issues, all('allow'))).toEqual({ denied: [], asked: [], rememberable: [] });
    const plan = planSend(issues, all('warn'));
    expect(plan.asked.map(i => i.id)).toEqual(['invalidRecipients', 'emptySubject', 'externalRecipients']);
    expect(plan.denied).toEqual([]);
    expect(plan.rememberable).toEqual([]);
  });
  it('warn-remember offers remembering unless the setting is locked', () => {
    expect(planSend(issues, all('warn-remember')).rememberable).toEqual(['invalidRecipients', 'emptySubject', 'externalRecipients']);
    expect(planSend(issues, all('warn-remember'), { emptySubject: true }).rememberable).toEqual(['invalidRecipients', 'externalRecipients']);
  });
  it('deny refuses only the denied checks', () => {
    const plan = planSend(issues, { ...all('warn'), emptySubject: 'deny' });
    expect(plan.denied.map(i => i.id)).toEqual(['emptySubject']);
    expect(plan.asked.map(i => i.id)).toEqual(['invalidRecipients', 'externalRecipients']);
  });
});

describe('externalDomains', () => {
  it('lists each outside domain once, lower-cased', () => {
    expect(externalDomains(['a@X.example', 'b@x.example', 'c@y.example'])).toEqual(['x.example', 'y.example']);
  });
});
