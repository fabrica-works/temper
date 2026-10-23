# Signing people in

Provisional, 2026-10-23. How people sign in to a deployment that people
reach at a name of its own, over HTTPS behind a TLS proxy: its public
address, its members, and Google's sign-in on the clients' listener,
which ends in jig's `Mint` (jig's `protocol/client.md`, section 8). The
printed link stays for recovery. temper knows nothing of one deployment:
every name here is an example, temper at `https://temper.example.com`.
temper as Forgejo's sign-in provider is still `../draft/sign-in.md`,
section 5.

## 1. In one page

- **A public address,** `application.clients.public`: the origin
  people's browsers use (section 2). jig's mapping admits its host beside
  the listener's, and sets `Secure` on an exchange that came by it; every
  address temper gives out is the public origin's.
- **Members are configured** (section 3): `application.sign_in.members`,
  each a Google address and a login. Google signs anyone in; temper
  mints for a member alone. An identity is Google's provider number and
  the member's address as configured.
- **Bound at the first sign-in:** the Google subject that first signed in
  with a member's address is kept in temper's main store, the forge
  connector's `Binding` family (section 5); another subject with that
  address is refused.
- **Two routes on the clients' listener** (section 4):
  `GET /sign-in/google` starts the sign-in and binds its state to the
  browser by a cookie; `GET /sign-in/google/return` checks Google's
  answer, the member and the binding, sends `Mint`, and on `Minted`
  sets jig's cookie and sends the browser to where it was going.
- **temper's own sans-IO sign-in** over skein-oauth's client: each
  sign-in under way is a `skein_oauth::Client`, its HTTP requests
  carried by the clients' fetcher (section 6), so the flow joins the fake
  issuer as records in its world.
- **Every ending has a page of its own,** server-owned and stock, never
  the client's (4.3).
- **The browser's signed-out page is the sign-in page,** temper's mark,
  "Sign in" and "Continue with Google", told the ways by the page itself
  (section 7). The terminal is unchanged.
- **The printed link signs in the first owner's Google identity** (2.3),
  so the link and Google sign in the same person.
- **No real Google in any test:** `skein-fake-oauth` stands in, its
  addresses given by the configuration's `endpoints` (3.1).

## 2. The deployment

### 2.1 The host

`temper serve`, standalone, its clients' listener on loopback at a fixed
port; a TLS proxy terminates TLS for temper's name and passes it on, the
`Host` as the browser sent it; plain HTTP is answered with a redirect to
HTTPS. The proxy denies the printed link's route,
`/jig/v1/sign-in/link`, from outside (2.3). jig's stream, `GET
/jig/v1/stream`, is passed unbuffered, with a read timeout longer than
its heartbeat (jig's `protocol/client.md`, 17.2).

### 2.2 The public address

`application.clients.public`: `null`, or the HTTPS origin the people's
browsers use (`configuration.md`, section 3). With one:

- **The Host allow-list** (jig's `protocol/browser.md`, 12.2) holds its
  host, with its port where it is not 443, beside the listener's own
  address: jig's `Settings { prefix: "/jig", public: Some(host) }`. With
  none, `public: None`, the listener's address alone.
- **`Secure` by the exchange's `Host`** (jig's `protocol/client.md`,
  17.3): jig's cookie is `Secure` on an exchange that came by the public
  name, never on one by the listener's address; temper's sign-in cookie
  (4.2) takes the same decision, by jig's `via`.
- **Every address temper gives out** is the public origin's: Google's
  return (4.2), and a GitHub App's redirect and setup URLs, were a GitHub
  forge configured (`../client/forge.md`, 9.1). With none, they are the
  listener's, `http://<listener>/`, as before.
- **The bundle is the same,** served at every other `GET`
  (`../client/browser.md`, 2.2).

### 2.3 The printed link

- **It stays, for recovery:** the owner's way in when Google or the
  configuration is wrong, and the terminal's only way.
