# HTTP listener limits in force (t57): the deployment assessment row and the
# Maintenance settings readout.
admin-http-limits-deployment = In force now: { $connections } connections ({ $connectionsSource ->
        [console] set in the console
        [explicit] set by the environment
       *[other] Auto
    }), { $requests } requests at once ({ $requestsSource ->
        [console] set in the console
        [explicit] set by the environment
       *[other] Auto
    }), { $streams } HTTP/2 streams per connection. A request at the limit waits up to { $queueWaitMs } ms, with at most { $queueMax } waiting.
admin-http-limits-warning = Warning: { $text }
