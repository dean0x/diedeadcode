//! Next.js framework detector.

use crate::plugins::registry::FrameworkDetector;

/// Detector for Next.js applications.
pub struct NextJsDetector;

impl NextJsDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NextJsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameworkDetector for NextJsDetector {
    fn name(&self) -> &'static str {
        "nextjs"
    }

    fn get_entry_patterns(&self) -> Vec<String> {
        vec![
            // Pages router
            "pages/**/*.tsx".to_string(),
            "pages/**/*.ts".to_string(),
            "pages/**/*.jsx".to_string(),
            "pages/**/*.js".to_string(),
            "src/pages/**/*.tsx".to_string(),
            "src/pages/**/*.ts".to_string(),
            "src/pages/**/*.jsx".to_string(),
            "src/pages/**/*.js".to_string(),
            // App router
            "app/**/page.tsx".to_string(),
            "app/**/page.ts".to_string(),
            "app/**/page.jsx".to_string(),
            "app/**/page.js".to_string(),
            "app/**/layout.tsx".to_string(),
            "app/**/layout.ts".to_string(),
            "app/**/layout.jsx".to_string(),
            "app/**/layout.js".to_string(),
            "app/**/loading.tsx".to_string(),
            "app/**/error.tsx".to_string(),
            "app/**/not-found.tsx".to_string(),
            "app/**/route.ts".to_string(),
            "src/app/**/page.tsx".to_string(),
            "src/app/**/page.ts".to_string(),
            "src/app/**/layout.tsx".to_string(),
            "src/app/**/route.ts".to_string(),
            // API routes
            "pages/api/**/*.ts".to_string(),
            "pages/api/**/*.js".to_string(),
            "src/pages/api/**/*.ts".to_string(),
            "app/api/**/route.ts".to_string(),
            "src/app/api/**/route.ts".to_string(),
            // Config files
            "next.config.js".to_string(),
            "next.config.mjs".to_string(),
            "next.config.ts".to_string(),
            "middleware.ts".to_string(),
            "middleware.js".to_string(),
            "src/middleware.ts".to_string(),
        ]
    }

    fn get_special_exports(&self) -> Vec<&'static str> {
        vec![
            // Pages router data fetching
            "getServerSideProps",
            "getStaticProps",
            "getStaticPaths",
            "getInitialProps",
            // App router
            "generateMetadata",
            "generateStaticParams",
            "generateViewport",
            "generateImageMetadata",
            // API routes
            "GET",
            "POST",
            "PUT",
            "DELETE",
            "PATCH",
            "HEAD",
            "OPTIONS",
            // Config
            "config",
            "runtime",
            "preferredRegion",
            "revalidate",
            "dynamic",
            "dynamicParams",
            "fetchCache",
            // Middleware
            "middleware",
            "matcher",
            // Default export (page component)
            "default",
        ]
    }

    fn detect_from_dependencies(&self, deps: &[String]) -> bool {
        deps.iter().any(|d| d == "next")
    }
}