- **On the host alone:** jig enables it only on a loopback listener; the
  proxy denies its route from outside; the owner reaches it through an
  SSH tunnel to the listener.
- **Whom it signs in:** with `sign_in` configured, the first of jig's
  `owners` whose identity is Google's provider, with that member's login
  as login and name; there must be one, or the start is refused, naming
  `engine.owners`. With `sign_in` `null`, the first owner of the link's
  provider, `owner` and `Owner`, as before.

## 3. Who may sign in

- **An identity is Google's provider and the member's address.**
  `application.sign_in.google.provider` names jig's provider of kind
  application that numbers Google's sign-in (jig's
  `protocol/configuration.md`, section 3, `providers`). The subject
  temper gives jig is the member's address exactly as configured, not
  Google's `sub`, so that a member can be named before their first
  sign-in: jig's `owners` and a project seed's roles name them.
- **Members are configured:** `application.sign_in.members`, each
  `{"address", "login"}` (3.1). Each of jig's owners of Google's provider
  is a member: its subject is a member's address, checked at start.
- **Google's word on the address:** the ID token's `email`, with
  `email_verified` true, or the sign-in is refused as not verified.
- **A member is the configured address equal to Google's,** compared with
  ASCII letters folded to lower case; the identity's subject is the
  configured spelling.
- **Bound to Google's account at the first sign-in:** temper keeps the
  `sub` that first signed in with a member's address (section 5). A
  later sign-in with that address and another `sub` is refused: an
  address Google gave to a new account is not the member.
- **The check is temper's, before `Mint`:** a member, bound to this
  `sub` or not yet bound, is minted; anyone else is answered with a page
  that says why, and nothing is minted.
- **Adding a member** is a change to the configuration and a restart. A
  removed member's binding stays, unread, until the address is a
  member's again.

### 3.1 The configuration

```
"application": {
  "clients": {"listener": "127.0.0.1:8367", "bundle": null,
              "public": "https://temper.example.com"},
  "sign_in": {
    "google": {
      "provider": 2,
      "client": "1234-abc.apps.googleusercontent.com",
      "secret": "google-client.secret",
      "endpoints": null
    },
    "members": [{"address": "sam@example.com", "login": "sam"}]
  },
  ...
}
```

`configuration.md`, section 3 holds each member's rule. In short:

- `clients.public`: `null`, or `https://HOST[:PORT]`: a lower-case DNS
  name, no user, path, query, fragment or final `/`, the port not 443;
  `http://` to a loopback host alone, for a test.
- `sign_in`: `null` for none, never missing: the browser's signed-out
  page keeps its printed link's line, and both routes answer not found.
- `google.provider`: a provider of jig's `engine.providers` of kind
  application.
- `google.client`: Google's client id, 1 to 256 bytes of visible ASCII.
- `google.secret`: a data path, a secrets' file holding the client
  secret (`configuration.md`, section 6).
- `google.endpoints`: `null` for Google's own, as in every deployment;
  otherwise the addresses of an issuer that is not Google, for a test
  or a fake, naming no deployment, as a GitHub forge's addresses are: `{"issuer", "authorization", "token", "keys",
  "transport"}`, each address an `https` URL, or `http` to a loopback
  host; `transport` as a forge's (`"plaintext"` only to loopback, or
  `{"tls": {"server_name", "trust"}}`). Google's own: issuer
  `https://accounts.google.com`, authorization
  `https://accounts.google.com/o/oauth2/v2/auth`, token
  `https://oauth2.googleapis.com/token`, keys
  `https://www.googleapis.com/oauth2/v3/certs`, TLS with the machine's
  roots. Each host is resolved at start; one that does not resolve
  refuses the start, naming it.
- `members`: 1 to `sign_in.members` entries (`configuration.md`, section
  7). `address`: 3 to 254 bytes of visible ASCII, one `@` with bytes on
  both sides; unique, folded. `login`: Forgejo's shape, 1 to 40 bytes of
  ASCII letters, digits, `-`, `_` and `.`, its first and last a letter
  or a digit; unique, folded.

