// Recipient autocomplete over contacts (plan §3 e7, §2.2 `ContactCard/autocomplete`).
// This is the reusable seam e10 wires into Compose's recipient field — e7 owns
// the ranking + the Solid hook, e10 supplies the reactive card source and the
// insertion callback (e7 does NOT edit Compose).
//
// Ranking is client-side over the loaded cards (works offline, no round-trip);
// the engine's `ContactCard/autocomplete` (api.ts `contactAutocomplete`) is the
// server-side equivalent e10 can swap the source for over the full DB. Both
// yield the same `ContactSuggestion` shape so the Compose call site is stable.

import { createMemo, createSignal, type Accessor } from 'solid-js';
import type { ContactCard } from '../../api/pim-types.ts';
import type { Id } from '../../api/jmap-types.ts';

/** One ranked completion — a single (card, email) pair with a display label. */
export interface ContactSuggestion {
  cardId: Id;
  name: string;
  email: string;
  /** `"Name <email>"`, or the bare email when the card is unnamed. */
  display: string;
  isFavorite: boolean;
  /** The match score (higher = better); exposed for tests / tie inspection. */
  score: number;
}

const CONTEXT_ORDER: Record<string, number> = { work: 3, home: 2, personal: 2 };

/** Format the RFC 5322-ish display label a composer would insert. */
export function suggestionDisplay(name: string, email: string): string {
  const trimmed = name.trim();
  return trimmed.length > 0 ? `${trimmed} <${email}>` : email;
}

/**
 * Score a (card, email) pair against a lowercased query. Higher is better; a
 * score of 0 means "no match" (dropped). Scoring, strongest first:
 *   - exact email / exact full-name match
 *   - name or email *starts with* the query (word-boundary aware for names)
 *   - substring match anywhere in name / email / nickname / org
 * with small boosts for favorites, the email's `pref`, and its context.
 */
function scorePair(card: ContactCard, email: string, pref: number, context: string, q: string): number {
  const name = card.name.full.toLowerCase();
  const addr = email.toLowerCase();
  const given = card.name.given.toLowerCase();
  const surname = card.name.surname.toLowerCase();
  const nicks = card.nicknames.map((n) => n.toLowerCase());
  const orgs = card.organizations.map((o) => o.toLowerCase());

  let score = 0;
  if (addr === q || name === q) score = 1000;
  else if (addr.startsWith(q)) score = 800;
  else if (name.startsWith(q) || given.startsWith(q) || surname.startsWith(q)) score = 700;
  else if (nicks.some((n) => n.startsWith(q))) score = 650;
  else if (name.includes(q) || addr.includes(q)) score = 400;
  else if (nicks.some((n) => n.includes(q)) || orgs.some((o) => o.includes(q))) score = 300;
  else return 0;

  if (card.isFavorite) score += 50;
  score += Math.min(pref, 9) * 2;
  score += CONTEXT_ORDER[context] ?? 0;
  return score;
}

/**
 * Rank the cards' emails against `prefix`, best first, capped at `limit`. Each
 * card contributes at most one suggestion (its best-scoring email). An empty or
 * whitespace prefix yields no suggestions (autocomplete opens only on input).
 */
export function rankSuggestions(
  cards: readonly ContactCard[],
  prefix: string,
  limit = 8,
): ContactSuggestion[] {
  const q = prefix.trim().toLowerCase();
  if (q.length === 0) return [];

  const best: ContactSuggestion[] = [];
  for (const card of cards) {
    let top: ContactSuggestion | null = null;
    for (const e of card.emails) {
      if (e.value.length === 0) continue;
      const score = scorePair(card, e.value, e.pref, e.context, q);
      if (score === 0) continue;
      if (top === null || score > top.score) {
        top = {
          cardId: card.id,
          name: card.name.full,
          email: e.value,
          display: suggestionDisplay(card.name.full, e.value),
          isFavorite: card.isFavorite,
          score,
        };
      }
    }
    if (top !== null) best.push(top);
  }

  best.sort((a, b) => b.score - a.score || a.display.localeCompare(b.display));
  return best.slice(0, limit);
}

/** The reactive autocomplete controller returned by {@link createContactAutocomplete}. */
export interface ContactAutocomplete {
  /** The current query text. */
  query: Accessor<string>;
  setQuery(q: string): void;
  /** Ranked suggestions for the current query (empty when the query is blank). */
  suggestions: Accessor<ContactSuggestion[]>;
  /** Clear the query + suggestions (e.g. after a pick / on blur). */
  reset(): void;
}

/**
 * Create a recipient-autocomplete controller over a reactive card source. e10
 * calls this from Compose with `() => app.contacts()` as the source and reads
 * `suggestions()` to render the dropdown, calling `setQuery` from the input and
 * `reset` after a pick. Pure ranking lives in {@link rankSuggestions}.
 */
export function createContactAutocomplete(
  source: () => readonly ContactCard[],
  opts: { limit?: number } = {},
): ContactAutocomplete {
  const limit = opts.limit ?? 8;
  const [query, setQuery] = createSignal('');
  const suggestions = createMemo(() => rankSuggestions(source(), query(), limit));
  return {
    query,
    setQuery,
    suggestions,
    reset: () => setQuery(''),
  };
}
