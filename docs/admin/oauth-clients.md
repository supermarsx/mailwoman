# Admin console: OAuth clients

The **OAuth clients** page (`#/oauth-clients`) edits the dynamic client registration policy
(RFC 7591) stored by `PUT /admin/oauth-dcr`. Registration is off unless an administrator turns it on.
The console does not list registered clients yet: the server has no admin route that lists them.

<a id="enabled"></a>
## Let apps register themselves

Opens `POST /oauth/register` to apps. Off by default. Clients added by other means are not affected.
The console asks for at least one redirect host before it saves this switch on, because an empty host
list refuses every registration.

<a id="allowedRedirectHostSuffixes"></a>
## Allowed redirect hosts

Every redirect address an app registers must be on one of these hosts or a subdomain of it:
`example.org` allows `example.org` and `app.example.org`. An empty list refuses every registration.

<a id="requireInitialAccessToken"></a>
## Require the shared registration token

Registration then needs `Authorization: Bearer <token>` with the value of `MW_DCR_INITIAL_ACCESS_TOKEN`,
set where the server starts. Without that variable every registration is refused.

<a id="defaultScope"></a>
## Granted scope

The scope every self-registered client gets: read, send and delete, over mail and/or calendar, contacts,
tasks and notes, with an optional limit of requests per minute. What a client asks for never widens it.
Members the page has no control for (account and folder selectors, IP allowlist, expiry, MCP tools) are
kept as stored.
