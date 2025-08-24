pub mod agent;
pub mod config;
pub mod linker;

// Re-export commonly used types
pub use agent::{Agent, AgentSource};
pub use config::AgentsConfig;
