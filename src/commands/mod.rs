use clap::Subcommand;

pub mod check;
pub mod clean;
pub mod copy;
pub mod dev;
pub mod download;
pub mod exclude;
pub mod init;
pub mod install;
pub mod login;
pub mod publish;
pub mod sync;

#[derive(Subcommand)]
pub(crate) enum Commands {
    Check(check::CheckArgs),
    Clean(clean::CleanArgs),
    Dev(dev::DevArgs),
    Copy(copy::CopyArgs),
    Download(download::DownloadArgs),
    Exclude(exclude::ExcludeArgs),
    Init(init::InitArgs),
    Install(install::InstallArgs),
    Login(login::LoginArgs),
    Publish(publish::PublishArgs),
    Sync(sync::SyncArgs),
}
