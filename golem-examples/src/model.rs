use fancy_regex::{Match, Regex};
use inflector::Inflector;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, io};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::FromStr, Serialize, Deserialize,
)]
pub struct ComponentName(String);

static COMPONENT_NAME_SPLIT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("(?=[A-Z\\-_:])").unwrap());

impl ComponentName {
    pub fn new(name: impl AsRef<str>) -> ComponentName {
        ComponentName(name.as_ref().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parts(&self) -> Vec<String> {
        let matches: Vec<Result<Match, fancy_regex::Error>> =
            COMPONENT_NAME_SPLIT_REGEX.find_iter(&self.0).collect();
        let mut parts: Vec<&str> = vec![];
        let mut last = 0;
        for m in matches.into_iter().flatten() {
            let part = &self.0[last..m.start()];
            if !part.is_empty() {
                parts.push(part);
            }
            last = m.end();
        }
        parts.push(&self.0[last..]);

        let mut result: Vec<String> = Vec::with_capacity(parts.len());
        for part in parts {
            let s = part.to_lowercase();
            let s = s.strip_prefix('-').unwrap_or(&s);
            let s = s.strip_prefix('_').unwrap_or(s);
            let s = s.strip_prefix(':').unwrap_or(s);
            result.push(s.to_string());
        }
        result
    }

    pub fn to_kebab_case(&self) -> String {
        self.parts().join("-")
    }

    pub fn to_snake_case(&self) -> String {
        self.parts().join("_")
    }

    pub fn to_pascal_case(&self) -> String {
        self.parts().iter().map(|s| s.to_title_case()).collect()
    }

    pub fn to_camel_case(&self) -> String {
        self.to_pascal_case().to_camel_case()
    }
}

impl fmt::Display for ComponentName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExampleKind {
    Standalone,
    ComposableAppCommon {
        group: ComposableAppGroupName,
        skip_if_exists: Option<PathBuf>,
    },
    ComposableAppComponent {
        group: ComposableAppGroupName,
    },
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, Serialize, Deserialize,
)]
pub enum GuestLanguage {
    Rust,
    Go,
    C,
    Zig,
    JavaScript,
    TypeScript,
    CSharp,
    Swift,
    Grain,
    Python,
    Scala2,
}

impl GuestLanguage {
    pub fn from_string(s: impl AsRef<str>) -> Option<GuestLanguage> {
        match s.as_ref().to_lowercase().as_str() {
            "rust" => Some(GuestLanguage::Rust),
            "go" => Some(GuestLanguage::Go),
            "c" | "c++" | "cpp" => Some(GuestLanguage::C),
            "zig" => Some(GuestLanguage::Zig),
            "js" | "javascript" => Some(GuestLanguage::JavaScript),
            "ts" | "typescript" => Some(GuestLanguage::TypeScript),
            "c#" | "cs" | "csharp" => Some(GuestLanguage::CSharp),
            "swift" => Some(GuestLanguage::Swift),
            "grain" => Some(GuestLanguage::Grain),
            "py" | "python" => Some(GuestLanguage::Python),
            "scala2" => Some(GuestLanguage::Scala2),
            _ => None,
        }
    }

    pub fn id(&self) -> String {
        match self {
            GuestLanguage::Rust => "rust".to_string(),
            GuestLanguage::Go => "go".to_string(),
            GuestLanguage::C => "c".to_string(),
            GuestLanguage::Zig => "zig".to_string(),
            GuestLanguage::JavaScript => "js".to_string(),
            GuestLanguage::TypeScript => "ts".to_string(),
            GuestLanguage::CSharp => "cs".to_string(),
            GuestLanguage::Swift => "swift".to_string(),
            GuestLanguage::Grain => "grain".to_string(),
            GuestLanguage::Python => "python".to_string(),
            GuestLanguage::Scala2 => "scala2".to_string(),
        }
    }

