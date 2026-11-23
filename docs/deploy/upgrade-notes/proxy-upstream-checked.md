# Upgrade note: proxy-mode upstreams are checked (26.20)

Applies to proxy mode (`MW_MODE=proxy`, the default).

## What changes

Before 26.20 the server connected to whatever JMAP server a sign-in named, and relayed API, download and upload requests to whatever URLs that server's session document listed. From 26.20:

- An upstream origin named in `MW_JMAP_UPSTREAMS` or `MW_UPSTREAM_DEFAULT` may be on any address. When `MW_JMAP_UPSTREAMS` is set, other origins are refused; when it is unset, other origins are allowed only on public addresses (loopback, private, link-local, cloud-metadata, CGNAT and unique-local addresses are refused).
- The session document's `apiUrl`, `downloadUrl` and `uploadUrl` must be on the upstream's origin. Otherwise sign-in is refused and the server logs `proxy login refused by upstream origin policy`.
- The initial session request follows a redirect only to the same origin. Redirects on API, download and upload requests are not followed.
- Each request is pinned to the address checked for it and ignores `HTTP_PROXY`.

## What stops working

A JMAP server on a private or loopback address (another container such as `http://stalwart:8080`, a Kubernetes service, `localhost`, a LAN address) that is neither in `MW_JMAP_UPSTREAMS` nor `MW_UPSTREAM_DEFAULT`. Sign-in fails with "Invalid credentials", and the server logs `proxy login refused by upstream policy`.

## What to do

- Name the server: `MW_UPSTREAM_DEFAULT=http://stalwart:8080`, or add it to `MW_JMAP_UPSTREAMS`, with the same scheme, host and port the sign-in uses.
- For a provider whose `/.well-known/jmap` redirects to another host, set `MW_UPSTREAM_DEFAULT` to the redirect's target.
- A JMAP server on a public address needs no change.
