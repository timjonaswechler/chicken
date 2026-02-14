use std::num::NonZeroU8;

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

    /// Configuration is invalid or missing.
    ConfigInvalid = 10,

    /// Failed to bind network port (e.g. port in use / insufficient permissions).
    BindPortFailed = 20,
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
            ExitCode::ConfigInvalid => "Invalid configuration",
            ExitCode::BindPortFailed => "Failed to bind network port",
        }
    }
}
