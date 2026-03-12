pub struct CrateConfig {
    pub name: &'static str,
    pub features: &'static [(&'static str, &'static str)],
    pub test_threads_1: bool,
    /// Integration test binaries (files in tests/)
    pub integration_tests: &'static [&'static str],
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
    },
    CrateConfig {
        name: "chicken_settings",
        features: &[("default", "")],
        test_threads_1: false,
        integration_tests: &[],
    },
    CrateConfig {
        name: "chicken",
        features: &[("default", "")],
        test_threads_1: false,
        integration_tests: &[],
    },
];

pub fn find_crate(name: &str) -> Option<&'static CrateConfig> {
    CRATES.iter().find(|c| c.name == name)
}
