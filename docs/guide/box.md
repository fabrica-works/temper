# Start a local temper box

This walk starts from a checkout of temper. Everything is on loopback.
The example's rule covers acme/widget alone. Its charters are short texts
to edit in `owned/charters/` before asking for work.

## Step 1. Install the three programs

Install temper, Forgejo **17.0.0**, and Forgejo runner **13.2.0** at the
paths below. temper does not download them. You also need `/usr/bin/git`,
`sha256sum`, and free loopback ports 3000, 7100 and 7101. Run this from
the temper checkout; keep this terminal for steps 2 to 5.

```sh
TEMPER=/usr/local/bin/temper
FORGEJO=/opt/forgejo/forgejo
RUNNER=/opt/forgejo/forgejo-runner
EXAMPLES="$PWD/docs/examples"
```

## Step 2. Copy the example into a private directory

Use a fresh directory. For a second box, change the three addresses
together in `temper.json`, including their repetitions in the forge,
landing rule and listeners.

```sh
umask 077
mkdir -m 700 temper-box
cd temper-box
cp "$EXAMPLES/owned.json" temper.json
cp -R "$EXAMPLES/owned" .
sed -i -e "s|/home/me/temper|$PWD|g" \
  -e "s|/opt/forgejo/forgejo-runner|$RUNNER|g" \
  -e "s|/opt/forgejo/forgejo|$FORGEJO|g" \
  -e "s|/usr/local/bin/temper|$TEMPER|g" temper.json owned/agent.json
```

## Step 3. Make the worker's secret and digest

The digest hashes the secret without its final newline. Keep both files
private; the worker receives the secret and the engine receives its digest.

```sh
mkdir -m 700 -p secrets
secret=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
printf '%s\n' "$secret" > secrets/worker.secret
printf %s "$secret" | sha256sum | cut -d' ' -f1 > secrets/worker.digest
unset secret
```

## Step 4. LLM sign-in (off until JP13)

Off until JP13 (fallback D): the owned example's Codex adapter needs a signed-in account and refuses deployment API keys; configured account files are no longer read.

## Step 5. Start the box and keep its terminal

The box resolves the provider's name at its start. It prepares Forgejo
and starts its runner; temper takes over the admin's token the
preparation left, makes its bot, and prints a sign-in link. Keep this
terminal open; copy that link for step 7.

```sh
"$TEMPER" box --config "$PWD" --data "$PWD" --cache "$PWD/cache"
```

## Step 6. Make acme/widget on Forgejo's web

Open `http://127.0.0.1:3000/`. Sign in as `ada` with the password in
`forge-1/person_password`, then change it when Forgejo asks. Its API
refuses the person until this change. In the `acme` organization, create
the private repository `widget`, initialized with a README on `main`,
and enable Actions in its settings.

Commit `check.sh` on `main`, with this text:

```text
test -s README.md
```

Commit `.forgejo/workflows/check.yaml` on `main`, with this text. It uses
the owned runner's `temper-host` label and no `uses:` steps, and runs on
pushes to `temper/**`, where temper pushes its work.

```yaml
on: { push: { branches: ['temper/**'] } }
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: |
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone "${{ github.server_url }}/${{ github.repository }}.git" .
          git checkout "${{ github.sha }}" && sh check.sh
```

The runner's jobs run as your user and can read temper's files: run only your own code,
never a stranger's pull request.

## Step 7. Adopt the repository in temper

Open a second terminal in the same `temper-box` directory. Set `TEMPER`
as in step 1, paste the printed link when prompted, and run the client:

```sh
TEMPER=/usr/local/bin/temper
printf 'Paste the sign-in link, then press Enter: '
read -r LINK
"$TEMPER" open client "$LINK"
```

Choose the numbered **Adopt a repository** control. Set **Owner** to
`acme`, **Name** to `widget`, **Landing branch** to `main`, and **Checks**
to `1`. Keep **It runs CI** checked. Choose **Adopt** and wait for
`Adopted acme/widget on forge 1`; adopting makes temper's bot a
collaborator on the repository and adds its webhook. Type `q` to leave
the client.

## Step 8. Ask for work (off until JP13)

Off until JP13 (fallback D): live work needs the signed-in account deferred in step 4.

Stop the box with Ctrl-C in its terminal. Start it again with the same
command; use its new link to reopen the client. Your
backups are yours: copy the forge directory and `state` with the box stopped.
