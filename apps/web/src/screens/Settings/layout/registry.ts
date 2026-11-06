// Static search index for the converted Settings sections, so a search can list
// rows in sections that have not been opened (and so are not in the DOM yet).
// Each section exports its CardSpec list next to the component that renders it;
// registry.test.tsx renders those components and fails if a spec row id is not
// present exactly once, so the index cannot drift from the screens.
import { COMPOSE_CARDS } from '../ComposePreferences.tsx';
import { COMPOSE_DEFAULTS_CARD } from '../ComposeDefaults.tsx';
import { DRAFT_SAVE_CARD } from '../DraftSavePreferences.tsx';
import { READER_CARD_SPECS } from '../ReaderAppearance.tsx';
import { MAIL_CARD_SPECS } from '../MailFeaturePreferences.tsx';
import { INTERFACE_CARD } from '../InterfacePreferences.tsx';
import { FOLDER_COUNT_CARD } from '../FolderCountPreferences.tsx';
import { CACHE_CARDS } from '../CachePreferences.tsx';
import { AVATAR_CARD } from '../ContactAppearance.tsx';
import { UNDO_CARD } from '../UndoRedoPreferences.tsx';
import { APPEARANCE_CARDS } from './appearanceSpecs.ts';
import { FOLDER_PREFERENCE_CARDS } from './foldersSpecs.ts';
import { t } from '../../../i18n';
import { cardMatches, tokens } from './searchModel.ts';
import type { CardSpec } from './rows.tsx';

export const SETTINGS_REGISTRY: readonly CardSpec[] = [
  ...READER_CARD_SPECS, ...MAIL_CARD_SPECS, ...COMPOSE_CARDS, COMPOSE_DEFAULTS_CARD, DRAFT_SAVE_CARD,
  INTERFACE_CARD, FOLDER_COUNT_CARD, ...CACHE_CARDS, AVATAR_CARD, UNDO_CARD, ...APPEARANCE_CARDS, ...FOLDER_PREFERENCE_CARDS,
];

export interface RegistryHit { section: string; card: string; cardTitle: string; row: string; label: string }

/** Rows whose rendered label or hint matches every word, outside `exceptSection`. */
export function searchRegistry(query: string, exceptSection: string): RegistryHit[] {
  const words = tokens(query);
  if (!words.length) return [];
  const hits: RegistryHit[] = [];
  for (const card of SETTINGS_REGISTRY) {
    if (card.section === exceptSection) continue;
    for (const row of card.rows) {
      const text = `${t(card.title)} ${t(row.label)} ${row.help ? t(row.help) : ''} ${row.keywords ?? ''}`;
      if (cardMatches(text, words)) hits.push({ section: card.section, card: card.id, cardTitle: t(card.title), row: row.id, label: t(row.label) });
    }
  }
  return hits;
}
