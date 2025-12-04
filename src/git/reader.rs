use crate::error::Result;
use git2::Repository;

pub struct GitReader {
    repo: Repository,
}

impl GitReader {
    pub fn new(path: Option<&str>) -> Result<Self> {
        let repo = if let Some(p) = path {
            Repository::open(p)?
        } else {
            Repository::open_from_env()?
        };

        Ok(Self { repo })
    }

    pub fn read_commit(&self, commit_ref: &str) -> Result<()> {
        tracing::info!("Reading commit: {}", commit_ref);
        // TODO: Implement commit reading
        Ok(())
    }

    pub fn read_staged(&self) -> Result<()> {
        tracing::info!("Reading staged changes");
        // TODO: Implement staged reading
        Ok(())
    }

    pub fn read_commit_range(&self, range: &str) -> Result<()> {
        tracing::info!("Reading commit range: {}", range);
        // TODO: Implement commit range reading
        Ok(())
    }
}
