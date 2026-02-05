//! ktme - Knowledge Transfer Me
//!
//! Rust-based CLI tool and MCP server for automated documentation generation.

pub mod ai;
pub mod cli;
pub mod config;
pub mod doc;
pub mod error;
pub mod git;
pub mod mcp;
pub mod service_detector;
pub mod storage;

pub use error::{KtmeError, Result};

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum InitMode {
    Fresh,
    Scan,
    Validate,
    Enhance,
    Sync,
    Research,
}
