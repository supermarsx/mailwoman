# Upgrade note: OAuth client redirect addresses (26.20)

Applies to deployments where apps register themselves with Mailwoman's OAuth server (**OAuth clients** in the admin console), or where an administrator registered OAuth clients.

## What changes

**Before:** a self-registered app's redirect address was checked only for its host, with a simple parser, so any scheme was accepted (including `http://` to a public host and app-specific schemes), and an address such as `https://evil.example\@app.example.org/cb` could pass the host check while a browser sent the authorization code to another host. Addresses were not checked again when an app asked for sign-in.

**After:**

- Redirect addresses are read as a browser reads them. A backslash, control characters, spaces, a user name or password, or a fragment are always refused.
- A self-registered app may use only `https`, or `http` to a loopback IP address (`127.0.0.1`, `::1`) that is also on **Allowed redirect hosts**. `http` to other hosts, `localhost`, and app-specific schemes are refused at registration.
- When an app asks for sign-in, its registered address is checked again: a self-registered app stored earlier with an address that no longer passes, and an administrator-registered client whose address has a user name, a backslash or a fragment, get no authorization code. Administrator-registered clients keep app-specific schemes.
- An authorization code or refresh token is redeemed exactly once, also when two requests with it arrive at the same time; the second gets `invalid_grant`.

## What may stop working

- Self-registered apps with an `http` redirect address to a host that is not a loopback IP, or with an app-specific scheme: they cannot register, and those already registered no longer receive an authorization code. Register them again with an `https` address, or, for an app on the same computer, `http` to `127.0.0.1` or `::1` (and add that address to **Allowed redirect hosts**).
- Administrator-registered clients whose redirect address contains a user name, a backslash or a fragment: they no longer receive an authorization code. Correct the address.

See [OAuth clients](/admin/oauth-clients#allowedRedirectHostSuffixes).
