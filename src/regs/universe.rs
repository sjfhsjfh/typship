/// Typst Official Package Registry: Universeuse anyhow::anyhow;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{LazyLock, OnceLock};

use anyhow::{anyhow, bail, Result};
use crossterm::style::Stylize;
use dialoguer::{Confirm, Input};
use futures_util::TryStreamExt;
use log::{info, warn};
use octocrab::models::pulls::PullRequest;
use octocrab::models::repos::{ContentItems, Object};
use octocrab::{params, Octocrab, Page};
use secrecy::SecretString;
use typst_syntax::package::{PackageManifest, PackageVersion};

use crate::config::CONFIG;
use crate::utils::walkers::walker_publish;
use crate::utils::{config_file, save_config};

// pub const UNIVERSE_REPO_ID: RepositoryId =
// RepositoryId::from("R_kgDOJ0PIWA");
pub const UNIVERSE_REPO_NAME: &str = "packages";
pub const UNIVERSE_REPO_OWNER: &str = "typst";

/// Unauthorized client for public access
pub static PUBLIC_CLIENT: LazyLock<Octocrab> =
    LazyLock::new(|| Octocrab::builder().build().unwrap());

pub static AUTH_CLIENT: OnceLock<Octocrab> = OnceLock::new();

pub fn get_authenticated_client() -> Result<&'static Octocrab> {
    // TODO: better secret management
    // TODO: Lock retry?
    let token = CONFIG
        .try_lock()?
        .tokens
        .universe
        .clone()
        .ok_or(anyhow::anyhow!(
            "You need to set up the token first. Run `typship login universe`."
        ))?;
    let token = SecretString::from(token);
    Ok(AUTH_CLIENT.get_or_init(|| Octocrab::builder().personal_token(token).build().unwrap()))
}

/// Get the list of package names under `packages/preview` directory in the
/// official Universe (GitHub) registry.
pub async fn packages() -> Result<ContentItems> {
    Ok(PUBLIC_CLIENT
        .repos(UNIVERSE_REPO_OWNER, UNIVERSE_REPO_NAME)
        .get_content()
        .path("packages/preview")
        .r#ref("main")
        .send()
        .await?)
}

/// Get the list of package versions under `packages/preview/{package_name}`
/// directory in the official Universe (GitHub) registry.
pub async fn package_versions(package_name: &str) -> Result<ContentItems> {
    Ok(PUBLIC_CLIENT
        .repos(UNIVERSE_REPO_OWNER, UNIVERSE_REPO_NAME)
        .get_content()
        .path(format!("packages/preview/{}", package_name))
        .r#ref("main")
        .send()
        .await?)
}

/// Get the list of *OPEN* pull requests in the official Universe (GitHub)
/// registry.
pub async fn pending_list() -> Result<Page<PullRequest>> {
    Ok(PUBLIC_CLIENT
        .pulls(UNIVERSE_REPO_OWNER, UNIVERSE_REPO_NAME)
        .list()
        .state(octocrab::params::State::Open)
        .send()
        .await?)
}

pub fn login() -> Result<()> {
    let overwrite = if CONFIG.try_lock()?.tokens.universe.is_some() {
        info!("Already logged in to the Universe registry");
        dialoguer::Confirm::new()
            .with_prompt("Do you want to overwrite the existing token?")
            .default(false)
            .interact()?
    } else {
        true
    };
    if !overwrite {
        return Ok(());
    }
    let token = dialoguer::Password::new()
        .with_prompt("Enter your GitHub personal access token (use `fine-grained token` instead of `tokens (classic)`)")
        .interact()?;
    CONFIG.try_lock()?.tokens.universe = Some(token);
    if let Ok(cfg) = CONFIG.try_lock() {
        save_config(&cfg)?;
    } else {
        anyhow::bail!("Failed to save the configuration file");
    }
    info!(
        "Your token has been saved to {}",
        config_file().to_string_lossy()
    );
    Ok(())
}

