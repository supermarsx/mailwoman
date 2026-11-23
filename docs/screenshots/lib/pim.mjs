// Write a demo/pim.json-style dataset through the signed-in user's JMAP session
// (stack E: the engine's own calendar, task, note and contact collections).
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { DEMO } from './paths.mjs';
import { userJmap } from './seed.mjs';

const CORE = 'urn:ietf:params:jmap:core';
const CAL = 'urn:mailwoman:calendars';
const TASKS = 'urn:mailwoman:tasks';
const NOTES = 'urn:mailwoman:notes';
const CONTACTS = 'urn:mailwoman:contacts';

const ids = (response) => Object.fromEntries(response.map(([, args, id]) => [id, args]));

export async function seedPim(client, stack, file) {
  if (stack.kind !== 'E') throw new Error(`pim seed ${file} needs stack E (engine mode)`);
  const data = JSON.parse(readFileSync(join(DEMO, file), 'utf8'));
  const people = JSON.parse(readFileSync(join(DEMO, 'people.json'), 'utf8')).people;
  const session = (await client.must('GET', '/jmap/session')).json;
  const accountId = session.primaryAccounts?.[CAL] ?? Object.keys(session.accounts)[0];
  const tz = data.timeZone ?? 'UTC';

  const lists = ids(
    await userJmap(client, [CORE, CAL, TASKS, CONTACTS], [
      ['Calendar/get', { accountId, ids: null }, 'cal'],
      ['AddressBook/get', { accountId, ids: null }, 'books'],
    ]),
  );
  const calendars = lists.cal.list ?? [];
  const calendar = calendars.find((c) => c.role === 'default') ?? calendars[0];
  const book = (lists.books.list ?? []).find((b) => b.isDefault) ?? lists.books.list?.[0];

  if (data.events?.length) {
    if (!calendar) throw new Error('no calendar to seed events into');
    const create = {};
    data.events.forEach((e, i) => {
      create[`e${i}`] = {
        calendarId: calendar.id,
        title: e.title,
        description: e.description ?? '',
        locations: e.location ? [{ name: e.location }] : [],
        start: e.start,
        timeZone: e.allDay ? null : tz,
        duration: e.duration ?? 'PT1H',
        showWithoutTime: Boolean(e.allDay),
        status: 'confirmed',
        freeBusyStatus: 'busy',
        participants: Object.fromEntries(
          (e.participants ?? []).map((key) => [people[key].email, { name: people[key].name, email: people[key].email, role: 'attendee', participationStatus: 'accepted', expectReply: false }]),
        ),
      };
    });
    await userJmap(client, [CORE, CAL], [['CalendarEvent/set', { accountId, create }, 'ev']]);
  }

  if (data.tasks?.length) {
    const taskList = calendars.find((c) => c.supportsTasks || /task/i.test(c.name)) ?? calendar;
    const create = {};
    data.tasks.forEach((t, i) => {
      create[`t${i}`] = {
        ...(taskList ? { listId: taskList.id } : {}),
        title: t.title,
        description: t.description ?? '',
        due: t.due ?? null,
        timeZone: tz,
        priority: t.priority ?? 0,
        status: t.status ?? 'needs-action',
        percentComplete: t.percentComplete ?? 0,
      };
    });
    await userJmap(client, [CORE, TASKS, CAL], [['Task/set', { accountId, create }, 'ts']]);
  }

  if (data.notes?.length) {
    const create = {};
    data.notes.forEach((n, i) => {
      create[`n${i}`] = {
        title: n.title,
        bodyHtml: n.body ?? '',
        bodyText: (n.body ?? '').replace(/<[^>]+>/g, ' ').replace(/\s+/g, ' ').trim(),
        pinned: Boolean(n.pinned),
        tags: n.tags ?? [],
        color: n.color ?? '',
      };
    });
    await userJmap(client, [CORE, NOTES], [['Note/set', { accountId, create }, 'ns']]);
  }

  if (data.contacts?.length) {
    if (!book) throw new Error('no address book to seed contacts into');
    const create = {};
    data.contacts.forEach((c, i) => {
      const p = c.person ? people[c.person] : null;
      const [given, ...rest] = (p?.name ?? '').split(' ');
      create[`c${i}`] = {
        addressBookId: book.id,
        kind: c.org ? 'org' : 'individual',
        name: p ? { full: p.name, given, surname: rest.join(' '), prefix: '', suffix: '' } : { full: c.org, given: '', surname: '', prefix: '', suffix: '' },
        organizations: p?.org ? [p.org] : c.org ? [c.org] : [],
        titles: c.title ? [c.title] : [],
        emails: [{ context: 'work', value: p?.email ?? c.email, pref: 1 }],
        phones: c.phone ? [{ context: 'work', value: c.phone }] : [],
        isFavorite: Boolean(c.favorite),
      };
    });
    await userJmap(client, [CORE, CONTACTS], [['ContactCard/set', { accountId, create }, 'cs']]);
  }
}
