# The Queen's Head, Stratford

Static site for [queensheadstratford.london](https://queensheadstratford.london) —
a pub on West Ham Lane, E15. The browser gets HTML, CSS, two variable fonts and
about thirty lines of JavaScript.

## Why it exists

The pub's presence online was a Craft Union template: a stock "Welcome to X, your
local pub based in Y" line, an hours table, an events widget reading *no events
available at the moment*, and a `BarOrPub` JSON-LD block with a malformed
`openingHours` string and no geo data. This replaces it with something that says
what the place actually is and that a search engine can read properly.

## Stack

- [Task](https://taskfile.dev) orchestrates; steps declare `sources`/`generates`
  so nothing re-runs without cause
- [Askama](https://askama.rs) renders `src/templates/*.html`
- Content is one TOML file parsed through serde with `deny_unknown_fields`
- [TailwindCSS](https://tailwindcss.com) v4, with Fraunces and Instrument Sans as
  variable woff2 (95KB for both, latin subset)
- Everything the build needs is pinned in [mise](https://mise.jdx.dev).
  [nano-web](https://github.com/radiosilence/nano-web) serves `dist/` for
  `task dev` and is yours to install
- Deployment: container image → k3s on a Hetzner VPS, behind Traefik terminating
  Let's Encrypt over DNS-01. The cluster and this deployment are one Pulumi
  program in [jaritanet](https://github.com/radiosilence/jaritanet)

## Editing the pub

`src/content/site.toml` is the whole site's content — hours, taps, events, travel,
the copy itself. Nothing in it is repeated in a template, and serde rejects a key
it does not know rather than rendering a blank section.

Events carry an ISO date and stop rendering once it has passed, so a list nobody
prunes goes short rather than wrong. That only works if a build runs, which is why
CI rebuilds nightly as well as on push.

## Decisions worth knowing

- Askama binds a template to a struct at compile time, so a field a template names
  but the struct lacks fails `cargo build`. That is the type check; there is no
  runtime view guard to maintain.
- Base element rules in `app.css` must stay inside `@layer base`. Unlayered CSS
  beats every `@layer` regardless of specificity. `@font-face` is the exception and
  sits outside deliberately — some engines ignore it inside a layer.
- Every asset reaches a page through `asset('img/crowd.jpg')`, which publishes it
  under a content hash and records the reference. Writing the path by hand fails
  the build, because the reference check resolves `href` and `src` against what was
  written. The share images are hashed too: an absolute URL in a `meta` tag is
  fetched by crawlers rather than checked by the build, and the hash is what stops a
  stale card living in a scraper's cache.
- The stylesheet and the manifest are derived rather than copied: both name other
  assets, so they are rewritten before their own hash is taken.
- `dist/` is written entirely by `generate`, which is what lets it delete anything
  that is not a page or a referenced asset.
- Pages are written as directory indexes (`dist/find-us/index.html`) because
  nano-web resolves `/find-us` to `/find-us/`. Don't put extensions on internal links.
- Adding a page means a template, a `page!` entry in `generator/templates.rs`, a
  route in `generator/routes.rs` and an arm in `meta_for`. The sitemap follows.
- There is one script, and it only ever *adds*: the hours table and the status pill
  are both correct with JavaScript off, and it upgrades the pill to "Open now" and
  marks today's row. No native element answers "is it open right now", which is the
  one question a pub website exists to answer.
- The icons are generated from `src/static/favicon.svg` with `qlmanage` and `sips`,
  both of which ship with macOS. They are committed, not built — the crest changes
  about as often as the pub does.

## Commands

```bash
task dev     # Rebuild on change, serve on :3000 with nano-web
task build   # css into .build/, then generate the site into dist/
task check   # clippy, rustfmt and tests
task ci      # What CI runs: check, then build
task clean   # Drop dist/, .build/, target/ and Task's checksum cache
```

`task --list` is authoritative; prefer it over this list going stale.
