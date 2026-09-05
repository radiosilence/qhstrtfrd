//! Renders every page into dist/, publishes the assets they referenced, and
//! checks that every absolute reference resolves to something written.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use askama::Template;
use time::{Date, OffsetDateTime};

use queenshead::assets::Assets;
use queenshead::content::Site;
use queenshead::routes::{PAGES, url};
use queenshead::templates::{Context, Faq, FindUs, History, Index, Meta, NavItem, Sport, WhatsOn};

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).to_owned();
    let dist = root.join("dist");
    let build = root.join(".build");

    let site = Site::load(&root.join("src/content/site.toml"))?;
    let mut assets = Assets::load(&root.join("src/static"))?;

    // The stylesheet names the fonts, so it is rewritten before its own hash is
    // taken — otherwise the fonts are also fetched at an unhashed URL that is
    // cached just as hard.
    let fraunces = assets.href("fraunces.woff2");
    let instrument = assets.href("instrument-sans.woff2");
    let css = fs::read_to_string(build.join("style.css"))
        .context("reading .build/style.css — run the css task first")?
        .replace("/fraunces.woff2", &fraunces)
        .replace("/instrument-sans.woff2", &instrument);
    assets.derive("style.css", css.into_bytes());

    let manifest = rewrite_manifest(&assets, &root.join("src/static/manifest.json"))?;
    assets.derive("manifest.json", manifest.into_bytes());

    // Written rather than shipped: it must list exactly the pages that exist, and
    // that is known here and nowhere else.
    assets.derive("sitemap.xml", sitemap(&site.venue.origin).into_bytes());

    // UTC, not London: `today` only decides whether an event has passed, and an
    // hour either side of midnight is corrected by the nightly rebuild.
    let today: Date = OffsetDateTime::now_utc().date();
    let upcoming = site.upcoming(today)?;

    let ctx = Context { assets: &assets };
    let mut written = BTreeSet::new();

    for page in PAGES {
        let path = url(page.slug);
        let nav: Vec<NavItem> = PAGES
            .iter()
            .filter_map(|p| {
                p.nav.map(|label| NavItem {
                    href: url(p.slug),
                    label,
                    current: p.slug == page.slug,
                })
            })
            .collect();

        let meta = meta_for(&site, page.slug, &path, &assets);

        let html = match page.slug {
            "" => Index {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
            }
            .render()?,
            "whats-on" => WhatsOn {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
                upcoming: &upcoming,
            }
            .render()?,
            "sport" => Sport {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
            }
            .render()?,
            "find-us" => FindUs {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
            }
            .render()?,
            "history" => History {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
            }
            .render()?,
            "faq" => Faq {
                ctx: &ctx,
                site: &site,
                meta,
                nav: &nav,
            }
            .render()?,
            other => anyhow::bail!("no template for slug `{other}`"),
        };

        let file = dist.join(path.trim_start_matches('/')).join("index.html");
        fs::create_dir_all(file.parent().context("page has no parent directory")?)?;
        fs::write(&file, html)?;
        written.insert(file);
    }

    let published = assets.publish(&dist)?;

    let mut current = written.clone();
    current.extend(published.iter().cloned());
    prune(&dist, &current)?;
    check_references(&dist, &written, &current)?;

    println!(
        "Generated {} pages and {} assets in {}",
        written.len(),
        published.len(),
        dist.display()
    );
    Ok(())
}

