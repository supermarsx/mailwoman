import type { devices } from '@playwright/test';

type Device = (typeof devices)[string];

/**
 * A Chromium device whose browser process sends the same User-Agent as its pages.
 *
 * Playwright sets the device's User-Agent on the browser context, but requests
 * made by a service worker or SharedWorker (the `/sw.js` script fetch among
 * them) carry Chromium's process User-Agent, which in headless mode says
 * `HeadlessChrome`. By default the server refuses that automation claim on the
 * first anonymous request (security.protection.userAgentBlockBots, 429), so the
 * worker fails to load and the page under test is not the one a browser gets.
 * Aligning the two keeps bot protection out of these tests; it is tested on its
 * own (crates/mw-server/tests/t32_request_protection.rs).
 */
export function chromiumAs(device: Device) {
  return { ...device, launchOptions: { args: [`--user-agent=${device.userAgent}`] } };
}
