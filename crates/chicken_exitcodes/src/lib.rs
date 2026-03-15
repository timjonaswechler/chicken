use {bevy::app::AppExit, std::num::NonZeroU8};

/// Process exit codes for the entire Chicken ecosystem.
///
/// Rules:
/// - 0 is reserved for success
/// - 1..=255 are error codes (NonZeroU8)
/// - Never recycle codes; old ones remain reserved
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ExitCode {
    /// Unclassified fatal error (fallback).
    GenericFatal = 1,
    ServerStartupFailed = 2,
    ServerShutdownFailed = 3,
    ServerGoingPublicFailed = 4,
    ServerGoingPrivateFailed = 5,
}

impl From<ExitCode> for AppExit {
    fn from(code: ExitCode) -> Self {
        AppExit::Error(code.nonzero())
    }
}

impl ExitCode {
    /// The numeric exit code (1..=255).
    pub const fn code(self) -> u8 {
        self as u8
    }

    pub fn nonzero(self) -> NonZeroU8 {
        // safe because all variants are != 0
        NonZeroU8::new(self.code()).expect("ExitCode must be non-zero")
    }

    pub const fn description(self) -> &'static str {
        match self {
            ExitCode::GenericFatal => "Generic fatal error",
            ExitCode::ServerStartupFailed => {
                "Server startup process failed. Read logs for details."
            }
            ExitCode::ServerShutdownFailed => "Server shutdown failed. Read logs for details.",
            ExitCode::ServerGoingPublicFailed => {
                "Server going public failed. Read logs for details."
            }
            ExitCode::ServerGoingPrivateFailed => {
                "Server going private failed. Read logs for details."
            }
        }
    }
}
