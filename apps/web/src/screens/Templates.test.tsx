import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { afterEach, expect, it, vi } from 'vitest';
import { clearSettings, loadSettings } from '../api/settings.ts';
import Templates from './Templates.tsx';

afterEach(clearSettings);

it('starts an editable draft from a shared message template with literal body text', async () => {
  const setting = (value: unknown) => ({ value, source: 'global', locked: false });
  await loadSettings({ fetcher: async () => new Response(JSON.stringify({ settings: {
    'mail.templates.enabled': setting(true),
    'mail.templates.allowPersonal': setting(false),
    'mail.templates.shared': setting([{ id: 'welcome', title: 'Welcome email', subject: 'Welcome!',
      body: '<b>Literal text</b>\nSecond line', to: 'person@example.test', bcc: 'copy@example.test' }]),
  } })) });
  const onCompose = vi.fn();
  render(() => <Templates onCompose={onCompose} />);
  fireEvent.click(await screen.findByRole('button', { name: 'Welcome email' }));
  fireEvent.click(screen.getByRole('button', { name: 'Compose from template' }));
  await waitFor(() => expect(onCompose).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({
    mode: 'new', subject: 'Welcome!', to: 'person@example.test', bcc: 'copy@example.test',
    bodyText: '<b>Literal text</b>\nSecond line', bodyHtml: '<p>&lt;b&gt;Literal text&lt;/b&gt;<br>Second line</p>',
  })));
});
