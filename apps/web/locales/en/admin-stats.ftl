# Statistics and Dashboard layout (t106): figures, charts and tables in separate sections.
admin-stats-key-figures = Key figures
admin-stats-charts = Charts
admin-stats-tables = Tables
admin-stats-group-traffic = Traffic in this period
admin-stats-group-storage = Storage and delivery, now
admin-stats-refresh = Refresh statistics
admin-stats-loading = Loading statistics…
admin-stats-requests-total = Requests
admin-stats-errors-total = Server errors (5xx)
admin-stats-p95-peak = Slowest 95th percentile
admin-stats-requests = Requests
admin-stats-errors = Server errors (5xx)
admin-stats-not-found = Not found (404)
admin-stats-latency = Response time
admin-stats-series-p50 = Median
admin-stats-series-p95 = 95th percentile
admin-stats-unit-count = Count per { $bucket }
admin-stats-unit-duration = Time per { $bucket }, in the unit shown
admin-stats-unit-accounts = Blocked accounts per domain
admin-stats-chart-about = Line chart from { $from } to { $to }.
admin-stats-bars-about = { $count ->
    [one] Bar chart with one domain.
   *[other] Bar chart with { $count } domains.
}
admin-stats-view-table = View as table
admin-stats-legend = Legend
admin-stats-empty-chart = Nothing was measured in this period yet. Choose a longer period, or check back once the server has handled some requests.
admin-stats-empty-blocked = No accounts are blocked.
admin-stats-delta-vs = vs previous period
admin-stats-delta-same = Same as the previous period
admin-stats-delta-sr-up = Up { $change } on the previous period.
admin-stats-delta-sr-down = Down { $change } on the previous period.
admin-stats-delta-sr-same = The same as the previous period.
admin-ct-group = Client load times
admin-ct-metric-timeToUsable = Time to usable
admin-ct-metric-ttfb = First byte
admin-ct-metric-dcl = DOM ready
admin-ct-metric-load = Page load
admin-ct-metric-fcp = First contentful paint
admin-ct-metric-lcp = Largest contentful paint
admin-ct-metric-folderOpen = Folder open
admin-ct-metric-folderLoadCold = Folder load, from the network
admin-ct-metric-folderLoadCached = Folder load, from cache
admin-ct-metric-messageLoadCold = Message load, from the network
admin-ct-metric-messageLoadCached = Message load, from cache or prefetch
admin-ct-device-desktop = Desktop
admin-ct-device-mobile = Mobile
admin-ct-device-unknown = Unknown
admin-ct-tile = { $metric }, { $stat }
admin-ct-samples =
    { $count ->
        [one] { $count } sample
       *[other] { $count } samples
    }
admin-ct-no-samples = No samples in this period.
admin-ct-few-samples = Only { $count } samples; at least { $min } are needed.
admin-ct-chart-usable = Time to usable
admin-ct-group-folder = Folder load times
admin-ct-group-message = Message load times
admin-ct-chart-folder-cold = Folder load from the network
admin-ct-chart-folder-cached = Folder load from cache
admin-ct-chart-message-cold = Message load from the network
admin-ct-chart-message-cached = Message load from cache or prefetch
admin-ct-folder-off = Folder load times are switched off by default. Only people who turned them on are measured.
admin-ct-message-off = Message load times are switched off by default. Only people who turned them on are measured.
admin-ct-chart-note = Percentiles show only for intervals with at least { $min } samples.
admin-ct-empty-chart = No interval has the { $min } samples needed to show a median. Choose a longer period.
admin-ct-col-samples = Samples
admin-ct-col-metric = Measure
admin-ct-col-device = Device
admin-ct-table-title = Client load times by device
admin-ct-table-caption = Samples, median and 95th percentile for each measure and device class.
admin-ct-empty-table = No browser load times were reported in this period.
admin-ct-failed = Client load times could not be loaded.
admin-ct-disabled = Collection of client load times is switched off. The figures below are from before it was switched off.
admin-ct-note = From signed-in browsers only, { $rate }% of sessions sampled, anonymous and aggregated. Time to usable runs from navigation start until the message list shows its first rows. Figures are estimates, within about { $error }% of the true value.
