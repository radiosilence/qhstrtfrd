//! Pages are written as directory indexes (`dist/whats-on/index.html`), which
//! nano-web resolves for both `/whats-on` and `/whats-on/`. Internal links carry
//! no extension.

pub struct Page {
    pub slug: &'static str,
    pub title: &'static str,
    /// Shown in the nav. The homepage is reached by the mark, not by a nav item.
    pub nav: Option<&'static str>,
}

pub const PAGES: &[Page] = &[
    Page {
        slug: "",
        title: "",
        nav: None,
    },
    Page {
        slug: "whats-on",
        title: "What's On",
        nav: Some("What's On"),
    },
    Page {
        slug: "sport",
        title: "Sport",
        nav: Some("Sport"),
    },
    Page {
        slug: "find-us",
        title: "Find Us",
        nav: Some("Find Us"),
    },
];

pub fn url(slug: &str) -> String {
    if slug.is_empty() {
        "/".to_owned()
    } else {
        format!("/{slug}")
    }
}
