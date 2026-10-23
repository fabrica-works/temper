# Signing in with Google, and Forgejo through temper

Draft, 2026-10-23. A temper deployment that people reach at a name of
its own, over HTTPS, where they sign in with their Google account, and
whose Forgejo signs them in through temper: once signed in to temper, a
person opens Forgejo signed in, with nothing more to do. The names here
are examples: temper at `https://temper.example.com`, Forgejo at
`https://git.example.com`. It records a session with the user on
2026-10-23. It crosses skein, jig and temper on purpose; an owner
splits it.

The mockups draw its pages ahead of this draft: `sign-in.html`, in the
world Google sign-in (`../client/mockups/README.md`).

## 1. In one page

- **One host, two sites, a TLS proxy in front** (section 2). `temper
  serve` runs the engine and its worker, its clients' listener on
  loopback, and a proxy (nginx, here) terminates TLS for temper's name
  in front of it. Forgejo runs beside it, managed
  (`../protocol/setup.md`, section 5), behind the proxy at a name of its
  own. Not `temper box`: Forgejo is the owner's, run as a service of its
  own.
- **Two hops of OpenID Connect:**

  ```
  Google ──▶ temper.example.com ──▶ Forgejo, at git.example.com
     temper signs people in     temper signs Forgejo's users in
  ```

- **Google signs people in to temper** (section 4) through jig's
  application sign-in: temper's routes run Google's flow with skein's
  OAuth client, and jig's `Mint` makes the sign-in (jig's
  `protocol/client.md`, section 8). Both are built; temper has never
  used them for people.
- **The sign-in page is stock** (4.1): the same for every deployment,
  "Sign in" and a button for each way of signing in, with nothing of
  the deployment's own. A deployment's own name and words wait
  (section 9).
- **Only members sign in** (section 3): the configuration names each by
  their Google address. Google signs anyone in; temper mints for a
  member alone, and answers anyone else with a page that says so.
- **temper is Forgejo's OpenID provider** (section 5): Forgejo has an
  authentication source that names temper, so Forgejo's sign-in
  bounces through temper, which answers at once for a person signed
  in. temper's "Open Forgejo ↗" starts that sign-in itself, so a person
  signed in to temper is signed in to Forgejo with no click.
- **The provider is new, the client side is not.** skein has the OAuth
  client, its ID token's checks and an RS256 signer; jig has `Mint`.
  What no repository has yet is the issuer's side: the discovery
  document, the authorization, the token, the user's claims and the
  key set (section 7).
- **Two stages** (section 7.4): first Google into temper, with Forgejo
  signing in its own way; then temper as Forgejo's provider.
- **Signing out of temper does not sign out of Forgejo** (section 6),
  and the signed-out page says so.

## 2. The deployment

### 2.1 The host

- **`temper serve`,** standalone (`../protocol/configuration.md`,
  section 2): the engine and one worker in one process, the agents on
  the same host. Its clients' listener on loopback at a fixed port,
  `127.0.0.1:8367`; its webhooks' listener on loopback, which Forgejo
  reaches on the same host.
- **Forgejo, managed** (`../protocol/setup.md`, section 5): run by its
  owner as a service, its runner beside it; temper sets itself up on
  it with a site admin's token and makes its bot. Forgejo's `ROOT_URL`
  is `https://git.example.com/`.
- **The proxy terminates TLS for both names,** a certificate each, and
  passes each to its loopback listener. For temper:
  - the `Host` passed through as the person's browser sent it, which
    the Host allow-list then checks (2.2);
  - jig's stream, `GET /jig/v1/stream`, unbuffered (nginx's
    `proxy_buffering off`) and with a read timeout longer than the
    stream's heartbeat (jig's `protocol/client.md`, 17.2);
  - the printed link's route, `/jig/v1/sign-in/link`, denied from
    outside (2.3);
  - plain HTTP answered with a redirect to HTTPS, and nothing else.

### 2.2 The public address

The configuration's new `application.clients.public`: the origin the
people's browsers use, `https://temper.example.com`. It is where the
listener's being on loopback stops telling temper what the browser
sees:

- **The Host allow-list** (jig's `protocol/browser.md`, 12.2) holds the
  public name beside the listener's loopback address.
- **The cookie is `Secure`** on an exchange that came by the public
  name. jig's mapping leaves `Secure` off on a loopback listener (jig's
  `protocol/client.md`, 17.3); behind the proxy the listener is loopback
  but the browser's origin is not. The mapping decides by the
  exchange's `Host`: the public name sets `Secure`, the loopback address
  does not.
