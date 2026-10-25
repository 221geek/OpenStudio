use crate::{
    domain::{models::*, visual::render_svg},
    infrastructure::{database::Database, providers},
    AppState,
};
use chrono::Utc;
use std::{fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;

fn db<'a>(state: &'a State<'a, AppState>) -> Result<std::sync::MutexGuard<'a, Database>, String> {
    state
        .0
        .lock()
        .map_err(|_| "Application storage is temporarily unavailable.".into())
}
#[tauri::command]
pub fn bootstrap(state: State<AppState>) -> Result<AppBootstrap, String> {
    let d = db(&state)?;
    Ok(AppBootstrap {
        projects: d.list_projects()?,
        brand_kits: d.list_brand_kits()?,
        providers: d.list_providers()?,
    })
}
#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<ProjectSummary>, String> {
    db(&state)?.list_projects()
}
#[tauri::command]
pub fn get_project(state: State<AppState>, id: String) -> Result<Option<Project>, String> {
    db(&state)?.get_project(&id)
}
#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    title: String,
    brand_kit_id: String,
    format: String,
) -> Result<Project, String> {
    db(&state)?.create_project(title, brand_kit_id, format)
}
#[tauri::command]
pub fn update_project(
    state: State<AppState>,
    id: String,
    title: String,
    brand_kit_id: String,
    format: String,
) -> Result<Project, String> {
    db(&state)?.update_project(&id, title, brand_kit_id, format)
}
#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    db(&state)?.delete_project(&id)
}
#[tauri::command]
pub fn list_brand_kits(state: State<AppState>) -> Result<Vec<BrandKit>, String> {
    db(&state)?.list_brand_kits()
}
#[tauri::command]
pub fn save_brand_kit(state: State<AppState>, mut kit: BrandKit) -> Result<BrandKit, String> {
    if kit.id.is_empty() {
        kit.id = Uuid::new_v4().to_string();
    }
    if kit.created_at.is_empty() {
        kit.created_at = Utc::now().to_rfc3339();
    }
    if kit.colors.len() < 3 {
        return Err("A direction artistique needs at least three colours.".into());
    }
    db(&state)?.save_brand_kit(kit)
}
#[tauri::command]
pub fn delete_brand_kit(state: State<AppState>, id: String) -> Result<(), String> {
    db(&state)?.delete_brand_kit(&id)
}
#[tauri::command]
pub fn list_providers(state: State<AppState>) -> Result<Vec<Provider>, String> {
    db(&state)?.list_providers()
}
#[tauri::command]
pub fn save_provider(state: State<AppState>, input: ProviderInput) -> Result<Provider, String> {
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let old = db(&state)?
        .list_providers()?
        .into_iter()
        .find(|p| p.id == id);
    let has_secret = input
        .api_key
        .as_ref()
        .map(|x| !x.trim().is_empty())
        .unwrap_or(old.as_ref().map(|x| x.has_secret).unwrap_or(false));
    if let Some(key) = input.api_key.filter(|x| !x.trim().is_empty()) {
        providers::store_secret(&id, &key)?;
    }
    let p = Provider {
        id,
        kind: input.kind,
        name: input.name,
        model: input.model,
        endpoint: input.endpoint,
        enabled: input.enabled,
        has_secret,
        created_at: old
            .map(|x| x.created_at)
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    };
    db(&state)?.save_provider(p)
}
#[tauri::command]
pub fn remove_provider(state: State<AppState>, id: String) -> Result<(), String> {
    providers::delete_secret(&id);
    db(&state)?.remove_provider(&id)
}
#[tauri::command]
pub fn activate_provider(state: State<AppState>, id: String) -> Result<(), String> {
    db(&state)?.enable_provider(&id)
}
#[tauri::command]
pub async fn check_local_providers() -> Vec<(ProviderKind, bool)> {
    providers::detect_local().await
}
#[tauri::command]
pub async fn generate_version(
    state: State<'_, AppState>,
    input: GenerationInput,
) -> Result<VisualVersion, String> {
    if input.attachments.len() > 8 {
        return Err("Attach up to eight reference files at a time.".into());
    }
    if input
        .attachments
        .iter()
        .any(|x| x.data_base64.len() > 14_000_000)
    {
        return Err("Each reference file must be smaller than 10 MB.".into());
    }
    let (provider, brand, base, format) = {
        let d = db(&state)?;
        let p = d
            .list_providers()?
            .into_iter()
            .find(|p| p.enabled)
            .ok_or("Connect and activate an AI provider before creating a visual.")?;
        if matches!(
            p.kind,
            ProviderKind::OpenAiCompatible | ProviderKind::Anthropic
        ) && !p.has_secret
        {
            return Err("This provider needs an API key in Settings.".into());
        };
        let project = d
            .get_project(&input.project_id)?
            .ok_or("Project not found")?;
        let brand = d
            .list_brand_kits()?
            .into_iter()
            .find(|b| b.id == project.brand_kit_id)
            .ok_or("The project's direction artistique no longer exists.")?;
        let base = input
            .base_version_id
            .and_then(|id| project.versions.into_iter().find(|x| x.id == id));
        (p, brand, base, project.format)
    };
    let (label, spec) = providers::generate(
        &provider,
        &brand,
        &input.prompt,
        base.as_ref().map(|x| &x.spec),
        &input.attachments,
        &format,
    )
    .await?;
    db(&state)?.append_version(&input.project_id, label, input.prompt, spec)
}
#[tauri::command]
pub async fn generate_brand_kit(
    state: State<'_, AppState>,
    name: String,
) -> Result<BrandKit, String> {
    let provider = {
        let d = db(&state)?;
        d.list_providers()?
            .into_iter()
            .find(|p| p.enabled)
            .ok_or("Connect and activate an AI provider before creating a direction artistique.")?
    };
    if matches!(
        provider.kind,
        ProviderKind::OpenAiCompatible | ProviderKind::Anthropic
    ) && !provider.has_secret
    {
        return Err("This provider needs an API key in Settings.".into());
    }
    let kit = providers::generate_brand_kit(&provider, &name).await?;
    db(&state)?.save_brand_kit(kit)
}
#[tauri::command]
pub fn export_version(
    state: State<AppState>,
    version_id: String,
    extension: String,
    path: String,
) -> Result<(), String> {
    let d = db(&state)?;
    let projects = d.list_projects()?;
    let mut found = None;
    for project in projects {
        if let Some(full) = d.get_project(&project.id)? {
            if let Some(v) = full.versions.into_iter().find(|v| v.id == version_id) {
                let brand = d
                    .list_brand_kits()?
                    .into_iter()
                    .find(|b| b.id == full.brand_kit_id)
                    .ok_or("Brand kit not found")?;
                found = Some((v, brand, full.format));
                break;
            }
        }
    }
    let (version, brand, format) = found.ok_or("Version not found")?;
    let svg = render_svg(&version.spec, &brand, &format)?;
    let output = PathBuf::from(path);
    if extension.eq_ignore_ascii_case("svg") {
        fs::write(output, svg).map_err(|e| e.to_string())?;
    } else if extension.eq_ignore_ascii_case("png") {
        let tree = resvg::usvg::Tree::from_str(&svg, &resvg::usvg::Options::default())
            .map_err(|e| e.to_string())?;
        let size = tree.size().to_int_size();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("Cannot allocate PNG canvas")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );
        pixmap.save_png(output).map_err(|e| e.to_string())?;
    } else {
        return Err("Only SVG and PNG exports are supported.".into());
    };
    Ok(())
}

#[tauri::command]
pub fn render_version_svg(state: State<AppState>, version_id: String) -> Result<String, String> {
    let d = db(&state)?;
    for project in d.list_projects()? {
        if let Some(full) = d.get_project(&project.id)? {
            if let Some(version) = full.versions.into_iter().find(|v| v.id == version_id) {
                let brand = d
                    .list_brand_kits()?
                    .into_iter()
                    .find(|b| b.id == full.brand_kit_id)
                    .ok_or("Brand kit not found")?;
                return render_svg(&version.spec, &brand, &full.format);
            }
        }
    }
    Err("Version not found".into())
}
