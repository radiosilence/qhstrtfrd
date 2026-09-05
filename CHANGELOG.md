# Changelog

## Unreleased

### Added

- The site: home, what's on, sport, find us, history and FAQ, rendered from one
  TOML content file by an Askama generator, styled with Tailwind v4 over a claret-and-brass
  palette taken from the pub itself.
- Structured data the pubco page never had — `BarOrPub` with real
  `openingHoursSpecification`, geo coordinates, amenities and `sameAs`; `Event`
  entries for anything in the diary; `WebSite` and a breadcrumb trail. Open Graph
  and Twitter cards with a per-page 1200×630 share image.
- A crest, drawn as SVG and rasterised into the full icon set.
- Nightly CI rebuild, so build-time event filtering keeps meaning what it says.

### Changed

- Cut copy that repeated itself on the same page, and meta descriptions that
  disagreed with the page about distances.
