# Routing outbound requests through a proxy

Mailwoman makes HTTP requests of its own: remote images in messages, calendar
subscriptions, key lookups, webhooks and more. On the **Outbound connections** page
of the admin console you can send some or all of these requests through an HTTP
`CONNECT` or SOCKS5 proxy that you run. This page explains how to add a route, put it
into use and test it. How the address policy works with a proxy, and its limits, is
in [Outbound connections (egress)](../security/egress.md).

![The Outbound proxy routes tab of Outbound connections with two routes: office-socks, in use, and office-connect, not in use.](/screenshots/deploy/egress-routes.webp){.mw-shot}

## Before you start

- You need an administrator account that can change outbound settings. With
  read-only access the page shows the routes but the controls are disabled.
- Saving, deleting, starting or stopping a route may ask you to confirm your
  identity again.
- `docker-compose.proxy.yml` in the repository is not a forward proxy. It runs
  reverse proxies in front of Mailwoman for testing. To try this feature, run a
  forward proxy such as Squid or a SOCKS5 server.

## Add a route

1. In the admin console, open **Outbound connections** (under **Configuration**) and
   stay on the **Outbound proxy routes** tab.
2. Click **Add a route**.
3. Fill in the form:
   - **Id**: your name for the route. It cannot be changed after the route is saved.
   - **Scheme**: **HTTP CONNECT** or **SOCKS5**.
   - **Host** and **Port**: the proxy's address.
   - **Username** and **Password**: only if the proxy requires them. A password
     needs a username.
   - **Route scope**: `global`, or `domain:example.org` to use the route only for
     accounts of that configured domain.
   - **Traffic carried by this route**: tick at least one kind of request. Kinds you
     leave unticked are not sent through this route.
   - **Allow plaintext http origins through this route**: off by default. When it is
     off, only `https` destinations are fetched through the route.
4. Click **Add**.

![The Add a route form filled in for a SOCKS5 proxy at proxy.example.net, port 1080, with the route scope global and two kinds of traffic ticked.](/screenshots/deploy/egress-add-route.webp){.mw-shot}

A saved route carries no traffic yet. The page says **No route is in use. Outbound
fetches go direct.** until you put a route into use.

## Put a route into use

1. In the routes table, click **Use this route** on the row.
2. The row's **State** changes to **In use**, and the page names the route in use.

Only one route is in use at a time; choosing another one takes the first out of use.
The change applies on the server that handled it at once, and on other server
processes within 15 seconds.

To stop using a proxy, click **Stop using a proxy**. Outbound requests then go
direct.

![The routes table below the notice naming the route in use, with marker 1 on Stop using a proxy and marker 2 on the Test button of the route in use.](/screenshots/deploy/egress-in-use.webp){.mw-crop}

1. **Stop using a proxy** takes the route out of use.
2. **Test** appears only on the route in use.
{.mw-callouts}

## Test the route in use

1. On the row that is **In use**, click **Test**.
2. Read the result under the row. It names the outcome, where the attempt stopped,
   the route, whether the request **Traversed the proxy**, and the URL fetched.

The test fetches one fixed URL through the route: `https://example.com/` unless
`MW_EGRESS_PROBE_URL` is set on the server. It changes nothing.

| Outcome | Meaning |
|---|---|
| **Connected** | The URL was fetched through the proxy. Any HTTP answer from the URL counts. |
| **Authentication rejected** | The proxy refused the username or password. |
| **Refused by policy** | Mailwoman's address policy refused the URL, the URL is `http` and the route does not allow plaintext, or the proxy refused to open the tunnel. |
| **Name did not resolve** | The proxy's host name did not resolve. |
| **Unreachable** | No TCP connection to the proxy, or the exchange failed or timed out. |
| **Origin TLS failed** | The tunnel opened, but the URL's certificate did not verify. |
| **Route is not valid** | The route's settings cannot be used. |

Where the attempt stopped is shown as **stopped at name resolution**, **stopped while
connecting to the proxy**, **stopped while opening the tunnel** or **stopped at the
origin**. If the page shows **The test could not be run, so nothing is known about
this route.**, the test itself failed to run (for example the route was deleted); it
says nothing about the route.

## Edit or delete a route

- Click **Edit** on the row, change the fields and click **Save**. Leave **Password**
  empty to keep the stored password; type a new one only to replace it.
- Click **Delete** on the row to remove the route.

The server never sends a stored password back to the browser. The **Password**
column shows only **Set** or **None**.

## What happens when the proxy is down

A request whose kind is assigned to the route in use fails when the proxy cannot be
reached. It does not fall back to a direct connection. With the default kinds,
remote images stop loading and calendar subscriptions stop refreshing until the proxy is back or you click **Stop
using a proxy**. Use **Test** to tell whether the proxy is down or Mailwoman refused
the destination.

## What is recorded

Every save, delete, start, stop and test writes an audit entry naming the route. A
save records the route's `scheme://host:port`, whether it has a password, and the
username; a test records the outcome and stage. No entry contains the password.

## Internal destination permissions

The **Internal destination permissions** tab on the same page is a separate list. It
lets a few kinds of request (outgoing webhooks, security event collectors, Nextcloud)
reach a named private service that the address policy would otherwise refuse. See
[Outbound connections (egress)](../security/egress.md).

## Options

| Field | What it does | Default |
|---|---|---|
| **Id** | Name of the route; fixed after saving | none |
| **Scheme** | **HTTP CONNECT** or **SOCKS5** | **HTTP CONNECT** |
| **Host**, **Port** | Address of the proxy | none |
| **Username**, **Password** | Credentials sent to the proxy | none |
| **Route scope** | `global`, or `domain:<configured domain>` | `global` |
| **Traffic carried by this route** | Which kinds of request use the route | **Remote message images and files**, **Calendar subscription feeds** |
| **Allow plaintext http origins through this route** | Allow `http` destinations through the route | off |

| Variable | What it does | Default |
|---|---|---|
| `MW_EGRESS_PROBE_URL` | URL fetched by **Test** | `https://example.com/` |

## Related

- [Outbound connections (egress)](../security/egress.md)
- [Running behind a reverse proxy](reverse-proxy.md)
