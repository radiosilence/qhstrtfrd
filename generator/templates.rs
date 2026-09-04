//! Askama binds a template to a struct at compile time, so a field a template
//! names but the struct lacks is a compile error rather than a blank in the
//! output. That is the type check; there is no runtime view guard to maintain.

use askama::Template;

use crate::assets::Assets;
use crate::content::{Site, Upcoming};

pub struct Context<'a> {
    pub assets: &'a Assets,
}

/// A nav entry, resolved once per page so the template does no comparing.
pub struct NavItem {
    pub href: String,
    pub label: &'static str,
    pub current: bool,
}

/// Everything the `<head>` needs, gathered rather than passed as six arguments.
pub struct Meta {
    /// The full `<title>`, including the venue suffix.
    pub title: String,
    /// `og:title` and the `<h1>`-adjacent heading — no suffix.
    pub heading: String,
    pub description: String,
    pub canonical: String,
    /// Absolute, because og:image will not take a relative URL.
    pub image: String,
    pub image_alt: String,
}

/// Declares a page template.
///
/// `{% extends %}` requires a child to carry every field the layout names, so the
/// shared ones are listed once here and each page adds only what it uses.
macro_rules! page {
    ($name:ident, $path:literal $(, $field:ident: $ty:ty)* $(,)?) => {
        #[derive(Template)]
        #[template(path = $path)]
        pub struct $name<'a> {
            pub ctx: &'a Context<'a>,
            pub site: &'a Site,
            pub meta: Meta,
            pub nav: &'a [NavItem],
            $(pub $field: $ty,)*
        }

        impl $name<'_> {
            fn asset(&self, name: &str) -> String {
                self.ctx.assets.href(name)
            }

            /// An absolute URL for a path that is already rooted.
            fn absolute(&self, path: &str) -> String {
                format!("{}{path}", self.site.venue.origin)
            }
        }
    };
}

page!(Index, "index.html", upcoming: &'a [Upcoming<'a>]);
page!(WhatsOn, "whats-on.html", upcoming: &'a [Upcoming<'a>]);
page!(Sport, "sport.html");
page!(FindUs, "find-us.html");
