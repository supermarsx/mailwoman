---
title: AI assistant in the admin console
screen: true
---

# AI assistant

The **AI assistant** page sets which AI endpoint the server sends assistant requests to, which capabilities users may use, and how much mail content may leave the server. Mailwoman includes no model and no default provider: with no endpoint the assistant is off and nothing is sent. This page describes the screen. What is sent for each feature, the order in which the controls apply and what is recorded are described in [AI assistant: data flows and controls](/assist); users' side is in [Using the AI assistant](/user/assist).

![The AI assistant page of the administration console: the running status, the endpoint, the capabilities, the data limits, the rate limit and the provider profiles.](/screenshots/admin-ops/opsc-assist-screen.webp){.mw-shot}

The line at the top reports what the running server does, for example **The AI assistant is running. Requests go to** and the host, **The AI assistant is off. Every AI assistant request is refused.**, or **The AI assistant is not running. The saved configuration takes effect when the server restarts.** The endpoint, capabilities and data limits are read when the server starts, so each of those fields says **Applies after the server restarts.** The form is offered only once the stored configuration has been read, so defaults never overwrite it. If the stored key does not open with the server's key (`MW_SERVER_KEY`), the page says so, the assistant is off, and you fill in the form including the API key and save.

## Before you start

- You need the console permission `ai_assistant.read` to see the page.
- Decide whether the endpoint is hosted by someone else. Everything the assistant sends goes to that endpoint; see [AI assistant: data flows and controls](/assist).

## Stop or resume the assistant

**Stop the AI assistant now** asks for confirmation (**Stop the AI assistant for everyone?**), then refuses every assistant request for every user at once. The configuration is kept and the stop survives restarts. **Turn the AI assistant on** reverses it; if the server started with the assistant off, turning it on takes effect when the server restarts.

## Set the endpoint

1. <a id="kind"></a> Under **Endpoint**, choose the **Endpoint type**: **None**, **OpenAI-compatible**, **Anthropic** or **Local program**. **None** turns the endpoint off without stopping the assistant.

2. <a id="baseUrl"></a> Enter the **Base URL**. It is required for an OpenAI-compatible endpoint and starts with `http://` or `https://`; for Anthropic it is optional and empty uses the provider's address.

3. <a id="apiKey"></a> Enter the **API key**. It is write-only. With a key stored the page says **A key is stored. The server does not show it again.** and offers **Replace the API key** and **Remove the stored key**. The server keeps a stored key only while the endpoint type and base URL stay the same; after either changes, the page asks for a key for the new endpoint and saving without one stores no key.

4. <a id="models"></a> Set the models. For OpenAI-compatible: **Chat model**, **Embedding model** and **Speech-to-text model**. For Anthropic: **Model**, **API version** and **Maximum reply tokens**. Empty fields show **Server default**.

![The Endpoint group of the AI assistant page: endpoint type, base URL, the stored API key, and the chat, embedding and speech-to-text models.](/screenshots/admin-ops/opsc-assist-endpoint.webp){.mw-crop}

1. **Endpoint type**.
2. **Base URL**.
3. **Remove the stored key**.
4. **Chat model**, with the embedding and speech-to-text models beside it.
{.mw-callouts}

<a id="program"></a>
### Local program

A **Local program** is offered only when `MW_ASSIST_LOCAL_PROGRAMS` lists the program's absolute path where the server starts (separate several paths with commas), and the server was restarted. The server starts the program for each request and passes the request, including the message text, on its standard input; **Arguments, one per line** are passed one per argument, without a shell. A stored program that is no longer listed shows **(not listed)** and the page refuses to save it, with **The configuration was not saved because the server does not allow this program.** from the server if it is submitted anyway. With a local program nothing leaves the server.

<a id="capabilityGrants"></a>
## Capabilities

Under **Capabilities**, tick what users may use: **Summarize**, **Draft and rewrite**, **Grammar**, **Dictation**, **Semantic search**, **Auto-tag**, **Recap** and **AI assistant chat**. Only the ticked capabilities can be used. The web app has no control that calls **Summarize** or **Recap**.

::: warning Auto-tag sends mail without being asked
While **Auto-tag** is ticked, the subject and preview of every message a user opens are sent to the endpoint each time it is opened, without the user asking. Leave it unticked if opening a message must not contact the endpoint.
:::

## Data limits

Under **Data limits**, these limits apply to every request whatever the capability. They apply to the mail content the web app labels with its account, folder and kind; see [AI assistant: data flows and controls](/assist) for what that does and does not guarantee.

-  <a id="accounts"></a>**Account ids whose mail may be sent**: with none listed, no mail content is sent to the endpoint; only prompts and audio go out.

