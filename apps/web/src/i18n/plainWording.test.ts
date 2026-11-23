import { describe, expect, it } from 'vitest';
import contacts from '../../locales/en/contacts.ftl?raw';
import keys from '../../locales/en/keys.ftl?raw';
import properties from '../../locales/en/properties.ftl?raw';
import textViewer from '../../locales/en/text-viewer.ftl?raw';

// User-visible text states what the app does now; it carries no roadmap wording.
const PLAIN: Array<[string, string]> = [
  [contacts, 'contacts-key-on-file'],
  [keys, 'keys-smime-not-in-app'],
  [keys, 'keys-generate-smime-note'],
  [properties, 'props-more-options'],
  [properties, 'props-more-options-help'],
  [textViewer, 'text-viewer-linked-unavailable'],
];

const messageOf = (source: string, id: string): string => {
  const line = source.split(/\r?\n/).find((l) => l.startsWith(`${id} = `));
  if (line === undefined) throw new Error(`missing ${id}`);
  return line;
};

describe('plain wording in user-visible catalog text', () => {
  it.each(PLAIN)('message has no roadmap phrasing', (source, id) => {
    expect(messageOf(source, id)).not.toMatch(/\byet\b|\blands\b|\buntil\b[^.]*\b(lands|ships)\b|\bcoming\b|\bplanned\b/i);
  });
});
