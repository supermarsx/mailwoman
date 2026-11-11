import {SETTINGS_TRANSFER_CARD} from '../SettingsTransfer.tsx';
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
import { PROPERTIES_CARD } from '../PropertiesPreferences.tsx';
import { CACHE_CARDS } from '../CachePreferences.tsx';
import { AVATAR_CARD } from '../ContactAppearance.tsx';
import { SAVE_CONTACTS_CARD } from '../SaveContactsSettings.tsx';
import { UNDO_CARD } from '../UndoRedoPreferences.tsx';
import { APPEARANCE_CARDS } from './appearanceSpecs.ts';
import { FOLDER_PREFERENCE_CARDS } from './foldersSpecs.ts';
import { NOTIFICATION_CARDS } from '../Notifications.tsx';
import { LIST_CARDS, SECURITY_SECTION_CARDS, DEVICE_PREFERENCE_CARDS, SHORTCUT_CARDS, PRIVACY_CARDS, COMPOSE_EXTRA_CARDS, MISC_SECTION_CARDS } from './listSpecs.ts';
import { t } from '../../../i18n';
import { cardMatches, tokens } from './searchModel.ts';
import { PIM_RETENTION_CARDS } from '../PimRetention.tsx';
import { COMPOSER_FEATURES_CARD } from '../ComposerFeatures.tsx';
import { COMPOSE_DELIVERY_CARD } from '../ComposeDelivery.tsx';
import type { CardSpec } from './rows.tsx';
import { SEARCH_SCOPE_CARD } from '../SearchScopePreferences.tsx';
import {REGIONAL_CARD} from '../RegionalFormats.tsx';

export const SETTINGS_REGISTRY: readonly CardSpec[] = [
  REGIONAL_CARD,
  SEARCH_SCOPE_CARD, SETTINGS_TRANSFER_CARD,
  ...READER_CARD_SPECS, ...MAIL_CARD_SPECS, ...COMPOSE_CARDS, COMPOSE_DEFAULTS_CARD, DRAFT_SAVE_CARD,
  COMPOSE_DELIVERY_CARD, COMPOSER_FEATURES_CARD,
  INTERFACE_CARD, FOLDER_COUNT_CARD, PROPERTIES_CARD, ...CACHE_CARDS, AVATAR_CARD, SAVE_CONTACTS_CARD, UNDO_CARD, ...APPEARANCE_CARDS, ...FOLDER_PREFERENCE_CARDS, ...NOTIFICATION_CARDS,
  ...LIST_CARDS, ...SECURITY_SECTION_CARDS, ...DEVICE_PREFERENCE_CARDS, ...SHORTCUT_CARDS, ...PRIVACY_CARDS, ...COMPOSE_EXTRA_CARDS, ...MISC_SECTION_CARDS, ...PIM_RETENTION_CARDS,
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
