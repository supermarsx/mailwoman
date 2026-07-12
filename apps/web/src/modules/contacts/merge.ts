// Client-side duplicate-merge (plan §3 e7 "merge-duplicates UI", risk #9). Merge
// is *non-destructive*: it produces a NEW merged card projected from a chosen
// primary plus the other duplicates (union of contact points, deduped), and the
// sources become tombstones — never edited in place, so a merge is reversible.
//
// This pure projection powers the merge *preview* the user reviews before
// committing; the slice then persists it via `ContactCard/merge` (engine, e10)
// and reflects the tombstones locally. Kept pure + exported for unit tests.

import type {
  ContactCard,
  ContactEmail,
  ContactName,
  ContactValue,
} from '../../api/pim-types.ts';

/** Case-insensitive dedupe key for a contexted value (email/phone/service). */
function contactPointKey(value: string): string {
  return value.trim().toLowerCase();
}

/** Union emails by address (case-insensitive), keeping the strongest `pref`. */
function mergeEmails(cards: ContactCard[]): ContactEmail[] {
  const byAddr = new Map<string, ContactEmail>();
  for (const card of cards) {
    for (const e of card.emails) {
      const key = contactPointKey(e.value);
      if (key.length === 0) continue;
      const prev = byAddr.get(key);
      if (prev === undefined) {
        byAddr.set(key, { ...e });
      } else if ((e.pref || 0) > (prev.pref || 0)) {
        byAddr.set(key, { ...e, context: prev.context || e.context });
      }
    }
  }
  return [...byAddr.values()];
}

/** Union contexted values (phones / online services) by value, first wins. */
function mergeValues(lists: ContactValue[][]): ContactValue[] {
  const seen = new Map<string, ContactValue>();
  for (const list of lists) {
    for (const v of list) {
      const key = contactPointKey(v.value);
      if (key.length === 0 || seen.has(key)) continue;
      seen.set(key, { ...v });
    }
  }
  return [...seen.values()];
}

/** Union a list of plain strings, trimming + case-insensitive dedupe, order-preserving. */
function mergeStrings(lists: string[][]): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const list of lists) {
    for (const s of list) {
      const t = s.trim();
      const key = t.toLowerCase();
      if (t.length === 0 || seen.has(key)) continue;
      seen.add(key);
      out.push(t);
    }
  }
  return out;
}

/** Prefer the primary's field; fall back to the first non-empty among the rest. */
function preferName(cards: ContactCard[]): ContactName {
  const pick = (get: (n: ContactName) => string): string => {
    for (const c of cards) {
      const v = get(c.name).trim();
      if (v.length > 0) return v;
    }
    return '';
  };
  return {
    full: pick((n) => n.full),
    given: pick((n) => n.given),
    surname: pick((n) => n.surname),
    prefix: pick((n) => n.prefix),
    suffix: pick((n) => n.suffix),
  };
}

/**
 * Project a merged card from `primary` + `others`. The primary supplies identity
 * (id / addressBook / uid / kind / favorite / photo / keys, with empty fields
 * back-filled from the duplicates); contact points and list-valued fields are
 * unioned and deduped. The result carries `primary.id` — the survivor.
 */
export function mergeCards(primary: ContactCard, others: ContactCard[]): ContactCard {
  const all = [primary, ...others];
  const firstNonEmpty = (get: (c: ContactCard) => string | null): string | null => {
    for (const c of all) {
      const v = get(c);
      if (v !== null && v.length > 0) return v;
    }
    return null;
  };
  const notes = mergeStrings(all.map((c) => (c.notes.length > 0 ? [c.notes] : []))).join('\n');
  return {
    id: primary.id,
    addressBookId: primary.addressBookId,
    uid: primary.uid,
    kind: primary.kind,
    name: preferName(all),
    nicknames: mergeStrings(all.map((c) => c.nicknames)),
    organizations: mergeStrings(all.map((c) => c.organizations)),
    titles: mergeStrings(all.map((c) => c.titles)),
    emails: mergeEmails(all),
    phones: mergeValues(all.map((c) => c.phones)),
    onlineServices: mergeValues(all.map((c) => c.onlineServices)),
    addresses: all.flatMap((c) => c.addresses),
    anniversaries: all.flatMap((c) => c.anniversaries).filter(
      (a, i, arr) => arr.findIndex((b) => b.kind === a.kind && b.date === a.date) === i,
    ),
    notes,
    photoBlobId: primary.photoBlobId ?? firstNonEmpty((c) => c.photoBlobId),
    // A merged card is a favorite if any source was.
    isFavorite: all.some((c) => c.isFavorite),
    groupIds: mergeStrings(all.map((c) => c.groupIds)),
    pgpKey: primary.pgpKey ?? firstNonEmpty((c) => c.pgpKey),
    smimeCert: primary.smimeCert ?? firstNonEmpty((c) => c.smimeCert),
    etag: primary.etag,
  };
}

/**
 * Heuristic duplicate detection for the "find duplicates" affordance: group
 * cards that share a normalized email or an identical full name. Returns each
 * cluster of size >= 2 (the merge candidates). Order-stable by first appearance.
 */
export function findDuplicateClusters(cards: readonly ContactCard[]): ContactCard[][] {
  const parent = new Map<string, string>();
  const find = (x: string): string => {
    let r = x;
    while (parent.get(r) !== undefined && parent.get(r) !== r) r = parent.get(r)!;
    return r;
  };
  const union = (a: string, b: string): void => {
    parent.set(find(a), find(b));
  };
  for (const c of cards) parent.set(c.id, c.id);

  const byKey = new Map<string, string>();
  const link = (key: string, id: string): void => {
    const prev = byKey.get(key);
    if (prev !== undefined) union(prev, id);
    else byKey.set(key, id);
  };
  for (const c of cards) {
    const name = c.name.full.trim().toLowerCase();
    if (name.length > 0) link(`name:${name}`, c.id);
    for (const e of c.emails) {
      const addr = contactPointKey(e.value);
      if (addr.length > 0) link(`email:${addr}`, c.id);
    }
  }

  const clusters = new Map<string, ContactCard[]>();
  for (const c of cards) {
    const root = find(c.id);
    (clusters.get(root) ?? clusters.set(root, []).get(root)!).push(c);
  }
  return [...clusters.values()].filter((g) => g.length >= 2);
}
