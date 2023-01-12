use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use console::style;
use git2::Repository;
use tracing::info;

use crate::config::Config;
use crate::console::MultiSpinner;
use crate::git::{CloneOptions, CloneRepository};
use crate::path::Path;
use crate::root::Root;
use crate::url::Url;

#[derive(Debug, Parser)]
pub struct Cmd {
    /// URL or pattern of the repository to clone.
    repo: Vec<String>,

    /// Clones their submodules recursively.
    #[clap(short, long)]
    recursive: bool,

    /// Change directory after cloned a repository (Shell extension required).
    #[clap(long)]
    cd: bool,

    /// Opens the directory after cloned a repository.
    #[clap(long)]
    open: Option<String>,
}

impl Cmd {
    pub async fn run(self) -> Result<()> {
        let root = Arc::new(Root::find()?);
        let config = Arc::new(Config::load_from(&root)?);

        let mut spinner = MultiSpinner::new();
        for repo in &self.repo {
            let repo = repo.clone();
            let open = self.open.clone();
            let root = Arc::clone(&root);
            let config = Arc::clone(&config);

            spinner = spinner.with_spin_while(format!("Cloning {}...", repo), move || async move {
                Self::clone(
                    &root,
                    &config,
                    &repo,
                    open.as_deref(),
                    CloneOptions {
                        recursive: self.recursive,
                    },
                )
            });
        }

        spinner.collect().await?.into_iter().for_each(
            |CloneResult {
                 path,
                 profile,
                 open,
             }| {
                info!(
                    "Cloned a repository successfully to: {}",
                    path.to_string_lossy(),
                );

                if let Some(name) = profile {
                    info!(
                        "\t-> Attached profile [{}] successfully.",
                        style(name).bold()
                    );
                }

                if let Some(app) = open {
                    info!(
                        "\t-> Opened the repository in [{}] successfully.",
                        style(&app).bold(),
                    );
                }
            },
        );

        Ok(())
    }

    fn clone(
        root: &Root,
        config: &Config,
        repo: &str,
        open: Option<&str>,
        options: CloneOptions,
    ) -> Result<CloneResult> {
        let url = Url::from_str(repo, config.defaults.owner.as_deref())?;
        let path = PathBuf::from(Path::resolve(root, &url));
        let profile = config
            .rules
            .resolve(&url)
            .and_then(|r| config.profiles.resolve(&r.profile));

        config
            .git
            .strategy
            .clone
            .clone_repository(url, &path, &options)?;

        let repo = Repository::open(&path)?;
        let profile = if let Some((name, p)) = profile {
            p.apply(&mut repo.config()?)?;
            Some(name.to_string())
        } else {
            None
        };

        let open = if let Some(app) = open {
            config.applications.open_or_intermediate(app, &path)?;
            Some(app.to_string())
        } else {
            None
        };

        Ok(CloneResult {
            path: repo.workdir().unwrap().to_path_buf(),
            profile,
            open,
        })
    }
}

struct CloneResult {
    path: PathBuf,
    profile: Option<String>,
    open: Option<String>,
}