    pub fn tier(&self) -> GuestLanguageTier {
        match self {
            GuestLanguage::Rust => GuestLanguageTier::Tier1,
            GuestLanguage::Go => GuestLanguageTier::Tier1,
            GuestLanguage::C => GuestLanguageTier::Tier1,
            GuestLanguage::Zig => GuestLanguageTier::Tier1,
            GuestLanguage::JavaScript => GuestLanguageTier::Tier1,
            GuestLanguage::TypeScript => GuestLanguageTier::Tier1,
            GuestLanguage::CSharp => GuestLanguageTier::Tier3,
            GuestLanguage::Swift => GuestLanguageTier::Tier2,
            GuestLanguage::Grain => GuestLanguageTier::Tier2,
            GuestLanguage::Python => GuestLanguageTier::Tier1,
            GuestLanguage::Scala2 => GuestLanguageTier::Tier1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GuestLanguage::Rust => "Rust",
            GuestLanguage::Go => "Go",
            GuestLanguage::C => "C",
            GuestLanguage::Zig => "Zig",
            GuestLanguage::JavaScript => "JavaScript",
            GuestLanguage::TypeScript => "TypeScript",
            GuestLanguage::CSharp => "C#",
            GuestLanguage::Swift => "Swift",
            GuestLanguage::Grain => "Grain",
            GuestLanguage::Python => "Python",
            GuestLanguage::Scala2 => "Scala 2",
        }
    }
}

impl fmt::Display for GuestLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for GuestLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GuestLanguage::from_string(s).ok_or({
            let all = GuestLanguage::iter()
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<String>>()
                .join(", ");
            format!("Unknown guest language: {s}. Expected one of {all}")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter, Serialize, Deserialize)]
pub enum GuestLanguageTier {
    Tier1,
    Tier2,
    Tier3,
}

impl GuestLanguageTier {
    pub fn from_string(s: impl AsRef<str>) -> Option<GuestLanguageTier> {
        match s.as_ref().to_lowercase().as_str() {
            "tier1" | "1" => Some(GuestLanguageTier::Tier1),
            "tier2" | "2" => Some(GuestLanguageTier::Tier2),
            "tier3" | "3" => Some(GuestLanguageTier::Tier3),
            _ => None,
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            GuestLanguageTier::Tier1 => 1,
            GuestLanguageTier::Tier2 => 2,
            GuestLanguageTier::Tier3 => 3,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            GuestLanguageTier::Tier1 => "tier1",
            GuestLanguageTier::Tier2 => "tier2",
            GuestLanguageTier::Tier3 => "tier3",
        }
    }
}

impl fmt::Display for GuestLanguageTier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for GuestLanguageTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        GuestLanguageTier::from_string(s).ok_or(format!("Unexpected guest language tier {s}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PackageName((String, String));

impl PackageName {
    pub fn from_string(s: impl AsRef<str>) -> Option<PackageName> {
        let parts: Vec<&str> = s.as_ref().split(':').collect();
        match parts.as_slice() {
            &[n1, n2] => Some(PackageName((n1.to_string(), n2.to_string()))),
            _ => None,
        }
    }

    pub fn to_pascal_case(&self) -> String {
        format!(
            "{}{}",
            self.0 .0.to_pascal_case(),
            self.0 .1.to_pascal_case()
        )
    }

    pub fn to_snake_case(&self) -> String {
        format!(
            "{}_{}",
            self.0 .0.to_snake_case(),
            self.0 .1.to_snake_case()
        )
    }

    pub fn to_string_with_double_colon(&self) -> String {
        format!("{}::{}", self.0 .0, self.0 .1)
    }

    pub fn to_string_with_colon(&self) -> String {
        format!("{}:{}", self.0 .0, self.0 .1)
    }

    pub fn to_string_with_slash(&self) -> String {
        format!("{}/{}", self.0 .0, self.0 .1)
    }

    pub fn to_kebab_case(&self) -> String {
        format!("{}-{}", self.0 .0, self.0 .1)
    }

    pub fn to_rust_binding(&self) -> String {
        format!(
            "{}::{}",
            self.0 .0.to_snake_case(),
            self.0 .1.to_snake_case()
        )
    }

    pub fn namespace(&self) -> String {
        self.0 .0.to_string()
    }

    pub fn namespace_title_case(&self) -> String {
        self.0 .0.to_title_case()
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_colon())
    }
}

impl FromStr for PackageName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PackageName::from_string(s).ok_or(format!(
            "Unexpected package name {s}. Must be in 'pack:name' format"
        ))
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::FromStr, Serialize, Deserialize,
)]
pub struct ExampleName(String);

impl ExampleName {
    pub fn from_string(s: impl AsRef<str>) -> ExampleName {
        ExampleName(s.as_ref().to_string())
    }

    pub fn as_string(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExampleName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::FromStr, Serialize, Deserialize,
)]
pub struct ComposableAppGroupName(String);

impl ComposableAppGroupName {
    pub fn from_string(s: impl AsRef<str>) -> ComposableAppGroupName {
        ComposableAppGroupName(s.as_ref().to_string())
    }

    pub fn as_string(&self) -> &str {
        &self.0
    }
}