-  <a id="folders"></a>**Folder ids to limit sending to**: with none listed, every folder of the listed accounts is allowed.

-  <a id="includeE2ee"></a>**Allow end-to-end-encrypted content to be sent**: off by default; off withholds end-to-end-encrypted content from every request.

-  <a id="includeAttachments"></a>**Allow attachments to be sent**: off by default; off withholds attachments from every request.

## Rate limit

<a id="rateLimitPerMin"></a>
The **Rate limit** group shows **Requests per minute** ([`assistant.rateLimitPerMin`](/admin/settings#assistant.ratelimitpermin)). **The server applies only `MW_ASSIST_RATE_LIMIT_PER_MIN`, read when it starts; a value saved here is not applied.**

![The Rate limit group of the AI assistant page with the text that the server applies only MW_ASSIST_RATE_LIMIT_PER_MIN and the Requests per minute field.](/screenshots/admin-ops/opsc-assist-limits.webp){.mw-crop}

The field in the image still carries the hint **Applies within 30 seconds** and **Default: 0 (refuses every request)**; neither describes what the server does, as explained here.

The limit counts outbound requests per account per minute. With the variable unset there is no limit. `0` refuses every assistant request. A chat turn counts 1; a semantic search on an empty cache counts up to 33. The field shows `0` as its default, but that is the stored setting's default, which the server does not read: with the variable unset the assistant is not limited. The daily limit `limit.ai_calls_per_day` that appears in the feature editor is not enforced either.

## Provider profiles

**Provider profiles** lets you keep several named endpoints and assign one to the whole deployment, to a domain or to one user. With no profile assigned, the endpoint above is used. Semantic search always uses the deployment endpoint.

![The Provider profiles section of the AI assistant page with no profiles yet and the Add profile button.](/screenshots/admin-ops/opsc-assist-profiles.webp){.mw-crop}

1. Choose **Add profile**, pick a **Preset**, and fill in the **Name**, **Endpoint URL**, **Model** and **API key**. **Request options** (**Maximum reply tokens**, **Temperature (0 to 2)**, **Time limit (ms)**) are optional.
2. Choose **Save profile**, then **Test profile**. The test sends a fixed prompt with no mail content and reports **Answered in** the time, or **Failed after** the time and the reason.
3. Under **Who uses which profile**, set the **Deployment default** and, in **By domain** and **By user**, the profile each uses. A user assigned a profile that no longer exists has no assistant.

An endpoint on this machine or the local network is refused until its exact host and port are listed under **Local endpoints** (**Grant**). Adding or removing one asks you to confirm that it is you (your password and second factor, or a passkey). Link-local and cloud metadata addresses can never be granted. Presets are marked **Tested against a local mock of this request format, not against the provider's live service.** or **Not tested.**

## Options

| Label | What it does | Default | Who can change it |
|---|---|---|---|
| **Requests per minute** ([`assistant.rateLimitPerMin`](/admin/settings#assistant.ratelimitpermin)) | Saved but not applied; only `MW_ASSIST_RATE_LIMIT_PER_MIN` is read | 0 stored; unlimited in effect | Administrator (no effect) |
| **AI assistant** ([`assistant.enabled`](/admin/settings#assistant.enabled)) | Lets a user turn the assistant off for themselves, below the deployment switch | On | Administrator, domain, user |
| [`assist.profile`](/admin/settings#assist.profile) | Which provider profile serves a user; empty means the deployment endpoint | Empty | Administrator (deployment, domain or user) |

## Troubleshooting

- **The AI assistant is not running. The configured program … is not listed in MW_ASSIST_LOCAL_PROGRAMS on the server, so the server did not start it.** Add the program's absolute path to `MW_ASSIST_LOCAL_PROGRAMS` and restart.
- **The AI assistant is on but not running because no usable endpoint is configured.** Choose an endpoint type other than **None** and fill in its required fields, then restart the server.
- **The stored AI assistant configuration cannot be read: its API key does not open with this server's key (MW_SERVER_KEY).** The configuration was saved under another key. Fill in the form, including the API key, and save.
- **Could not load the AI assistant configuration. Saving is disabled so the stored configuration is not overwritten.** Reload the page; the form stays disabled until the stored configuration has been read.
- Every assistant request is refused and the endpoint is set: check whether `MW_ASSIST_RATE_LIMIT_PER_MIN` is set to `0`.

## Related

- [AI assistant: data flows and controls](/assist)
- [Using the AI assistant](/user/assist)
- [Outbound connections](/security/egress)
- [Settings reference](/admin/settings)
