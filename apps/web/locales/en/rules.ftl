# Rules (mail-filter) module (audit #1, SPEC §6.1/§10.5). English source catalog.

rules-title = Rules & filters
rules-intro = Automatically sort, tag, or act on incoming mail. Rules that use only standard tests run on the mail server as Sieve; the rest run in the engine at delivery.
rules-empty = No rules yet.
rules-new = New rule
rules-list-label = Mail rules
rules-enabled = Enabled
rules-delete-named = Delete rule { $name }

# Builder
rules-builder-title = Rule builder
rules-builder-hint = Select a rule to edit, or create a new one.
rules-name-label = Rule name
rules-match-legend = When to run
rules-match-all = Match all conditions
rules-match-any = Match any condition
rules-conditions-legend = Conditions
rules-actions-legend = Actions
rules-add-condition = Add condition
rules-add-action = Add action
rules-remove-condition = Remove condition
rules-remove-action = Remove action
rules-cond-type-label = Field
rules-cond-op-label = Operator
rules-cond-value-label = Match value
rules-act-type-label = Action
rules-act-value-label = Action value
rules-save = Save rule
rules-editing = Editing { $name }

# Condition fields and operators
rules-cond-from = From
rules-cond-to = To
rules-cond-subject = Subject
rules-cond-thread = Thread
rules-op-contains = contains
rules-op-is = is

# Actions
rules-act-move = Move to mailbox
rules-act-tag = Add tag
rules-act-archive = Archive
rules-act-suppress = Suppress notification
rules-act-stop = Stop processing

# Where it runs
rules-runs-server = Server (Sieve)
rules-runs-server-detail = Uploaded to the mail server and applied there.
rules-runs-engine = Engine
rules-runs-engine-detail = Applied by the engine at delivery.

# Panes
rules-panes-label = Rule tools
rules-tab-builder = Builder
rules-tab-raw = Raw Sieve
rules-tab-dryrun = Dry run

# Raw editor
rules-raw-label = Raw Sieve source
rules-raw-reset = Reset to generated
rules-lint-clean = No problems found.

# Dry run
rules-dryrun-title = Dry run
rules-dryrun-help = Enter a sample message to preview which rules would fire.
rules-dryrun-results = Dry-run results
rules-dryrun-none = No enabled rules to evaluate.
rules-dryrun-nomatch = No match
rules-dryrun-stopped = Not reached (a prior rule stopped processing)