impl Default for ComposableAppGroupName {
    fn default() -> Self {
        ComposableAppGroupName("default".to_string())
    }
}

impl fmt::Display for ComposableAppGroupName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TargetExistsResolveMode {
    Skip,
    MergeOrSkip,
    Fail,
    MergeOrFail,
}

pub type MergeContents = Box<dyn FnOnce(&[u8]) -> io::Result<Vec<u8>>>;

pub enum TargetExistsResolveDecision {
    Skip,
    Merge(MergeContents),
}

#[derive(Debug, Clone)]
pub struct Example {
    pub name: ExampleName,
    pub kind: ExampleKind,
    pub language: GuestLanguage,
    pub description: String,
    pub example_path: PathBuf,
    pub instructions: String,
    pub adapter_source: Option<PathBuf>,
    pub adapter_target: Option<PathBuf>,
    pub wit_deps: Vec<PathBuf>,
    pub wit_deps_targets: Option<Vec<PathBuf>>,
    pub exclude: HashSet<String>,
    pub transform_exclude: HashSet<String>,
    pub transform: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleParameters {
    pub component_name: ComponentName,
    pub package_name: PackageName,
    pub target_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExampleMetadata {
    pub description: String,
    #[serde(rename = "appCommonGroup")]
    pub app_common_group: Option<String>,
    #[serde(rename = "appCommonSkipIfExists")]
    pub app_common_skip_if_exists: Option<String>,
    #[serde(rename = "appComponentGroup")]
    pub app_component_group: Option<String>,
    #[serde(rename = "requiresAdapter")]
    pub requires_adapter: Option<bool>,
    #[serde(rename = "adapterTarget")]
    pub adapter_target: Option<String>,
    #[serde(rename = "requiresGolemHostWIT")]
    pub requires_golem_host_wit: Option<bool>,
    #[serde(rename = "requiresWASI")]
    pub requires_wasi: Option<bool>,
    #[serde(rename = "witDepsPaths")]
    pub wit_deps_paths: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub instructions: Option<String>,
    #[serde(rename = "transformExclude")]
    pub transform_exclude: Option<Vec<String>>,
    pub transform: Option<bool>,
}

#[cfg(test)]
mod tests {
    use crate::model::{ComponentName, PackageName};
    use once_cell::sync::Lazy;

    static N1: Lazy<ComponentName> = Lazy::new(|| ComponentName::new("my-test-component"));
    static N2: Lazy<ComponentName> = Lazy::new(|| ComponentName::new("MyTestComponent"));
    static N3: Lazy<ComponentName> = Lazy::new(|| ComponentName::new("myTestComponent"));
    static N4: Lazy<ComponentName> = Lazy::new(|| ComponentName::new("my_test_component"));

    #[test]
    pub fn component_name_to_pascal_case() {
        assert_eq!(N1.to_pascal_case(), "MyTestComponent");
        assert_eq!(N2.to_pascal_case(), "MyTestComponent");
        assert_eq!(N3.to_pascal_case(), "MyTestComponent");
        assert_eq!(N4.to_pascal_case(), "MyTestComponent");
    }

    #[test]
    pub fn component_name_to_camel_case() {
        assert_eq!(N1.to_camel_case(), "myTestComponent");
        assert_eq!(N2.to_camel_case(), "myTestComponent");
        assert_eq!(N3.to_camel_case(), "myTestComponent");
        assert_eq!(N4.to_camel_case(), "myTestComponent");
    }

    #[test]
    pub fn component_name_to_snake_case() {
        assert_eq!(N1.to_snake_case(), "my_test_component");
        assert_eq!(N2.to_snake_case(), "my_test_component");
        assert_eq!(N3.to_snake_case(), "my_test_component");
        assert_eq!(N4.to_snake_case(), "my_test_component");
    }

    #[test]
    pub fn component_name_to_kebab_case() {
        assert_eq!(N1.to_kebab_case(), "my-test-component");
        assert_eq!(N2.to_kebab_case(), "my-test-component");
        assert_eq!(N3.to_kebab_case(), "my-test-component");
        assert_eq!(N4.to_kebab_case(), "my-test-component");
    }

    static P1: Lazy<PackageName> = Lazy::new(|| PackageName::from_string("foo:bar").unwrap());
    static P2: Lazy<PackageName> = Lazy::new(|| PackageName::from_string("foo:bar-baz").unwrap());

    #[test]
    pub fn package_name_to_pascal_case() {
        assert_eq!(P1.to_pascal_case(), "FooBar");
        assert_eq!(P2.to_pascal_case(), "FooBarBaz");
    }
}
