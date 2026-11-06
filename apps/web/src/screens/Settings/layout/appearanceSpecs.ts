// Search-index entries for the Appearance cards that Settings.tsx renders itself
// (importing Settings.tsx here would be circular). Row ids are the ids of the
// element that carries each row's label; settings.appearance.test.tsx renders
// the real Settings screen and fails if one is missing or duplicated.
import { LANGUAGE_CARD } from '../Language.tsx';
import { BAR_CARD } from '../BarPreferences.tsx';
import type { CardSpec } from './rows.tsx';

export const APPEARANCE_CARDS: readonly CardSpec[] = [
  { id: 'settings-appearance-theme-card', section: 'appearance', title: 'settings-appearance-theme-card', help: 'settings-appearance-theme-card-help', rows: [
    { id: 'appearance-mode', label: 'appearance-mode', help: 'appearance-mode-hint-fixed', keywords: 'dark light system schedule' },
    { id: 'settings-theme', label: 'settings-theme', help: 'appearance-gallery-hint' },
    { id: 'appearance-sync', label: 'appearance-sync' },
  ] },
  { id: 'settings-appearance-colours-card', section: 'appearance', title: 'settings-appearance-colours-card', help: 'settings-appearance-colours-card-help', rows: [
    { id: 'settings-accent', label: 'settings-accent' },
    { id: 'appearance-custom-accent', label: 'appearance-custom-accent' },
    { id: 'settings-badge-color', label: 'settings-badge-color' },
  ] },
  LANGUAGE_CARD, BAR_CARD,
];
