# Upgrade note: retention jobs are scheduled

Applies to every deployment.

## What changes

- **Three retention jobs now run daily by themselves.** Before this release nothing created a scheduled task for `user_retention` (people's own retention rules and the organisation-wide mail rules), `retention_archive` (archive check and removal of expired archive files) or `gc_uploads` (upload cleanup), so none of them ran unless someone started it. On its first start with this release the server creates one task for each job.
- **7-day preview on servers that already have accounts.** For the first 7 days after the upgrade each new task only previews: a scheduled run changes nothing and records "would change N". After that date the task deletes. A server with no accounts starts deleting at once.
- **Upload cleanup keeps uploads that are still needed.** `gc_uploads` and `mailwoman maintenance gc-uploads` used to remove every upload older than the cutoff. They now keep an upload that a draft saved on the server, a contact photo or a submission still names. The scheduled task removes uploads older than 30 days; the command-line default stays 24 hours.
- **Usage counters are pruned.** The retention sweeper now deletes usage counter rows older than `usage.retentionDays` (default 730 days). Before this release the setting had no effect.
- The tasks are created once. A job you switch off stays off after a restart, and a deleted task is not recreated.

## What to do

1. After the upgrade, open **Compliance > Retention** and look at **Scheduled retention jobs**. Each job shows its next run and, after its first scheduled run, how many messages, archive files or uploads it would change.
2. If a count is higher than you expect, switch that job off before the date shown in "starts deleting on", then review the rules: people's rules under **Settings > Mail > Retention**, `retention.user.mandatory`, the retention classes and the archive settings.
3. Drafts kept only in a browser and messages in a client's offline send queue are not visible to the server. An upload they use is removed after 30 days; reopening such a draft afterwards shows the attachment as missing. Tell people who keep unsent drafts with attachments in the browser for weeks.

See [Retention](../../admin/retention.md#scheduled-retention-jobs).