Google's registration is a web application's OAuth client in Google's
console, its redirect URI the public origin's `/sign-in/google/return`,
with the non-sensitive scopes `openid email profile` alone.

## 4. Signing in with Google

### 4.1 The routes

On the clients' listener, after what jig's mapping claims and beside the
GitHub routes (`../client/browser.md`, 2.2): `GET /sign-in/google` and
`GET /sign-in/google/return`, any other method `405`, any other path
under `/sign-in/` the page as today. With `sign_in` `null`, both answer
not found. Every answer of theirs carries the page's policy,
`Cache-Control: no-store` and `Referrer-Policy: no-referrer`.

### 4.2 The flow

1. **Start:** `GET /sign-in/google?then=PATH`, the address of the page
   the person was on, and `&account=choose` for "Use another Google
   account". `then` is a path on the origin: one leading `/`, not `//`,
   no `\`, visible ASCII once decoded, at most 1024 bytes; absent or
   anything else is `/`.
   - Past `sign_in.pending` sign-ins under way, the busy page (4.3).
   - Otherwise temper draws the sign-in's tag, 16 bytes, from the
     clients' generator, and starts a `skein_oauth::Client` (6.1). Its
     key set is fetched first; then it gives its authorization URL, with
     `prompt=select_account` for `account=choose`.
   - The answer, once the URL is there: `303`, `Location` the URL, and
     `Set-Cookie: temper-sign-in=<tag, base64url>; Path=/sign-in/google;
     HttpOnly; SameSite=Lax; Max-Age=<sign_in.deadline in seconds>`,
     with `Secure` by the exchange's `Host` (2.2). A key set not fetched
     is the unreachable page (4.3).
2. **Google** signs the person in and sends the browser to `GET
   /sign-in/google/return?code=…&state=…`, or `?error=…&state=…`. Lax
   lets the sign-in cookie come back with this cross-site navigation;
   jig's, strict, does not, and is not needed.
3. **The return,** whose every answer clears the sign-in cookie
   (`Max-Age=0`, the same `Path` and `Secure`):
   - no cookie, or one naming no sign-in under way: the expired page;
   - `error`: the sign-in is cancelled; the cancelled page, saying
     Google's word;
   - otherwise the redirect, the return address and the request's query,
     goes to the sign-in's client, which checks the state in constant
     time, exchanges the code (the client secret in the body), and
     checks the ID token: its issuer, audience, expiry, nonce and RS256
     signature by the key set (skein's `oauth.md`, 2.2). A state that
     does not match, a deadline past or an exchange that fails: the
     expired page, or the unreachable page when Google could not be
     reached.
4. **The claims:** temper reads `email`, `email_verified` and `name` from
   the checked token's payload (6.2). No `email`, or `email_verified`
   not true: the unverified page. An address no member's: the
   not-a-member page.
5. **The binding:** temper asks the engine's root to bind the member's
   address to the token's `sub` (section 5). Another `sub` bound: the
   other-account page. The root not running yet, or stopping: the busy
   page.
6. **`Mint`** with the identity (Google's provider, the member's address
   as configured), the member's login, and Google's `name` where it is
   UTF-8 within jig's bound, the login otherwise. On `Minted`: `303` to
   `then`, jig's cookie set by the mapping's `cookie(&settings, via,
   &credential, remaining)`, `remaining` the sign-in's expiry less the
   wall clock. On `MintRefused`, busy or not ready: the busy page; its
   other reasons are configuration the start checks, and answer the busy
   page too.
7. **Google's tokens are dropped:** the client's record is never kept,
   and the sign-in under way ends with its answer. The page at `then`
   loads, and its own exchanges carry jig's cookie: the client signs in
   by `Transport` (jig's `client/domain/client.md`, section 12).

- **The return's call is held** while the exchange, the binding and
  `Mint` run, within `sign_in.deadline`; past it, the expired page.
- **Sign-ins under way are memory alone:** a restart drops them, and a
  return after it is answered the expired page.

### 4.3 Its endings

Each a page of the server's own, HTML in temper-service as the GitHub
routes' are (`../client/forge.md`, 9.1): the level-1 heading, a
paragraph, and its links; no script; stock, as the sign-in page is.
`ADDRESS` is the token's address, escaped. "Use another Google account"
is `/sign-in/google?then=THEN&account=choose`, "Continue with Google"
`/sign-in/google?then=THEN`, "Back to sign in" `THEN`.

| Ending | Status | Heading | Paragraph | Links |
|---|---|---|---|---|
| a member, minted | `303` to `then` | | | |
| not a member | 403 | "ADDRESS can't sign in here" | "Google signed you in, but no one here signs in with this address. Ask an owner to add it, or use the Google account they added." | Use another Google account; Back to sign in |
| bound to another subject | 403 | "ADDRESS signs in here with another Google account" | "This isn't the Google account that first signed in with this address. Use that one, or ask an owner." | Use another Google account; Back to sign in |
| not verified | 403 | "Google hasn't verified ADDRESS" ("this account's address" with none) | "temper signs in only an address Google has verified. Verify it with Google, or use another account." | Use another Google account; Back to sign in |
| cancelled, or Google's `error` | 400 | "The sign-in didn't finish" | `access_denied`: "You cancelled at Google. Nothing was kept."; another: "Google answered ERROR. Nothing was kept.", ERROR its code, escaped, at most 64 bytes | Continue with Google; Back to sign in |
| expired: no cookie, a tag or state unknown, used or past its deadline, a restart since | 400 | "The sign-in didn't finish" | "It took too long, or temper restarted while you were at Google. Nothing was kept." | Continue with Google; Back to sign in |
| unreachable: the key set or the token endpoint not reached | 502 | "The sign-in didn't finish" | "temper couldn't reach Google. Nothing was kept: try again in a moment." | Continue with Google; Back to sign in |
| busy: pending full, the root not running, `MintRefused` | 503 | "temper couldn't sign you in just now" | "It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment." | Continue with Google |

## 5. The binding

- **The forge connector's family `Binding`,** byte 18 of temper's
  families (`store.md`, section 2): the forge connector is temper's one
  connector, whose `Stored` holds every family of temper's, so the
  binding is restored in the forge's restart step with the rest, and
  written in the root's decisions, with no range of its own.
- **Its key, `Binding(address)`:** the member's address folded to lower
  case. **Its row,** `{ address, subject }`: that address and Google's
  `sub`, at most 255 bytes, as skein-oauth bounds a subject.
- **Asked by the clients' composition:** `forge::Event::Bind { request,
  address, subject }`, through the root as the forge's other events
  come. The connector answers `forge::Request::Bound { request, answer }`
  once its decision's commit is durable: `Bound` (bound before to this
  `sub`, or bound now, its row saved) or `Another` (bound to another
  `sub`, nothing written). The forge holds its bindings in memory, at
  most `forge.bindings`, the configured members' count; a row past it
  is refused at restart, as each family's is.
- **Never erased:** a binding outlives its member's removal; a store made
  again binds afresh (`store.md`, 4).

## 6. The sign-in's exchanges

### 6.1 The sign-ins

`temper-service`'s `clients::google`: a table of at most
`sign_in.pending` sign-ins under way, each its tag, its `then`, its
deadline, the call held, if any, and a `skein_oauth::Client` made with
Google's `Registration` (skein's `oauth.md`, section 1): the configured
endpoints, the client id, the client secret from its file, the redirect
URI the public origin's `/sign-in/google/return`, the scopes `openid
email profile`, the form wire, `pkce_for_confidential`, and terms with
OpenID (the issuer and the key set's URL, so that no discovery document
is fetched) and, for `account=choose`, the parameter
`prompt=select_account`. The state, nonce and verifier come from the
clients' generator. Its requests go out as the client asks: `Http` to
the fetcher, `Visit` to the held start's answer; its terminals end the
sign-in: `Tokens` with the checked record, or `Failed`.

### 6.2 The claims

A small reader in `clients::google` over the record's ID token, already
checked by the client: its payload's `email` (a string within the
member's bound), `email_verified` (`true`, or the string `"true"`, as
Google has sent) and `name` (a string), each optional; nothing else is
read, and the token is dropped with the record.

### 6.3 The fetcher

`clients::fetch`: one HTTP/1.1 exchange per connection, skein-http's
client over skein-tls's client with the configured trust, or plaintext
to a loopback address alone; the origin of each request's URL must be
one of the configured endpoints', resolved at start, or the request
fails before any socket. At most `sign_in.fetches` at once, each within
the client's deadline and `sign_in.body_bytes`; its answer is the
client's `HttpResponse`. A step machine of the clients' composition,
its sockets tagged as the server's are.

## 7. The browser's sign-in page

- **The page tells the ways.** When temper-service makes the bundle's
  page at start (`../client/browser.md`, 2.2), it inserts `<meta
  name="temper-sign-in" content="WAYS">` before the page's `</head>`,
  `WAYS` the configured ways, space-separated (`google`, or nothing),
  and the `ETag` is the page's so made; a page without `</head>` is
  refused at start, naming the bundle. The web shell reads the meta at
  start and gives the view the ways at construction. Further facts of a
  signed-out page come as further `temper-` metas made the same way
  (stage 2: the forge's name, for "to continue to").
- **The signed-out main** (`../client/README.md`, 5.1), in the browser
  with a way: temper's mark, the level-1 heading "Sign in", and a link
  for each way, "Continue with Google" to `/sign-in/google?then=PATH`,
  `PATH` the printed address of the page, percent-encoded. With no way,
  the printed link's line, as before. The terminal's "Signed out." is
  unchanged.
- **A navigation, not an exchange:** the link is the page's own `href`,
  so the policy's `form-action 'none'` holds.

## 8. Testing

- **The sign-in world,** `temper-service`'s own tests
  (`crates/temper-service/src/tests/sign_in.rs`), as its GitHub routes'
  are (`../domain/testing.md`, 2.1): the clients' composition with its
  Google sign-ins, jig's clients' component and the engine's root on its
  store, people's requests as calls through `fixtures::http::Peer`, and
  `skein-fake-oauth`'s `Issuer` as Google, answering the sign-ins' HTTP
  requests as records where the fetcher stands, its accounts by
  `sign_in_as`. Stories: a member minted, `then` kept, its binding kept
  and the same `sub` minted again; a second `sub` with that address
  refused; each other ending of 4.3; a restart between start and return
  answered expired. Each story within the world's budget
  (`../domain/testing.md`, 6.1).
- **Step tests:** the configuration's members and their checks; the
  `Binding` family's codec, goldens and fuzz seed; the forge connector's
  bind in its world (`tests/engine/forge`); the claims' reader; the
  fetcher against scripted io; the endings' pages' bytes.
- **The view's tests:** the signed-out main with and without a way, and
  its link's `then`.
- **The browser tier,** one story (`../client/browser.md`, section 4):
  from the page's "Continue with Google", the sign-in cookie set on
  temper's origin comes back from the issuer's cross-site redirect, a
  fake issuer on another site (`localhost` against `127.0.0.1`), and the
  return answers a member's check, not the expired page.
- **No real-loop story.**

## 9. Open questions

- **A deployment's own sign-in page.** Default: stock. Later, perhaps a
  name and a line in the configuration, or a template of the owner's.
- **Members at runtime:** an owner's invitation and roles from the
  client wait for a design of their own. Default: the configuration's
  members, roles by jig's seed.
- **Google's Workspace domain** as a member rule (`hd`). Default:
  addresses alone.
- **A terminal sign-in for a member who is not the owner:** a device
  code's flow. Default: the browser alone.