pub async fn publish(manifest: &PackageManifest, package_dir: &Path, dry_run: bool) -> Result<()> {
    // TODO: check if exist in package repo(name), check pr
    info!("Checking the packages in the official packages repo...");
    let mut is_new_package = true;
    let packages = packages().await?;
    for package in packages.items {
        if package.name != manifest.package.name {
            continue;
        }
        info!("Package `{}` found in official packages repo", package.name);
        is_new_package = false;
        let versions = package_versions(&package.name).await?;
        let existing_versions = versions
            .items
            .into_iter()
            .map(|v| v.name)
            .collect::<Vec<_>>();
        info!("Existing versions: {}", existing_versions.join(", "));
        if existing_versions.contains(&manifest.package.version.to_string()) {
            bail!(
                "Package version `{}` already exists in the official packages repo",
                &manifest.package.version.to_string()
            )
        }
    }

    info!("Checking the pending PRs...");
    let prs = pending_list().await?;
    for pr in prs.items {
        if let Some(submission) = pr
            .title
            .and_then(|t| PackageSubmission::try_from_title(&t).ok())
        {
            if submission.name == manifest.package.name {
                match submission.version.cmp(&manifest.package.version) {
                    std::cmp::Ordering::Greater => {
                        bail!(
                            "Package version `{}`(newer) is already submitted in PR #{}: {}",
                            submission.version,
                            pr.number,
                            pr.url.underlined()
                        );
                    }
                    std::cmp::Ordering::Equal => {
                        bail!(
                            "Package version `{}`(current) is already submitted in PR #{}: {}",
                            submission.version,
                            pr.number,
                            pr.url.underlined()
                        );
                    }
                    std::cmp::Ordering::Less => {
                        warn!(
                            "Package version `{}`(older) is already submitted in PR #{}: {}",
                            submission.version,
                            pr.number,
                            pr.url.underlined()
                        );
                    }
                }
            }
        }
    }

    // TODO: confirm the files, pr message, etc.
    let submission = PackageSubmission {
        name: manifest.package.name.clone().into(),
        version: manifest.package.version,
        msg: Some(SubmissionMessage {
            is_new_package,
            desc: manifest
                .package
                .description
                .clone()
                .ok_or(anyhow!("Missing description"))?
                .into(),
        }),
    };

    // Danger zone
    let client = get_authenticated_client()?;
    let me = client.current().user().await?;
    let my_repo: String = Input::new()
        .with_prompt("Enter the name of your forked repository")
        .allow_empty(false)
        .default(UNIVERSE_REPO_NAME.into())
        .interact_text()?;
    let my_fork = client.repos(&me.login, my_repo);
    let parent = my_fork.get().await?.parent;
    if let Some(p) = parent {
        if p.name != UNIVERSE_REPO_NAME
            || p.owner
                .map(|o| o.login != UNIVERSE_REPO_OWNER)
                .unwrap_or(true)
        {
            bail!("The given repository is not a fork of the official packages repo");
        }
    } else {
        bail!("The given repository is not a fork");
    }

    info!("Creating corresponding branch in your fork...");
    if !dry_run {
        let branch_name = submission.branch_name().clone();
        let branch_name = branch_name.as_str();
        let main_head = client
            .repos(UNIVERSE_REPO_OWNER, UNIVERSE_REPO_NAME)
            .get_ref(&params::repos::Reference::Branch("main".into()))
            .await?
            .object;
        let main_sha = match &main_head {
            Object::Commit { sha, .. } => sha,
            Object::Tag { sha, .. } => sha,
            _ => unreachable!(),
        };

        if my_fork
            .list_branches()
            .send()
            .await?
            .into_stream(client)
            .try_any(|b| async move { b.name == branch_name })
            .await?
        {
            if !Confirm::new()
                .with_prompt(format!(
                    "Branch `{}` already exists in your fork. Do you want to overwrite it?",
                    branch_name
                ))
                .default(false)
                .interact()?
            {
                bail!("Aborted");
            }
            my_fork
                .delete_ref(&params::repos::Reference::Branch(branch_name.into()))
                .await?;
        }
        let new_branch = my_fork
            .create_ref(
                &params::repos::Reference::Branch(submission.branch_name()),
                main_sha,
            )
            .await?;

        info!(
            // TODO: add url here?
            "Branch `{}` created",
            new_branch.ref_field,
        );
    } else {
        info!("Dry run: branch creation skipped");
    }

    info!("Uploading files to personal fork...");
    let mut files = Vec::new();

    for entry in walker_publish(package_dir).flatten() {
        if !entry.path().is_file() {
            continue;
        }
        let entry = entry.path();
        let entry = entry.strip_prefix(package_dir).unwrap();
        files.push(entry.to_path_buf());
    }
    info!(
        "Files to upload:\n{}",
        files
            .iter()
            .map(|f| format!("\t{}", f.display()))
            .collect::<Vec<_>>()
            .join("\n")
    );
    if !dry_run {
        if !Confirm::new()
            .with_prompt("Do you want to continue?")
            .default(false)
            .interact()?
        {
            bail!("Aborted");
        }

        // TODO: multi-threading?
        for file in files {
            let content = std::fs::read(package_dir.join(&file))?;
            my_fork
                .create_file(
                    submission
                        .repo_path()
                        .join(&file)
                        .to_string_lossy()
                        .into_owned(),
                    format!("[Typship] Add {}", file.display()),
                    &content,
                )
                .branch(submission.branch_name())
                .send()
                .await
                .map(|_| info!("Uploaded: {}", file.display()))?;
        }
    } else {
        info!("Dry run: file upload skipped");
    }

    info!("Generating submission PR...");
    if !dry_run {
        if let Some(msg) = &submission.msg {
            let sub = client
                .pulls(UNIVERSE_REPO_OWNER, UNIVERSE_REPO_NAME)
                .create(
                    submission.title(),
                    format!("{}:{}", me.login, submission.branch_name()),
                    "main",
                )
                .body(msg.to_string(manifest.template.is_some()))
                .draft(true)
                .send()
                .await?;
            info!(
                "PR created: {}",
                sub.html_url.unwrap().as_str().underlined()
            );
        } else {
            bail!("Missing submission message");
        }
    } else {
        info!("Dry run: PR creation skipped");
    }
    Ok(())
}

