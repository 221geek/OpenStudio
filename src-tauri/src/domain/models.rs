use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandColor {
    pub name: String,
    pub hex: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandKit {
    pub id: String,
    pub name: String,
    pub sector: String,
    pub description: String,
    pub colors: Vec<BrandColor>,
    pub display_font: String,
    pub body_font: String,
    pub tone: Vec<String>,
    pub dos: Vec<String>,
    pub donts: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSpec {
    pub headline: String,
    #[serde(default)]
    pub image_data: String,
    #[serde(default)]
    pub kicker: String,
    #[serde(default)]
    pub subline: String,
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub cta: String,
    #[serde(default)]
    pub footer: String,
    #[serde(default)]
    pub composition: String,
    #[serde(default)]
    pub reference_note: String,
    pub scheme: String,
    pub alignment: String,
    pub title_size: u32,
    pub shape: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualVersion {
    pub id: String,
    pub project_id: String,
    pub sequence: i64,
    pub label: String,
    pub prompt: String,
    pub spec: VisualSpec,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub title: String,
    pub brand_kit_id: String,
    pub format: String,
    pub created_at: String,
    pub updated_at: String,
    pub versions: Vec<VisualVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub brand_kit_id: String,
    pub format: String,
    pub created_at: String,
    pub updated_at: String,
    pub latest_version: Option<VisualVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    CodexCli,
    ClaudeCli,
    OpenAiCompatible,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub kind: ProviderKind,
    pub name: String,
    pub model: String,
    pub endpoint: Option<String>,
    pub enabled: bool,
    pub has_secret: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInput {
    pub id: Option<String>,
    pub kind: ProviderKind,
    pub name: String,
    pub model: String,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationInput {
    pub project_id: String,
    pub prompt: String,
    pub base_version_id: Option<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub name: String,
    pub mime_type: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppBootstrap {
    pub projects: Vec<ProjectSummary>,
    pub brand_kits: Vec<BrandKit>,
    pub providers: Vec<Provider>,
}
