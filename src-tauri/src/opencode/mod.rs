pub mod executor;
pub mod session_parser;

pub use executor::{
    create_session, run_opencode_task, OpenCodeConfig, OpenCodeError, OpenCodeOutput,
    SessionManager,
};

#[cfg(test)]
pub use executor::run_mock_opencode_task;
