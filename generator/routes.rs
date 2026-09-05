//! Pages are written as directory indexes (`dist/whats-on/index.html`), which
//! nano-web resolves for both `/whats-on` and `/whats-on/`. Internal links carry
//! no extension.

pub struct Page {
    pub slug: &'static str,
    /// Shown in the nav. The homepage is reached by the mark, not by a nav item.
    pub nav: Option<&'static str>,
}

pub const PAGES: &[Page] = &[
    Page {
        slug: "",
        nav: None,
    },
    Page {
        slug: "whats-on",
        nav: Some("What's On"),
    },
    Page {
        slug: "sport",
        nav: Some("Sport"),
    },
    Page {
        slug: "find-us",
        nav: Some("Find Us"),
    },
    Page {
        slug: "history",
        nav: Some("History"),
    },
    Page {
        slug: "faq",
        nav: Some("FAQ"),
    },
];

pub fn url(slug: &str) -> String {
    if slug.is_empty() {
        "/".to_owned()
    } else {
        format!("/{slug}")
    }
}
