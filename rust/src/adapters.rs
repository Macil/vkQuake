pub mod common;

// Allow the actual implementation to be compiled in tests to make sure it compiles,
// and because it must be enabled in tests for rust-analyzer to see it at all by default.
#[cfg_attr(test, allow(dead_code))]
mod actual;

#[cfg(not(test))]
pub use actual::*;

// Don't compile mock at all in non-test builds.
#[cfg(test)]
mod mock;

#[cfg(test)]
pub use mock::*;
// TODO set things up so rust-analyzer sees actual by default instead of mock.
// https://github.com/rust-lang/rust-analyzer/issues/7225
// Maybe require tests to be run with a feature flag controlling mocks.
