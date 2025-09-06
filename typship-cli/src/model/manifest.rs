use serde::{Deserialize, Serialize};
use typst_syntax::package::{PackageManifest, ToolInfo};

pub trait ManifestTools: TryFrom<ToolInfo> + Into<ToolInfo> + Sized {}

pub trait GetTools<T: ManifestTools> {
    fn tools(&self) -> T;
}

impl GetTools<TypshipTools> for PackageManifest {
    fn tools(&self) -> TypshipTools {
        self.tool.clone().try_into().unwrap_or_default()
    }
}

pub trait UpdateTools<T: ManifestTools> {
    fn mut_tools(&mut self, f: impl FnOnce(&mut T));
}

impl UpdateTools<TypshipTools> for PackageManifest {
    fn mut_tools(&mut self, f: impl FnOnce(&mut TypshipTools)) {
        let mut tools = self.tool.clone().try_into().unwrap_or_default();
        f(&mut tools);
        self.tool = tools.into();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypshipInfo {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TypshipTools {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typship: Option<TypshipInfo>,
}

impl TryFrom<ToolInfo> for TypshipTools {
    type Error = anyhow::Error;

    fn try_from(value: ToolInfo) -> Result<Self, Self::Error> {
        Ok(toml::from_str(&toml::to_string(&value)?)?)
    }
}

impl From<TypshipTools> for ToolInfo {
    fn from(value: TypshipTools) -> Self {
        toml::from_str(&toml::to_string(&value).unwrap()).unwrap()
    }
}

impl ManifestTools for TypshipTools {}