- **Every address temper gives out** is the public origin's: Google's
  return (4.2), the provider's issuer and endpoints (5.2), and a GitHub
  App's redirect and setup URLs, were a GitHub forge configured.
- **The bundle is the same,** served at every other `GET` as today
  (`../client/browser.md`, 2.2).

### 2.3 The printed link

- **It stays, for recovery:** the owner's way in when Google or the
  configuration is wrong, and the terminal's only way (`temper open`
  signs in by a link, `../client/README.md`, section 7).
- **On the host alone.** jig enables it only where its clients reach it
  on loopback (jig's `protocol/configuration.md`, section 3, `clients`);
  the listener is, and the proxy denies its route from outside. The
  owner reaches it through an SSH tunnel to the listener.
- **It signs in the first owner** as jig's `owners` names them: the
  owner's Google identity (section 3), so the link and Google sign in
  the same person.

## 3. Who may sign in

- **An identity is Google's provider and the member's address.** jig's
  `providers` numbers Google's sign-in, of kind application. The
  subject temper gives jig is the member's Google address as the
  configuration writes it, not Google's own subject, so that a member
  can be named before their first sign-in: jig's `owners` and a project
  seed's parties' roles name them on a first start (jig's
  `protocol/configuration.md`, section 3).
- **Members are configured:** `application.sign_in.members`, each with
  their Google address and a login: the name Forgejo gives their
  account (5.4), unique among members, in a Forgejo username's shape,
  checked at start. jig's `owners` are members, checked at start.
- **Bound to Google's account at the first sign-in.** temper keeps the
  Google subject (`sub`) that first signed in with a member's address,
  in a record of its own in the store. A later sign-in with the same
  address and another subject is refused (4.3): an address Google
  gave to a new account is not the member.
- **Google's word on the address:** `email_verified` true, or the
  sign-in is refused.
- **The check is temper's,** before `Mint`: a member, bound or not yet
  bound, is minted; anyone else is answered with the refusal page and
  nothing is minted.
- **Adding a member is a change to the configuration and a restart**
  (section 9).

## 4. Signing in with Google

### 4.1 The sign-in page

- **Signed out, every page of temper's is the sign-in page.** In the
  browser, the view's signed-out main (`../client/README.md`, 5.1)
  becomes temper's mark, "Sign in", and a button for each way of
  signing in the configuration gives: here one, "Continue with Google".
  It replaces "Signed out. To sign in, open a sign-in link `temper
  serve` printed." where a way is configured; a deployment with none
  keeps that line.
- **Stock:** nothing on it is the deployment's own: no name, no words
  about it, no note on who may sign in. The same page serves every
  deployment, and says only what temper itself knows: arriving from
  Forgejo, "to continue to" the forge (5.3); signed out, that the forge
  keeps its sign-in (section 6).
- **The terminal is unchanged:** "Signed out." and its own line
  (`../client/README.md`, 6.6).
- **Where the person was going is kept:** the page's address, carried
  through the sign-in as `then`, a path on the public origin, so a link
  into a chat opens the chat once signed in.

### 4.2 The flow

1. "Continue with Google" goes to `GET /sign-in/google?then=PATH`,
   temper's route on the clients' listener. temper starts a sign-in
   with skein's OAuth client as a confidential client with Google's
   registration (skein's `oauth.md`, sections 1 and 2): its state, its
   nonce and its PKCE verifier. It sets a short cookie that binds the
   state to this browser, `temper-sign-in`, `HttpOnly; Secure;
   SameSite=Lax; Path=/sign-in/google`, which lives as long as the
   sign-in's deadline, and answers `303` to Google's authorization URL,
   scopes `openid email profile`.
2. Google signs the person in, and sends the browser to
   `GET /sign-in/google/return?code=…&state=…`. Lax lets the binding
   cookie come back with this cross-site navigation; jig's cookie,
   strict, does not, and is not needed here.
3. temper checks the state against the cookie, exchanges the code with
   Google's token endpoint, checks the ID token (its issuer, audience,
   expiry, nonce and signature, by Google's published keys; skein's
   `oauth.md`, 2.2) and then the member (section 3).
4. temper sends `Mint` with the identity, the member's login and
   Google's `name`. On `Minted`, temper's answer sets jig's cookie by
   the mapping's function, clears the binding cookie, and is `303` to
   `then`. Google's tokens are dropped: temper keeps nothing of them.
5. The page at `then` loads, and its own exchanges, same-origin, carry
   jig's cookie: the client signs in by `Transport` (jig's
   `client/domain/client.md`, section 12).

Google's registration is a web application's OAuth client in Google's
console, its redirect URI the public origin's `/sign-in/google/return`,
with the non-sensitive scopes alone. Its client id is in the
configuration; its secret is a secrets' file
(`../protocol/configuration.md`, section 6).

### 4.3 Its endings

The return route answers each ending with a page of its own, as the
GitHub routes do (`../client/forge.md`, 9.1), never a page of the
client's: the person is not signed in. Each is stock, as the sign-in
page is.

| Ending | Its page |
|---|---|
| a member | no page: `303` to `then` |
| not a member | "ADDRESS can't sign in here: Google signed you in, but no one here signs in with this address", with "Use another Google account", a fresh sign-in with Google's account chooser |
| the address bound to another Google account | "ADDRESS signs in here with another Google account", with "Use another Google account" |
| the address not verified by Google | "Google hasn't verified ADDRESS", with "Use another Google account" |
| cancelled at Google, or Google's `error` | "The sign-in didn't finish", saying Google's word, with "Try again" |
| the state unknown, used or past its deadline, or a restart since | "The sign-in didn't finish: it took too long, or temper restarted", with "Try again" |
| `MintRefused`: not ready or busy | "temper couldn't sign you in just now", with "Try again" |

## 5. temper as Forgejo's sign-in provider

### 5.1 What a person sees

- **From temper:** "Open Forgejo ↗" goes to Forgejo's own route that
  starts a sign-in with the temper source, `/user/oauth2/temper` on
  Forgejo's origin; Forgejo sends the browser to temper, temper answers
  at once, and Forgejo opens signed in. No click.
- **At Forgejo first:** Forgejo's sign-in page offers "Sign in with
  temper". Signed in to temper, one click; signed out, temper's sign-in
  page says "to continue to" the forge, Google follows, then Forgejo,
  signed in.
- **The first time,** Forgejo makes the person's account, named by
  their login (5.4); an account already there is linked once, by its
  password (5.4).

### 5.2 The endpoints

On temper's clients' listener, its issuer the public origin:

| Route | What it is |
|---|---|
| `GET /.well-known/openid-configuration` | the discovery document: the issuer, the endpoints below, the code flow alone, the scopes, the claims, the signing algorithm |
| `GET /oauth/authorize` | the authorization, a page of the client's (5.3) |
| `POST /oauth/token` | the code's exchange, from Forgejo's server, with its client secret: the ID token and an access token |
| `GET /oauth/userinfo` | the claims, for the access token |
| `GET /oauth/keys` | the key set |

- **One client, configured:** Forgejo's, `forgejo`, for the forge
  temper knows it as, its redirect URI exactly Forgejo's origin's
  `/user/oauth2/temper/callback`, its secret a secrets' file. Several
  clients are a list; one is all a deployment with one Forgejo needs.
- **Codes and access tokens are the engine's memory:** a code lives a
  minute and is spent once, bound to its client, its redirect URI, its
  sign-in, its nonce and, when Forgejo sends one, its PKCE challenge;
  an access token lives five minutes. A restart drops them: a sign-in
  to Forgejo under way fails, and the person signs in again.
- **The signing key** is drawn at the first start and kept in jig's
  secrets' store. Its algorithm is RS256 (section 9).
- **What Forgejo 17's client does,** and the endpoints meet (its goth
  v1.82.0): it always sends a PKCE challenge, S256, and its verifier at
  the exchange; it authenticates at the token endpoint by Basic first,
  and by its secret in the body when that fails, so the token endpoint
  takes both; it calls the userinfo endpoint only when the discovery
  document advertises one, by `GET` with the access token as Bearer,
  and requires its `sub` to be the ID token's. The ID token is required
  in the token's answer either way.

### 5.3 The authorization

jig's cookie is `SameSite=Strict` and its `Path` is jig's prefix
(jig's `protocol/client.md`, 17.3), so a request to `/oauth/authorize`
never carries it, and a navigation that started on another site (a
link to Forgejo in a mail) would not carry it anywhere. So the
authorization is decided by the client, not by the route:

1. `GET /oauth/authorize?…` is answered with temper's page, as every
   other `GET` is. The client root's routing reads it as a place of its
   own, `Authorize`, its parameters in the address.
2. The client signs in by `Transport`: its exchanges are same-origin,
   started by temper's page, so the cookie goes with them whatever site
   the navigation came from.
3. **Signed in:** the client sends jig's new `Authorize` with the
   request's parameters. jig checks the client, the redirect URI and
   the scopes, draws the code for the signer's sign-in, and answers the
   address to go to, Forgejo's callback with the code and Forgejo's
   state. The page replaces its address with it, so the back button
   never returns to the authorization. The person sees "Continuing to
   Forgejo…" for as long as that takes.
4. **Signed out:** the sign-in page, under its "Sign in", "to continue
   to" the forge as the client names it, "Forgejo at HOST"
   (`../client/forge.md`, 5.4), its `then` the authorization's address,
   so Google's return lands back at step 1.
5. **Refused** (an unknown client, a redirect URI not registered): the
   page says so and goes nowhere: a redirect URI that does not match is
   never followed.

No consent page: Forgejo is the deployment's own, configured by its
owners.

### 5.4 The claims, and Forgejo's account

- **`sub`:** the identity, Google's provider number and the member's
  address. It outlives the store: a deployment started afresh gives
  Forgejo the same `sub` for the same member, never a party's number,
  which a fresh store would give to someone else.
- **`preferred_username`:** the member's login. **`email`,**
  verified: the member's address. **`name`:** as Google gave it. No
  `nickname`: Forgejo's client takes a user's nickname from `nickname`,
  then `preferred_username`, so without one `USERNAME = nickname` and
  `USERNAME = preferred_username` (below) both name the account by the
  login.
- **Forgejo's settings** (its configuration cheat sheet):
  - `[oauth2_client] ENABLE_AUTO_REGISTRATION = true` and `USERNAME =
    preferred_username`: a member's first sign-in makes their account,
    named by their login;
  - `ACCOUNT_LINKING = login`, Forgejo's default: an account that
    already exists with that name or address is linked once, by its
    password. Not `auto`, which links by a matching name or address
    alone;
  - `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true`; and, once the
    source works, `ENABLE_INTERNAL_SIGNIN = false`, so the temper
    source is Forgejo's only sign-in on the web. Its admin keeps
    Forgejo's command line, and temper's bot its tokens.
- **The source is added on Forgejo's host,** since Forgejo's API has no
  route for authentication sources (Forgejo 17):
  `forgejo admin auth add-oauth --name temper --provider openidConnect
  --key forgejo --secret … --auto-discover-url
  https://temper.example.com/.well-known/openid-configuration`. temper's
  setup for a managed Forgejo prints it with its values; for an owned
  Forgejo, temper's preparation runs it (`../protocol/setup.md`, 6.3).
  The name `temper` is the callback's path segment, so it is fixed.

## 6. Signing out

- **Signing out of temper ends temper's sign-in only.** Forgejo keeps
  its own session, and Google its own. While a forge signs its people
  in through temper, the signed-out page says so: "You signed out.
  Forgejo at HOST keeps its own sign-in: sign out there too."
- **Signing out of Forgejo** ends Forgejo's session; its next sign-in
  bounces through temper, and a person still signed in to temper is
  signed in again at once.
- **Single sign-out is open** (section 9).

## 7. What each repository gains

### 7.1 skein

- **The issuer's half of OpenID Connect,** sans-IO, beside `skein-oauth`:
  an authorization request checked against a client's registration;
  the token request, with a client's secret by basic or in the body,
  and its PKCE; the ID token's claims and signature, by
  `skein_tls::sign_rs256` or the algorithm chosen (section 9); the
  discovery document and the key set. A crate of its own, as the
  client's is.
- **A world for it against `skein-oauth`:** the client and the issuer
  each other's peer, with `skein-fake-oauth` as the independent issuer
  it already is.

### 7.2 jig

- **The public address in the default mapping** (2.2): the Host
  allow-list's extra name, and `Secure` decided by the exchange's
  `Host`.
- **The provider** in the clients' component, since jig owns sign-ins
  and credentials (jig's `domain/people.md`, section 3): the clients'
  registrations in its configuration; `Authorize` from a signed-in
  client, on the default mapping as `POST <prefix>/v1/authorize`; the
  code's exchange and the claims as messages of the application's
  server, as `Mint` is; the signing key in its secrets' store. The
  routes and their addresses stay the application's (jig's
  `protocol/client.md`, 17.1: no provider route).
- **Nothing for Google:** jig's sign-in helper for OAuth stays where
  jig's `protocol/client.md`, section 15 puts it, sugar adopted once a
  second application needs it. temper's flow is its own.

### 7.3 temper

- **The configuration's members:** `application.clients.public` (2.2),
  `application.sign_in` with Google's registration and its members
  (section 3), and the provider's client for Forgejo (5.2).
- **Google's routes** (4.2), their endings' pages (4.3) and the
  member's binding record (section 3).
- **The provider's routes** (5.2), handing each to jig.
- **The client:** the browser's stock sign-in page with the configured
  ways (4.1), the `Authorize` place (5.3), and "Open Forgejo ↗" to
  Forgejo's route for the temper source when the forge signs in through
  temper.
- **The setup:** the `add-oauth` command printed for a managed Forgejo,
  run for an owned one (5.4).

### 7.4 The stages

1. **Google into temper:** 2.2, 2.3, section 3, section 4, and jig's
   public address. The deployment is usable by its members; Forgejo
   signs them in by its own passwords meanwhile.
2. **temper as Forgejo's provider:** section 5, skein's issuer and
   jig's provider. Forgejo's passwords are then retired by
   `ENABLE_INTERNAL_SIGNIN = false`.

## 8. Testing

- **The Google flow in temper's worlds** with `skein-fake-oauth` as
  Google: a member minted, each ending of 4.3, the binding refused for
  a second subject, and `then` kept. No browser.
- **The provider against a real Forgejo** in the process tier, on
  skein's throwaway Forgejo with the temper source added by its command
  line: a scripted client with a cookie jar follows Forgejo's sign-in
  through temper and back, and Forgejo's API then names the member's
  account. It proves the claims and Forgejo's settings, which no fake
  can.
- **The browser tier, for what only a browser proves:** jig's strict
  cookie absent from a navigation that started on another site and
  present on the page's own exchanges (5.3), and the binding cookie's
  return from Google's cross-site redirect (4.2).

## 9. Open questions

- **A deployment's own sign-in page.** Default: stock (4.1). Later,
  perhaps a name and a line in the configuration, or a template of the
  owner's, as Forgejo takes its `APP_NAME` and `APP_SLOGAN` and
  templates under its custom directory.
- **The signing algorithm: RS256.** Forgejo 17's OpenID Connect client
  (goth v1.82.0) reads the ID token's payload and checks its `aud`,
  `iss` and `exp`, with no fetch of the key set: it verifies no
  signature, so it accepts ES256, and any algorithm, alike. That is
  acceptable here, since Forgejo takes the token directly from temper's
  token endpoint, over TLS, authenticated by its secret by Basic
  (OpenID Connect Core, 3.1.3.7). So ES256 buys Forgejo nothing, and
  RS256 stays, for its own reasons: `skein_tls::sign_rs256` signs it;
  its signatures (PKCS#1 v1.5) are deterministic, so the worlds replay
  byte for byte, where `ring`'s ECDSA draws from a system randomness of
  its own and signs differently on every run; and skein-oauth's client,
  the issuer's peer in its world, verifies RS256 alone. `ring` signs
  RS256 with a key it is given but cannot make one: the key, 2048 bits,
  is made at the first start by the setup (`openssl genrsa`) and kept
  in jig's secrets' store. Rotation waits.
- **Single sign-out: none,** said on the signed-out page. Forgejo 17
  takes no signal from its provider: no back-channel, front-channel or
  session-management logout, and no RP-initiated one (it parses the
  discovery document's `end_session_endpoint` and never uses it); its
  own sign-out is local. A logout Forgejo would take from its provider
  waits for Forgejo to take one.
- **Members at runtime.** jig seeds roles on a first start only. A
  member added later signs in with no project; an owner's invitation,
  and roles granted from the client, wait for a design of their own.
  Default: the configuration's members, roles by the seed.
- **Owners as Forgejo's admins.** Forgejo's source can make admins of a
  group claim (`--admin-group`) and map groups onto teams
  (`--group-team-map`). Default: none; Forgejo's admins are made on
  Forgejo.
- **Google's Workspace domain** as a member rule (`hd`), for a
  deployment of one organization. Default: addresses alone.
- **The person's forge identity.** With temper as Forgejo's provider,
  temper knows each member's Forgejo account by construction: the way
  to a person's approval posted as the person (`../README.md`, section
  8) and to "the forge as people's sign-in provider", turned around
  (`../client/README.md`, section 11).
- **The terminal for a member who is not the owner:** a device code's
  flow. Default: the browser alone.

## 10. The mockups

`../client/mockups/sign-in.html` in the world Google sign-in
(`world-google-sign-in.js`): the stock sign-in page signed out;
arriving from Forgejo signed out; each of 4.3's endings; signed out.
Behind it, the chats page and the Forges page, whose Forgejo at
git.example.com opens signed in through temper. Each is marked "ahead"
while this draft is one.
