# Admin console: AI assistant

The **AI assistant** page (`#/ai-assistant`) edits the deployment's assistant configuration
(`GET/PUT /admin/assist`): the endpoint, the capabilities it may use and the data limits. It shows what
the running server reports (`GET /admin/assist/status`). The endpoint, capabilities and data limits are
read when the server starts, so those fields say "restart needed". The form is offered only once the
stored configuration has been read, so defaults never overwrite it.

**Stop the AI assistant now** asks first, then refuses every assistant request at once, for every user,
and is kept across restarts. **Turn the AI assistant on** reverses it (after a server that started with
it off restarts).

<a id="kind"></a>
## Endpoint type

None, OpenAI-compatible, Anthropic, or a local program. A local program is offered only from the list in
`MW_ASSIST_LOCAL_PROGRAMS`, set where the server starts; the server refuses any other program.

<a id="baseUrl"></a>
## Base URL

Required for an OpenAI-compatible endpoint, optional for Anthropic (empty uses the provider's address).

<a id="apiKey"></a>
## API key

Write-only. With a key stored the page says so and offers **Replace** or **Remove the stored key**. The
server keeps a stored key only while the endpoint type and base URL stay the same: after either changes
the page asks for a key for the new endpoint, and saving without one stores no key.

<a id="models"></a>
## Models and limits of the endpoint

OpenAI-compatible: chat, embedding and speech-to-text models. Anthropic: model, API version and the
longest reply in tokens. Empty fields use the server defaults.

<a id="program"></a>
## Local program and arguments

The program the server starts for each request (the request, including the selected message text, goes
to its standard input). Arguments are a list, one entry per argument, passed without a shell. A stored
program that is no longer listed is shown marked, and the page refuses to save it.

<a id="capabilityGrants"></a>
## Capabilities

Only the capabilities ticked here can be used: summarize, draft and rewrite, grammar, dictation,
semantic search, auto-tag, recap, assistant chat.

<a id="accounts"></a>
## Accounts

The account ids whose mail may be sent to the endpoint. With none listed, no mail content is sent.

<a id="folders"></a>
## Folders

Folder ids to limit sending to. With none listed, every folder of the listed accounts is allowed.

<a id="includeE2ee"></a>
## End-to-end-encrypted content

Off (the default) withholds end-to-end-encrypted content from every request.

<a id="includeAttachments"></a>
## Attachments

Off (the default) withholds attachments from every request.

<a id="rateLimitPerMin"></a>
## Rate limit

The settings-model key `assistant.rateLimitPerMin` (`GET/PUT /admin/settings/assistant.rateLimitPerMin`):
requests per minute across all users, 0 for no limit, at most 10000. It applies within 30 seconds.
`MW_ASSIST_RATE_LIMIT_PER_MIN` pins it. On a server without the settings model the page says the value
is set with that variable.
