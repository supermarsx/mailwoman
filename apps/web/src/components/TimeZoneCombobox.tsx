// Searchable time-zone picker: IANA ids grouped by region, each row with its UTC offset
// and local time right now (so DST is already applied), the browser zone pinned on top.
// Stored values stay IANA ids; `browserValue` is what choosing the pinned row stores.

import { createMemo, type JSX } from 'solid-js';
import { t } from '../i18n';
import { formatUtcOffset, localTimeZone, pickerTimeZones, zoneClock, zoneOffsetMinutes } from '../i18n/timezones.ts';
import { Combobox, type ComboboxOption } from './Combobox.tsx';

/** Option rows for the picker, computed for the instant `now`. */
export function timeZoneOptions(now: Date, zones: readonly string[], pinned: readonly ComboboxOption[] = []): ComboboxOption[] {
  const rows = zones.map((zone): ComboboxOption => {
    const minutes = zoneOffsetMinutes(zone, now);
    const offset = minutes === null ? '' : formatUtcOffset(minutes);
    const short = offset.replace('UTC', '').replace(/^([+-])0/, '$1').replace(/:00$/, '');
    const slash = zone.indexOf('/');
    const region = slash < 0 ? t('timezone-group-other') : zone.slice(0, slash).replaceAll('_', ' ');
    return {
      value: zone,
      text: zone,
      label: slash < 0 ? zone : zone.slice(slash + 1).replaceAll('_', ' '),
      detail: [offset, zoneClock(zone, now)].filter(Boolean).join(' · '),
      // "GMT+1", "UTC+01:00" and "+01:00" all find the zone; the id keeps "Europe/Lisbon" searchable.
      keywords: minutes === null ? zone : `${zone} ${offset} ${offset.replace('UTC', 'GMT')} GMT${short} UTC${short}`,
      group: region,
    };
  });
  return [...pinned, ...rows];
}

export function TimeZoneCombobox(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  /** Stored by the pinned "browser time zone" row: the zone id, or `local`. */
  browserValue?: string | undefined;
  /** Extra pinned rows, such as "No comparison time zone". */
  extra?: readonly ComboboxOption[] | undefined;
  freeText?: boolean | undefined;
  disabled?: boolean | undefined;
  describedBy?: string | undefined;
  id?: string | undefined;
}): JSX.Element {
  const options = createMemo(() => {
    const browser = localTimeZone();
    const pinned: ComboboxOption[] = [
      ...(props.browserValue === undefined ? [] : [{ value: props.browserValue, label: t('timezone-use-browser', { zone: browser }), keywords: `${browser} browser device local` }]),
      ...(props.extra ?? []),
    ];
    return timeZoneOptions(new Date(), pickerTimeZones(), pinned);
  });
  return <Combobox label={props.label} value={props.value} options={options()} onChange={props.onChange} freeText={props.freeText} disabled={props.disabled} describedBy={props.describedBy} id={props.id} />;
}
