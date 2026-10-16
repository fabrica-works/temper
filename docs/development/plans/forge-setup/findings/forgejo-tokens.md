# Forgejo 17.0.0's token, user and org routes, probed (2026-10-16)

Probed by hand on loopback with the pinned binary
(`~/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`),
a scratch work path removed afterwards; the commands and bodies are in
`~/src/rust/tmp/temper/forge-setup-plan/spike/tokens.md`. The admin's
token was made by `forgejo admin user generate-access-token --username
temper-admin --token-name probe-1 --scopes
write:admin,write:organization,write:user --raw` (a comma-joined
`--scopes` is taken; exit 0). Each row below becomes a conformance fact
in the increment named.

| # | Request | Answer | Fact in |
|---|---|---|---|
| 1 | `POST /api/v1/admin/users` `{username, email, password, must_change_password: false, full_name: "temper:bot"}` under the admin's token | 201; `GET /api/v1/users/temper-bot` shows `full_name` `temper:bot` | temper 01.2 |
| 2 | `POST /api/v1/admin/users/temper-bot/tokens` `{name, scopes}` | 201; `sha1` is the token, `token_last_eight` its end; the list leaves `sha1` blank | temper 01.3 |
| 3 | `GET /api/v1/admin/users/temper-bot/tokens` | 200; rows `{id, name, scopes}` | temper 01.3 |
| 4 | `DELETE /api/v1/admin/users/temper-bot/tokens/<id>`, and `…/<name>` | 204 both | temper 01.3 |
| 5 | `GET /api/v1/user` with a token of `write:repository` alone | **403** `token does not have at least one of required scope(s): [read:user]` | temper 01.3 |
| 6 | the same with `write:repository,read:user` | 200, `login` `temper-bot` | temper 01.3 |
| 7 | `POST /api/v1/admin/users/temper-admin/tokens` (the admin's own) | 201 | temper 01.3 |
| 8 | `POST /api/v1/orgs` `{username: acme, visibility: private}` with `write:admin` alone | 403, needs `write:organization`; with `write:admin,write:organization`: 201, private | temper 02.1 |
| 9 | `POST /api/v1/admin/users/temper-admin/orgs` with `write:admin` alone | 201, private | temper 02.1 |
| 10 | `GET /api/v1/user` under basic auth, for a user made with `must_change_password: true` | **403** `You must change your password. Change it at: …/user/change_password` | temper 03.2 (the person) |

What the plan takes from them: the bot's git token needs `read:user`
beside `write:repository` for reconcile's check that it answers
`GET /user` as the bot (row 5; setup.md 5 step 3 as asked of the
designer); the process tier's person takes a token made on the admin's
routes, since the API refuses its first password (row 10; decisions.md
20); a token is deleted by its listed id (row 4).
