# Upgrade note: forced headers, skip sent copy and priority headers now apply

Applies to deployments, in engine mode and in proxy mode, that have set
`compose.headers.forced`, `compose.sentFolder.skipCopy` or
`mail.priority.sendHeaders` in the admin console, globally, for a domain, for a
user or in a feature profile. From their introduction until this release the
server read these settings, but the message it sent or forwarded did not include
their changes:

- `compose.headers.forced`: the listed headers were not added to sent messages.
- `compose.sentFolder.skipCopy`: a copy of each sent message was still kept or
  filed, as the client asked.
- `mail.priority.sendHeaders`: the priority headers were written as the client
  sent them, without the administrator's list applied.

Refusals in the same request path were not affected and applied throughout:
`headerNotAllowed`, `replyToNotAllowed`, `legalHold`, `processingRestricted`,
`permanentDeleteDenied` and `sizeComputeOff`. In proxy mode the internal
`mailwomanPermanent` argument was also forwarded to the upstream JMAP server; it
is now removed before forwarding.

## Before you upgrade

Review these three settings in **Settings** in the admin console and in each
feature profile. After the upgrade they apply to every message sent through
`/jmap/api` (cookie and native bearer sessions): forced headers appear in sent
mail, no sent copy is kept where `compose.sentFolder.skipCopy` is on, and
priority headers follow `mail.priority.sendHeaders`. Clear or change any value
that was saved but not meant to take effect.
