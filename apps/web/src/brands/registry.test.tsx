import {describe, expect, it} from 'vitest';
import {render, screen, waitFor} from '@solidjs/testing-library';
import {BRANDS, brandFor, brandForHost, loadBrandPath} from './registry.ts';
import {PATHS} from './paths.generated.ts';
import {BrandLogo, monogramHue, monogramLetter} from './BrandLogo.tsx';

describe('brand registry', () => {
  it('resolves every id, name and alias to its entry, with a path for each', async () => {
    const ids = new Set<string>();
    for (const entry of BRANDS) {
      expect(ids.has(entry.id), entry.id).toBe(false);
      ids.add(entry.id);
      expect(brandFor(entry.id)?.id).toBe(entry.id);
      expect(brandFor(entry.name)?.id).toBe(entry.id);
      for (const alias of entry.aliases) expect(brandFor(alias), alias).toBeDefined();
      expect(typeof PATHS[entry.id], entry.id).toBe('string');
      expect(await loadBrandPath(entry.id)).toBe(PATHS[entry.id]);
    }
    expect(Object.keys(PATHS).sort()).toEqual([...ids].sort());
  });

  it('matches domains and hosts by suffix, longest first', () => {
    expect(brandFor('gmail.com')?.id).toBe('gmail');
    expect(brandFor('Google Mail')?.id).toBe('gmail');
    expect(brandFor('outlook.com')?.id).toBe('outlook');
    expect(brandForHost('imap.gmail.com')?.id).toBe('gmail');
    expect(brandForHost('https://s3.eu-west-1.amazonaws.com/bucket')?.id).toBe('aws');
    expect(brandForHost('someone@icloud.com')?.id).toBe('icloud');
    expect(brandForHost('mail.example.org:993')).toBeUndefined();
    expect(brandForHost('')).toBeUndefined();
    expect(brandForHost('com')).toBeUndefined();
    expect(brandForHost('notgmail.com')).toBeUndefined();
  });

  it('holds sanitised path data only', () => {
    for (const [id, d] of Object.entries(PATHS)) {
      expect(d, id).toMatch(/^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/);
      expect(d.length, id).toBeLessThan(6000);
    }
    const text = JSON.stringify(BRANDS) + JSON.stringify(PATHS);
    expect(text).not.toMatch(/https?:|\/\/|<script|<\/|javascript:|\bon[a-z]+\s*=|url\(|data:/i);
    for (const entry of BRANDS) if (entry.color) expect(entry.color).toMatch(/^#[0-9A-F]{6}$/);
  });
});

describe('BrandLogo', () => {
  it('draws the bundled mark, labelled when it stands alone', async () => {
    const {container} = render(() => <BrandLogo brand="gmail.com" size={24} />);
    expect(screen.getByRole('img', {name: 'Gmail'})).toBeTruthy();
    await waitFor(() => expect(container.querySelector('path')?.getAttribute('d')).toBe(PATHS.gmail));
    expect(container.querySelector('svg')?.getAttribute('data-size')).toBe('24');
  });

  it('is hidden from assistive technology when decorative, and mono uses currentColor', async () => {
    const {container} = render(() => <BrandLogo brand="github" decorative mono />);
    const svg = container.querySelector('svg')!;
    expect(svg.getAttribute('aria-hidden')).toBe('true');
    expect(svg.getAttribute('role')).toBeNull();
    await waitFor(() => expect(container.querySelector('path')?.getAttribute('fill')).toBe('currentColor'));
  });

  it('falls back to a deterministic monogram for unknown brands', () => {
    const {container} = render(() => <BrandLogo brand="Acme Mail" />);
    expect(container.querySelector('svg')?.getAttribute('data-brand')).toBe('monogram');
    expect(container.querySelector('text')?.textContent).toBe('A');
    expect(container.querySelector('path')).toBeNull();
    expect(monogramHue('Acme Mail')).toBe(monogramHue('acme mail'));
    expect(monogramLetter('  ünder')).toBe('Ü');
    expect(monogramLetter('')).toBe('?');
  });
});

describe('brandInText', () => {
  it('finds a brand inside an operator-chosen label, and not on short words', async () => {
    const {brandInText} = await import('./registry.ts');
    expect(brandInText('Acme Okta SSO')?.id).toBe('okta');
    expect(brandInText('Sign in with Microsoft Entra')?.id).toBe('microsoft');
    expect(brandInText('Google Workspace')?.id).toBe('google');
    expect(brandInText('Mail box live')).toBeUndefined();
    expect(brandInText('Company login')).toBeUndefined();
  });
});
