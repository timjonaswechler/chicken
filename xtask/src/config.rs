pub struct CrateConfig {
    pub name: &'static str,
    pub features: &'static [(&'static str, &'static str)],
    pub test_threads_1: bool,
    /// Integration test binaries (files in tests/)
    pub integration_tests: &'static [&'static str],
    /// Include in CI test run
    pub ci: bool,
}

pub const CRATES: &[CrateConfig] = &[
    CrateConfig {
        name: "chicken_states",
        features: &[("hosted", "hosted"), ("headless", "headless")],
        test_threads_1: false,
        integration_tests: &[
            "app",
            "menu_wiki",
            "menu_singleplayer",
            "menu_settings",
            "menu_multiplayer",
        ],
        ci: true,
    },
    CrateConfig {
        name: "chicken_network",
        features: &[
            ("default", ""),
            ("server", "server"),
            ("client", "client"),
            ("all", "server,client"),
        ],
        test_threads_1: true,
        integration_tests: &[],
        // excluded from CI: 34 pre-existing compile errors, see issue #12
        ci: false,
    },
    CrateConfig {
        name: "chicken_protocols",
        features: &[
            ("default", ""),
            ("server", "server"),
            ("client", "client"),
            ("all", "server,client"),
        ],
        test_threads_1: false,
        integration_tests: &[],
        ci: true,
    },
    CrateConfig {
        name: "chicken_settings",
        features: &[("default", "")],
        test_threads_1: false,
        integration_tests: &[],
        ci: true,
    },
    CrateConfig {
        name: "chicken",
        features: &[("default", "")],
        test_threads_1: false,
        integration_tests: &[],
        ci: true,
    },
];

pub fn find_crate(name: &str) -> Option<&'static CrateConfig> {
    CRATES.iter().find(|c| c.name == name)
}