struct PackageSubmission {
    name: String,
    version: PackageVersion,
    msg: Option<SubmissionMessage>,
}

impl PackageSubmission {
    pub fn title(&self) -> String {
        format!("{}:{}", self.name, self.version)
    }

    pub fn repo_path(&self) -> PathBuf {
        PathBuf::from(format!("packages/preview/{}/{}", self.name, self.version))
    }

    pub fn branch_name(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    fn try_from_title(title: &str) -> Result<Self> {
        let parts = title.split(':').collect::<Vec<_>>();
        if parts.len() != 2 {
            bail!("Invalid title format");
        }
        let name = parts[0].to_string();
        let version = PackageVersion::from_str(parts[1]).map_err(|_| anyhow!("Invalid version"))?;
        Ok(Self {
            name,
            version,
            msg: None,
        })
    }
}

struct SubmissionMessage {
    /// I am submitting
    /// - [ ] a new package
    /// - [ ] an update for a package
    is_new_package: bool,

    /// Please add a brief description of your package below and explain why you
    /// think it is useful to others. If this is an update, please briefly say
    /// what changed.
    ///
    /// Description: Explain what the package does and why it's useful.
    desc: String,
}

impl SubmissionMessage {
    pub fn to_string(&self, has_template: bool) -> String {
        // TODO: maybe leave a interactive dialog for the user to fill in the
        // description?
        let template = "\n- [x] ensured that my package is licensed such that users can use and distribute the contents of its template directory without restriction, after modifying them through normal use.\n";
        format!(
            "I am submitting\n\
            - [{}] a new package\n\
            - [{}] an update for a package\n\n\
            Description: {}\n\n\
            I have read and followed the submission guidelines and, in particular, I\n\
            - [x] selected a name that isn't the most obvious or canonical name for what the package does\n\
            - [x] added a `typst.toml` file with all required keys\n\
            - [x] added a `README.md` with documentation for my package\n\
            - [x] have chosen a license and added a `LICENSE` file or linked one in my `README.md`\n\
            - [x] tested my package locally on my system and it worked\n\
            - [x] `exclude`d PDFs or README images, if any, but not the LICENSE\n{}",
            if self.is_new_package { "x" } else { " " },
            if !self.is_new_package { "x" } else { " " },
            self.desc,
            if has_template { template } else { "" }
        )
    }
}
