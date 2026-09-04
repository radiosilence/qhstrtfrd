//! `src/content/site.toml`, parsed once into the shape the templates want.
//!
//! Serde is the schema: a key the file misspells fails the build rather than
//! rendering an empty section, which is the failure mode a CMS-shaped pub site
//! always ends up with. `deny_unknown_fields` is what makes that true in the
//! other direction too — a `[[event]]` where `[[events]]` was meant is caught
//! here rather than silently dropping the list.

use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::Path;
use time::{Date, Month, Weekday, macros::format_description};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Site {
    pub venue: Venue,
    pub about: About,
    pub matchday: Matchday,
    pub hours: Vec<Hours>,
    pub bar: Bar,
    pub sport: Sport,
    pub whatson: WhatsOnIntro,
    pub regulars: Vec<Regular>,
    /// Absent means none. A pub with nothing booked is a real state, and the
    /// page says so rather than inventing a gig to fill the space.
    #[serde(default)]
    pub events: Vec<Event>,
    pub functions: Functions,
    pub facilities: Vec<Facility>,
    pub travel: Travel,
    pub access: Access,
    pub history: History,
    pub faq: Vec<Faq>,
    /// Never rendered. It exists so that provenance lives beside the claims
    /// rather than in a comment nobody updates, and so that `deny_unknown_fields`
    /// does not reject the block that records where all this came from.
    #[allow(dead_code)]
    pub sources: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Venue {
    pub name: String,
    pub short_name: String,
    pub initials: String,
    pub tagline: String,
    pub summary: String,
    pub street: String,
    pub street_alt: String,
    pub locality: String,
    pub city: String,
    pub region: String,
    pub postcode: String,
    pub country: String,
    pub phone_display: String,
    pub phone_link: String,
    pub latitude: f64,
    pub longitude: f64,
    pub facebook: String,
    pub instagram: String,
    pub origin: String,
}

impl Venue {
    /// One line, for the footer and the `address` microdata.
    pub fn address_line(&self) -> String {
        format!(
            "{}, {}, {} {}",
            self.street, self.locality, self.city, self.postcode
        )
    }

    /// Google Maps takes a query string; a postcode plus the street is more
    /// reliable than coordinates, which drop a pin in the middle of the road.
    pub fn maps_url(&self) -> String {
        format!(
            "https://www.google.com/maps/search/?api=1&query={}",
            format_args!(
                "{}%2C+{}",
                self.street.replace(' ', "+"),
                self.postcode.replace(' ', "+")
            )
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct About {
    pub heading: String,
    pub body: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matchday {
    pub heading: String,
    pub lede: String,
    pub body: String,
    pub walk_label: String,
    pub walk_detail: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hours {
    pub day: String,
    pub open: String,
    pub close: String,
}

impl Hours {
    /// Schema.org wants `Mo`, `Tu`, … — derived so the two cannot disagree.
    pub fn schema_day(&self) -> &str {
        &self.day[..2]
    }

    /// `11:00` → `11am`, `23:00` → `11pm`. Pub hours are never on the half hour
    /// here, but the minutes are kept when they are not zero.
    pub fn open_display(&self) -> String {
        friendly_time(&self.open)
    }

    pub fn close_display(&self) -> String {
        friendly_time(&self.close)
    }
}

/// `23:00` → `11pm`; `22:30` → `10.30pm`.
fn friendly_time(hhmm: &str) -> String {
    let Some((h, m)) = hhmm.split_once(':') else {
        return hhmm.to_owned();
    };
    let (Ok(h), Ok(m)) = (h.parse::<u8>(), m.parse::<u8>()) else {
        return hhmm.to_owned();
    };
    let suffix = if h < 12 { "am" } else { "pm" };
    let hour = match h % 12 {
        0 => 12,
        other => other,
    };
    if m == 0 {
        format!("{hour}{suffix}")
    } else {
        format!("{hour}.{m:02}{suffix}")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    pub heading: String,
    pub note: String,
    pub taps: Vec<Tap>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tap {
    pub name: String,
    pub style: String,
    pub note: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sport {
    pub heading: String,
    pub lede: String,
    pub note: String,
    pub competitions: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Regular {
    pub name: String,
    pub when: String,
    pub blurb: String,
}

/// The standing "what we put on" copy. Deliberately carries no times: nothing
/// published states them, so nothing here claims them.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhatsOnIntro {
    pub heading: String,
    pub lede: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Functions {
    pub heading: String,
    pub body: String,
}

/// One question and answer, rendered as copy and as FAQPage structured data.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faq {
    pub q: String,
    pub a: String,
}

/// The timeline. `approx` is the honest bit: a moment we can only place loosely
/// says so in the markup rather than being rounded to a year that reads as fact.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    pub heading: String,
    pub lede: String,
    pub note: String,
    pub moments: Vec<Moment>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moment {
    pub year: String,
    pub approx: bool,
    pub body: String,
}

/// Where a claim on this site came from.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub url: String,
    #[allow(dead_code)]
    pub covers: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub date: String,
    pub name: String,
    pub time: String,
    pub blurb: String,
}

/// An event that has not happened yet, with its date already spelled out.
///
/// The weekday is computed rather than written down: a hand-typed "Friday" that
/// disagrees with the date is the one error nobody proofreads for.
pub struct Upcoming<'a> {
    pub event: &'a Event,
    /// `Sat 12 Sep`
    pub label: String,
    /// `2026-09-12` — the `datetime` attribute and the JSON-LD `startDate`.
    pub iso: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facility {
    pub name: String,
    pub detail: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Travel {
    pub heading: String,
    pub lede: String,
    pub routes: Vec<TravelRoute>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TravelRoute {
    pub mode: String,
    pub detail: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Access {
    pub heading: String,
    pub body: String,
}

impl Site {
    pub fn load(path: &Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
    }

    /// Events from `today` onwards, soonest first.
    ///
    /// Filtering at build time is what keeps the page honest without anyone
    /// pruning the file: a build that has not run for a month shows a short list
    /// rather than a list of things that already happened. The nightly rebuild in
    /// CI is what makes "today" mean today.
    pub fn upcoming(&self, today: Date) -> Result<Vec<Upcoming<'_>>> {
        let fmt = format_description!("[year]-[month]-[day]");
        let mut out = Vec::new();

        for event in &self.events {
            let date = Date::parse(&event.date, &fmt)
                .with_context(|| format!("event `{}` has an unparseable date", event.name))?;
            if date < today {
                continue;
            }
            out.push((
                date,
                Upcoming {
                    event,
                    label: format!(
                        "{} {} {}",
                        short_weekday(date.weekday()),
                        date.day(),
                        short_month(date.month())
                    ),
                    iso: event.date.clone(),
                },
            ));
        }

        out.sort_by_key(|(date, _)| *date);
        Ok(out.into_iter().map(|(_, u)| u).collect())
    }
}

fn short_weekday(day: Weekday) -> &'static str {
    match day {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

fn short_month(month: Month) -> &'static str {
    match month {
        Month::January => "Jan",
        Month::February => "Feb",
        Month::March => "Mar",
        Month::April => "Apr",
        Month::May => "May",
        Month::June => "Jun",
        Month::July => "Jul",
        Month::August => "Aug",
        Month::September => "Sep",
        Month::October => "Oct",
        Month::November => "Nov",
        Month::December => "Dec",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn twelve_hour_clock_drops_zero_minutes() {
        assert_eq!(friendly_time("11:00"), "11am");
        assert_eq!(friendly_time("23:00"), "11pm");
        assert_eq!(friendly_time("22:30"), "10.30pm");
        assert_eq!(friendly_time("00:00"), "12am");
        assert_eq!(friendly_time("12:00"), "12pm");
    }

    /// The file that ships must parse, and its events must have real dates —
    /// otherwise the first anyone knows is a broken deploy.
    #[test]
    fn shipped_content_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/site.toml");
        let site = Site::load(&path).expect("site.toml parses");
        assert_eq!(site.hours.len(), 7, "a week has seven days");
        site.upcoming(date!(2000 - 01 - 01))
            .expect("every event date parses");
        assert!(
            !site.sources.is_empty(),
            "content must record where its claims came from"
        );
    }
}