/// Per-page title, description and share image.
///
/// Here rather than in `routes.rs` because it reads the content: the homepage's
/// description is the venue summary, and duplicating that into a route table is
/// how the two drift apart.
fn meta_for(site: &Site, slug: &str, path: &str, assets: &Assets) -> Meta {
    let v = &site.venue;
    let canonical = format!("{}{}", v.origin, path);
    // Hashed like any other asset: an absolute URL in a meta tag is fetched by
    // crawlers, not resolved by the reference checker, so the hash is what stops
    // a stale share card living in a scraper's cache forever.
    let image = format!(
        "{}{}",
        v.origin,
        assets.href(&format!(
            "og-{}.jpg",
            if slug.is_empty() { "home" } else { slug }
        ))
    );

    let (title, heading, description, image_alt) = match slug {
        "whats-on" => (
            format!("What's On — {}, {}", v.name, v.locality),
            "What's On".to_owned(),
            format!(
                "Live music, salsa Thursdays, darts and pool at {} on West Ham Lane, Stratford E15.",
                v.name
            ),
            "Live music at the Queen's Head, Stratford".to_owned(),
        ),
        "sport" => (
            format!("Sport — {}, {}", v.name, v.locality),
            "Sport".to_owned(),
            format!(
                "Sky Sports and TNT Sports on HD screens at {}, Stratford E15 — Premier League, Champions League, rugby, racing and boxing, a walk from the London Stadium.",
                v.name
            ),
            "A full room watching the football at the Queen's Head, Stratford".to_owned(),
        ),
        "history" => (
            format!("History — {}, West Ham Lane, {}", v.name, v.locality),
            "Older than the skyline".to_owned(),
            format!(
                "Photographed on West Ham Lane in 1985, briefly traded as Le Pub, refitted in 2023. What can actually be shown about the history of {}, Stratford E15.",
                v.name
            ),
            "The bar at the Queen's Head, Stratford".to_owned(),
        ),
        "faq" => (
            format!("FAQ — {}, {} {}", v.name, v.locality, v.postcode),
            "Straight answers".to_owned(),
            format!(
                "Do they show West Ham? Is it dog friendly? Is there food? Straight answers about {}, {} {} — including the ones where the answer is no.",
                v.name, v.locality, v.postcode
            ),
            "Pump clips along the bar at the Queen's Head".to_owned(),
        ),
        "find-us" => (
            format!("Find Us — {}, {} {}", v.name, v.locality, v.postcode),
            "Find Us".to_owned(),
            format!(
                "{}, {}, {} {}. Ten minutes from Stratford station, 25 from the London Stadium. Open from 11am every day.",
                v.street, v.locality, v.city, v.postcode
            ),
            "The bar at the Queen's Head, Stratford".to_owned(),
        ),
        _ => (
            format!(
                "{} — {} pub on West Ham Lane, {} {}",
                v.name, v.locality, v.city, v.postcode
            ),
            v.tagline.clone(),
            v.summary.clone(),
            "A full house at the Queen's Head, Stratford".to_owned(),
        ),
    };

    Meta {
        title,
        heading,
        description,
        canonical,
        image,
        image_alt,
    }
}

/// The manifest declares icons the browser fetches without any page linking them,
/// so its own references are rewritten and thereby recorded as used.
fn rewrite_manifest(assets: &Assets, path: &Path) -> Result<String> {
    let mut out = fs::read_to_string(path)?;
    for name in ["icon-192.png", "icon-512.png", "icon-maskable.png"] {
        out = out.replace(&format!("/{name}"), &assets.href(name));
    }
    Ok(out)
}

fn sitemap(origin: &str) -> String {
    let urls: String = PAGES
        .iter()
        .map(|p| format!("  <url><loc>{origin}{}</loc></url>\n", url(p.slug)))
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n{urls}</urlset>\n"
    )
}

/// dist/ is written entirely here, so anything else in it is from an earlier
/// build: a removed page's directory, or the previous hash of a changed file.
fn prune(dist: &Path, keep: &BTreeSet<PathBuf>) -> Result<()> {
    for entry in walkdir::WalkDir::new(dist)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && !keep.contains(entry.path()) {
            fs::remove_file(entry.path())?;
        }
    }

    // Depth-first, so a directory is judged only once its children are gone.
    for entry in walkdir::WalkDir::new(dist)
        .contents_first(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_dir()
            && entry.path() != dist
            && fs::read_dir(entry.path())?.next().is_none()
        {
            fs::remove_dir(entry.path())?;
        }
    }

    Ok(())
}

/// A link that misses is a routing bug; an `src` that misses is an asset written
/// by hand rather than through `asset()`. Both would ship as a 404 nobody clicks.
fn check_references(
    dist: &Path,
    pages: &BTreeSet<PathBuf>,
    current: &BTreeSet<PathBuf>,
) -> Result<()> {
    let mut broken = BTreeSet::new();

    for page in pages {
        let html = fs::read_to_string(page)?;

        for attr in ["href=\"", "src=\""] {
            for part in html.split(attr).skip(1) {
                let Some(value) = part.split('"').next() else {
                    continue;
                };
                if !value.starts_with('/') {
                    continue;
                }

                let path = value.split(['?', '#']).next().unwrap_or(value);
                let target = dist.join(path.trim_start_matches('/'));

                if !current.contains(&target) && !current.contains(&target.join("index.html")) {
                    broken.insert(format!("{} -> {value}", page.display()));
                }
            }
        }
    }

    if !broken.is_empty() {
        anyhow::bail!(
            "references with nothing behind them:\n  {}",
            broken.into_iter().collect::<Vec<_>>().join("\n  ")
        );
    }

    println!("Checked links and assets across {} pages", pages.len());
    Ok(())
}
