pub mod client;
pub mod prompts;
pub mod providers;

pub use client::AIClient;

#[cfg(test)]
mod tests;