# Upgrade note: the admin console answers only at its configured path (26.20)

Applies to every deployment.

## What changes

- The admin console, its sign-in and its API answer at one path. There is no
  built-in fallback and no alias: every other path gets the same `404` as an
  unknown path.
- A **new installation** gets a generated path (20 random characters) on its
  first start. The server stores it and logs it once; `deploy.sh init` writes one
  to `app.env` as `MW_ADMIN_PATH` before the first start.
- An **upgraded installation** keeps the path it had: `MW_ADMIN_PATH` if it set
  one, otherwise `/admin`. With `/admin`, every start logs a warning recommending
  a path of your own. Nobody is locked out.
- The admin session cookie is scoped to the console path. Cookies issued before
  the upgrade (`Path=/`) keep working until they expire or the admin signs out.
- The console's server-metadata editor posts to `{console}/server-metadata/jmap`
  instead of `/jmap/api`.

## Moving an upgraded console off /admin

1. In the console, open Security > Console address and save a path of your own
   or a generated one (you confirm your password or second factor first). Or, on
   the server, run `mailwoman admin console-path generate` (or `set /your-path`).
2. Write the new address down.
3. Restart the server. The console moves, `/admin` answers `404`, and every admin
   session ends; sign in again at the new address.
4. Update bookmarks, reverse-proxy rules and monitoring that name `/admin`.

While `MW_ADMIN_PATH` is set it decides the path, and the console and CLI refuse
to change it; edit or remove the variable instead.

## If you lose the path

Run `mailwoman admin console-path show` on the server (with Docker:
`docker exec <container> mailwoman admin console-path show`), or set
`MW_ADMIN_PATH` and restart.

## What this protects against, and what it does not

A path nobody can guess keeps scanners and drive-by sign-in attempts away from
the console. It does not isolate the console: the console shares the mail app's
origin, so a script running in the mail app that learns the path can still reach
the console from a browser that holds an admin session. Keep the admin password,
two-factor sign-in and, where possible, a network restriction on the path.
