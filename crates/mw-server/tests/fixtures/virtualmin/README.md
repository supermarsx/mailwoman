# Virtualmin Remote API fixtures

Used by the Virtualmin fixture test in `crates/mw-server/src/hosting_transport.rs`.

No live Virtualmin server produced these. Each `*.json` body is the output of
Virtualmin's own converter, `convert_remote_format` in `json-lib.pl` from
[virtualmin-gpl](https://github.com/virtualmin/virtualmin-gpl) at commit
`1f615ee62e1fad6515ffde015536c67c9965597a`, applied to the matching `*.txt`
file. That is what `remote.cgi` does for a request with `json=1` (it forces
`multiline`). The `*.txt` files follow the line format printed by
`list-domains.pl` and `list-users.pl` in multiline mode; the values in them are
made up.

API reference: <https://www.virtualmin.com/docs/development/remote-api/>

Regenerate (Perl with JSON::PP, a checkout of virtualmin-gpl):

```sh
perl record.pl ./virtualmin-gpl/json-lib.pl list-domains 0 list-domains.txt list-domains.json
perl record.pl ./virtualmin-gpl/json-lib.pl list-users 0 list-users.txt list-users.json
perl record.pl ./virtualmin-gpl/json-lib.pl create-user 0 create-user.txt create-user.json
perl record.pl ./virtualmin-gpl/json-lib.pl create-user 256 create-user-exists.txt create-user-failure.json
```
