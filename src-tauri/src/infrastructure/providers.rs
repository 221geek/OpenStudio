use crate::domain::models::{Attachment, BrandKit, Provider, ProviderKind, VisualSpec};
use base64::{engine::general_purpose::STANDARD, Engine};
use keyring::Entry;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::process::Command;

const SERVICE: &str = "com.openstudio.app";
const POSTER_ART_DIRECTION_SKILL: &str =
    include_str!("../../../skills/poster-art-direction/SKILL.md");
const POSTER_QUALITY_GATE: &str =
    include_str!("../../../skills/poster-art-direction/references/poster-quality.md");

pub fn store_secret(provider_id: &str, secret: &str) -> Result<(), String> {
    Entry::new(SERVICE, provider_id)
        .map_err(|e| e.to_string())?
        .set_password(secret)
        .map_err(|e| e.to_string())
}
pub fn delete_secret(provider_id: &str) {
    if let Ok(entry) = Entry::new(SERVICE, provider_id) {
        let _ = entry.delete_credential();
    }
}
fn secret(provider_id: &str) -> Result<String, String> {
    Entry::new(SERVICE, provider_id)
        .map_err(|e| e.to_string())?
        .get_password()
        .map_err(|_| "The API key is unavailable in the system credential store.".to_string())
}

pub async fn detect_local() -> Vec<(ProviderKind, bool)> {
    let codex = Command::new("codex")
        .arg("--version")
        .output()
        .await
        .map(|x| x.status.success())
        .unwrap_or(false);
    let claude = Command::new("claude")
        .arg("--version")
        .output()
        .await
        .map(|x| x.status.success())
        .unwrap_or(false);
    vec![
        (ProviderKind::CodexCli, codex),
        (ProviderKind::ClaudeCli, claude),
    ]
}

pub async fn generate(
    provider: &Provider,
    brand: &BrandKit,
    prompt: &str,
    base: Option<&VisualSpec>,
    attachments: &[Attachment],
    format: &str,
) -> Result<(String, VisualSpec), String> {
    if provider.kind == ProviderKind::OpenAiCompatible && is_openai_image_model(&provider.model) {
        return generate_openai_image(provider, brand, prompt, attachments, format).await;
    }
    let plan_raw = request_provider(
        provider,
        &creative_analysis_instruction(brand, prompt, attachments),
        attachments,
    )
    .await?;
    let plan = parse_creative_plan(&plan_raw)?;
    let raw = request_provider(
        provider,
        &generation_instruction(brand, prompt, base, attachments, &plan),
        attachments,
    )
    .await?;
    parse_spec(&raw)
}

fn is_openai_image_model(model: &str) -> bool {
    let model = model.to_lowercase();
    model.starts_with("gpt-image-") || model == "chatgpt-image-latest"
}

async fn generate_openai_image(
    provider: &Provider,
    brand: &BrandKit,
    brief: &str,
    attachments: &[Attachment],
    format: &str,
) -> Result<(String, VisualSpec), String> {
    let endpoint = provider
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    let key = secret(&provider.id)?;
    let prompt = format!(
        "Create one premium, production-ready social poster. Follow this art-direction skill:\n{}\n\nBrand: {}. Palette (use visually, never print): {}. Typography direction: {} / {}. Brief: {}. Render all supplied public facts accurately and legibly. Use a deliberate editorial composition, rich visual hierarchy, and polished art direction. Do not render design notes, palette values, font names, placeholders, or OpenStudio branding.",
        POSTER_ART_DIRECTION_SKILL,
        brand.name,
        serde_json::to_string(&brand.colors).unwrap_or_default(),
        brand.display_font,
        brand.body_font,
        brief,
    );
    let size = image_size_for_format(format);
    let client = reqwest::Client::new();
    let response = if attachments
        .iter()
        .any(|file| file.mime_type.starts_with("image/"))
    {
        let mut form = reqwest::multipart::Form::new()
            .text("model", provider.model.clone())
            .text("prompt", prompt)
            .text("size", size.to_owned())
            .text("quality", "high")
            .text("output_format", "png");
        for attachment in attachments
            .iter()
            .filter(|file| file.mime_type.starts_with("image/"))
        {
            let bytes = STANDARD
                .decode(&attachment.data_base64)
                .map_err(|_| "An image reference could not be decoded.".to_string())?;
            let part = reqwest::multipart::Part::bytes(bytes)
                .file_name(attachment.name.clone())
                .mime_str(&attachment.mime_type)
                .map_err(|error| error.to_string())?;
            form = form.part("image[]", part);
        }
        client
            .post(format!("{endpoint}/images/edits"))
            .bearer_auth(key)
            .multipart(form)
            .send()
            .await
            .map_err(|error| error.to_string())?
    } else {
        client.post(format!("{endpoint}/images/generations")).bearer_auth(key).json(&json!({"model":provider.model,"prompt":prompt,"size":size,"quality":"high","output_format":"png","n":1})).send().await.map_err(|error| error.to_string())?
    };
    if !response.status().is_success() {
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Image provider request failed".into()));
    }
    let body: Value = response.json().await.map_err(|error| error.to_string())?;
    let image = body
        .pointer("/data/0/b64_json")
        .and_then(Value::as_str)
        .ok_or("The image provider returned no PNG data.")?;
    Ok((
        "Image generation".into(),
        VisualSpec {
            headline: brief.chars().take(80).collect(),
            image_data: format!("data:image/png;base64,{image}"),
            kicker: String::new(),
            subline: String::new(),
            details: Vec::new(),
            cta: String::new(),
            footer: brand.name.clone(),
            composition: "image".into(),
            reference_note: "Generated with the configured image model.".into(),
            scheme: "light".into(),
            alignment: "left".into(),
            title_size: 72,
            shape: String::new(),
        },
    ))
}

fn image_size_for_format(format: &str) -> &'static str {
    match format {
        "story" => "1024x1536",
        "landscape" => "1536x1024",
        _ => "1024x1536",
    }
}

#[derive(Debug, serde::Deserialize)]
struct CreativePlan {
    facts: Vec<String>,
    hierarchy: Vec<String>,
    reference_reading: String,
    visual_strategy: String,
    avoid: Vec<String>,
}

async fn request_provider(
    provider: &Provider,
    instruction: &str,
    attachments: &[Attachment],
) -> Result<String, String> {
    match provider.kind {
        ProviderKind::CodexCli => {
            run_cli_with_files(
                "codex",
                &["exec", "--skip-git-repo-check", "-"],
                instruction,
                attachments,
            )
            .await
        }
        ProviderKind::ClaudeCli => {
            run_cli_with_files(
                "claude",
                &["-p", "--output-format", "text"],
                instruction,
                attachments,
            )
            .await
        }
        ProviderKind::OpenAiCompatible => openai(provider, instruction, attachments).await,
        ProviderKind::Anthropic => anthropic(provider, instruction, attachments).await,
    }
}

async fn run_cli_with_files(
    binary: &str,
    args: &[&str],
    instruction: &str,
    attachments: &[Attachment],
) -> Result<String, String> {
    if attachments.is_empty() {
        return run_cli(binary, args, instruction).await;
    }
    let folder = std::env::temp_dir().join(format!("openstudio-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let paths = attachments
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let safe = a.name.replace(['/', '\\'], "_");
            let path = folder.join(format!("{i}-{safe}"));
            let bytes = STANDARD
                .decode(&a.data_base64)
                .map_err(|_| "An attached file could not be decoded.".to_string())?;
            std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
            Ok(path.display().to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let augmented = format!(
        "{instruction}\n\nReference attachments are local files. Inspect them before answering: {}",
        paths.join(", ")
    );
    let result = run_cli(binary, args, &augmented).await;
    let _ = std::fs::remove_dir_all(folder);
    result
}

pub async fn generate_brand_kit(provider: &Provider, name: &str) -> Result<BrandKit, String> {
    let instruction = format!("You are a brand strategist. Build a first editable artistic direction from this brand name only: {name}. Return ONLY JSON, no markdown: {{\"sector\":string,\"description\":string,\"colors\":[{{\"name\":string,\"hex\":\"#RRGGBB\",\"role\":string}}],\"display_font\":string,\"body_font\":string,\"tone\":[string],\"dos\":[string],\"donts\":[string]}}. Return exactly four accessible, coherent colors. Do not invent factual claims about the business; describe the visual direction as a proposal.");
    let raw = match provider.kind {
        ProviderKind::CodexCli => {
            run_cli(
                "codex",
                &["exec", "--skip-git-repo-check", "-"],
                &instruction,
            )
            .await?
        }
        ProviderKind::ClaudeCli => {
            run_cli("claude", &["-p", "--output-format", "text"], &instruction).await?
        }
        ProviderKind::OpenAiCompatible => openai(provider, &instruction, &[]).await?,
        ProviderKind::Anthropic => anthropic(provider, &instruction, &[]).await?,
    };
    parse_brand_kit(&raw, name)
}

async fn run_cli(binary: &str, args: &[&str], input: &str) -> Result<String, String> {
    let mut child = Command::new(binary)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| format!("{binary} is not available. Install it and sign in, then retry."))?;
    use tokio::io::AsyncWriteExt;
    child
        .stdin
        .take()
        .ok_or("Unable to open provider input")?
        .write_all(input.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    let output = tokio::time::timeout(Duration::from_secs(120), child.wait_with_output())
        .await
        .map_err(|_| "The provider timed out after two minutes.".to_string())?
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
async fn openai(
    provider: &Provider,
    prompt: &str,
    attachments: &[Attachment],
) -> Result<String, String> {
    let endpoint = provider
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    let key = secret(&provider.id)?;
    let mut content = vec![json!({"type":"text","text":prompt})];
    for a in attachments {
        if a.mime_type.starts_with("image/") {
            content.push(json!({"type":"image_url","image_url":{"url":format!("data:{};base64,{}",a.mime_type,a.data_base64),"detail":"high"}}));
        } else {
            content
                .push(json!({"type":"file","file":{"filename":a.name,"file_data":a.data_base64}}));
        }
    }
    let client = reqwest::Client::new();
    let response=client.post(format!("{endpoint}/chat/completions")).bearer_auth(key).json(&json!({"model":provider.model,"messages":[{"role":"user","content":content}],"temperature":0.7,"response_format":{"type":"json_object"}})).send().await.map_err(|e|e.to_string())?;
    if !response.status().is_success() {
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Provider request failed".into()));
    }
    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    body.pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or("Provider returned no text content".into())
}
async fn anthropic(
    provider: &Provider,
    prompt: &str,
    attachments: &[Attachment],
) -> Result<String, String> {
    let endpoint = provider
        .endpoint
        .as_deref()
        .unwrap_or("https://api.anthropic.com/v1/messages");
    let key = secret(&provider.id)?;
    let mut content = vec![json!({"type":"text","text":prompt})];
    for a in attachments {
        if a.mime_type.starts_with("image/") {
            content.push(json!({"type":"image","source":{"type":"base64","media_type":a.mime_type,"data":a.data_base64}}));
        } else if a.mime_type == "application/pdf" {
            content.push(json!({"type":"document","source":{"type":"base64","media_type":"application/pdf","data":a.data_base64}}));
        }
    }
    let response=reqwest::Client::new().post(endpoint).header("x-api-key",key).header("anthropic-version","2023-06-01").json(&json!({"model":provider.model,"max_tokens":1024,"messages":[{"role":"user","content":content}]})).send().await.map_err(|e|e.to_string())?;
    if !response.status().is_success() {
        return Err(response
            .text()
            .await
            .unwrap_or_else(|_| "Provider request failed".into()));
    }
    let body: Value = response.json().await.map_err(|e| e.to_string())?;
    body.pointer("/content/0/text")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or("Provider returned no text content".into())
}
fn creative_analysis_instruction(
    brand: &BrandKit,
    prompt: &str,
    attachments: &[Attachment],
) -> String {
    format!("You are the research and creative-strategy stage of a poster studio. Follow the embedded POSTER ART DIRECTION SKILL below. Do NOT design yet. Study the user brief, the brand system, and every attached reference image. Return ONLY JSON: {{\"facts\":[string],\"hierarchy\":[string],\"reference_reading\":string,\"visual_strategy\":string,\"avoid\":[string]}}. Facts are audience-facing facts ONLY: include every supplied deadline, date, place, URL, call-to-action and proper name verbatim. Never include hex codes, palette names, colour roles, font names or any other brand metadata in facts. Hierarchy must order what is read first through last. Reference_reading must identify composition, density, typography, imagery, palette and rhythm observed in the attachments. Visual_strategy must state exactly how to merge the reference with the brand colours and fonts. Never invent facts.\n\nPOSTER ART DIRECTION SKILL:\n{}\n\nAttached references: {}. Brand name: {}. Brand colours: {}. Brand typography: {} / {}. Brand rules: {}. Brief: {}",POSTER_ART_DIRECTION_SKILL,attachments.len(),brand.name,serde_json::to_string(&brand.colors).unwrap_or_default(),brand.display_font,brand.body_font,brand.dos.join("; "),prompt)
}
fn generation_instruction(
    brand: &BrandKit,
    prompt: &str,
    base: Option<&VisualSpec>,
    attachments: &[Attachment],
    plan: &CreativePlan,
) -> String {
    format!("You are the final production art director for an editable, premium social poster. You have already received a research plan. Apply the embedded POSTER ART DIRECTION SKILL and its quality gate. Obey the plan exactly. Produce a designed composition, never the generic pattern of centered text plus one decorative circle. Answer with ONLY JSON: {{\"label\":string,\"kicker\":string,\"headline\":string,\"subline\":string,\"details\":[string],\"cta\":string,\"footer\":string,\"composition\":\"editorial|event|split|typographic\",\"reference_note\":string,\"scheme\":\"light|dark|accent|secondary\",\"alignment\":\"left|center\",\"title_size\":number,\"shape\":\"circle\"}}. Every text field is public poster copy. NEVER output colour values, hex codes, palette roles, font names, design analysis or private notes in headline, kicker, subline, details, CTA or footer. Details must preserve only audience-facing facts from the research plan. Use a clear poster hierarchy, intentional negative space, and a reference-led arrangement. If a reference exists, its composition and visual rhythm outweigh a generic social template; the DA supplies colour and type, the reference supplies the art direction. Select composition event for information-dense event posters, split for image-led references, editorial for asymmetric type systems, and typographic only when the reference proves type alone is sufficient.\n\nPOSTER ART DIRECTION SKILL:\n{}\n\nQUALITY GATE:\n{}\n\nRESEARCH PLAN: facts={:?}; hierarchy={:?}; reference reading={}; visual strategy={}; avoid={:?}.\nBRAND: {} / colours {} / fonts {} + {}. Existing version: {}. Original brief: {}. Attachments available: {}",POSTER_ART_DIRECTION_SKILL,POSTER_QUALITY_GATE,plan.facts,plan.hierarchy,plan.reference_reading,plan.visual_strategy,plan.avoid,brand.name,serde_json::to_string(&brand.colors).unwrap_or_default(),brand.display_font,brand.body_font,base.map(|x|serde_json::to_string(x).unwrap_or_default()).unwrap_or_else(||"none".into()),prompt,attachments.len())
}
fn parse_creative_plan(raw: &str) -> Result<CreativePlan, String> {
    let candidate = raw
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<CreativePlan>(line).ok())
        .or_else(|| serde_json::from_str::<CreativePlan>(raw).ok())
        .or_else(|| {
            let a = raw.find('{')?;
            let b = raw.rfind('}')?;
            serde_json::from_str(&raw[a..=b]).ok()
        })
        .ok_or(
            "The creative-analysis stage did not return a usable plan. Try again.".to_string(),
        )?;
    if candidate.facts.is_empty() || candidate.hierarchy.is_empty() {
        return Err(
            "The creative-analysis stage did not extract the required poster information.".into(),
        );
    }
    Ok(candidate)
}
fn parse_spec(raw: &str) -> Result<(String, VisualSpec), String> {
    let candidate = raw
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line).ok())
        .or_else(|| serde_json::from_str::<Value>(raw).ok())
        .or_else(|| {
            let a = raw.find('{')?;
            let b = raw.rfind('}')?;
            serde_json::from_str(&raw[a..=b]).ok()
        })
        .ok_or(
            "The provider did not return the expected visual JSON. Try again with a shorter brief."
                .to_string(),
        )?;
    let label = candidate
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("New visual direction")
        .to_string();
    let headline = candidate
        .get("headline")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .filter(|s| is_public_poster_copy(s))
        .ok_or("The provider response is missing a headline.")?
        .to_string();
    Ok((
        label,
        VisualSpec {
            headline,
            image_data: String::new(),
            kicker: candidate
                .get("kicker")
                .and_then(Value::as_str)
                .map(public_copy)
                .unwrap_or_default(),
            subline: candidate
                .get("subline")
                .and_then(Value::as_str)
                .map(public_copy)
                .unwrap_or_default(),
            details: candidate
                .get("details")
                .and_then(Value::as_array)
                .map(|xs| {
                    xs.iter()
                        .filter_map(Value::as_str)
                        .filter(|value| is_public_poster_copy(value))
                        .map(str::to_string)
                        .take(6)
                        .collect()
                })
                .unwrap_or_default(),
            cta: candidate
                .get("cta")
                .and_then(Value::as_str)
                .map(public_copy)
                .unwrap_or_default(),
            footer: candidate
                .get("footer")
                .and_then(Value::as_str)
                .map(public_copy)
                .unwrap_or_default(),
            composition: choice(
                &candidate,
                "composition",
                &["editorial", "event", "split", "typographic"],
                "editorial",
            ),
            reference_note: candidate
                .get("reference_note")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            scheme: choice(
                &candidate,
                "scheme",
                &["light", "dark", "accent", "secondary"],
                "light",
            ),
            alignment: choice(&candidate, "alignment", &["left", "center"], "left"),
            title_size: candidate
                .get("title_size")
                .and_then(Value::as_u64)
                .unwrap_or(72)
                .clamp(38, 180) as u32,
            shape: candidate
                .get("shape")
                .and_then(Value::as_str)
                .unwrap_or("circle")
                .to_string(),
        },
    ))
}
fn choice(v: &Value, key: &str, options: &[&str], fallback: &str) -> String {
    let x = v.get(key).and_then(Value::as_str).unwrap_or(fallback);
    if options.contains(&x) {
        x.to_string()
    } else {
        fallback.to_string()
    }
}

fn public_copy(value: &str) -> String {
    if is_public_poster_copy(value) {
        value.trim().to_owned()
    } else {
        String::new()
    }
}

fn is_public_poster_copy(value: &str) -> bool {
    let lower = value.to_lowercase();
    !value.trim().is_empty()
        && !value.contains('#')
        && ![
            "couleur",
            "color",
            "palette",
            "typograph",
            "police",
            "font",
            "archivo",
            "onest",
            "hex",
            "accent —",
            "fond —",
            "texte —",
        ]
        .iter()
        .any(|token| lower.contains(token))
}

fn parse_brand_kit(raw: &str, name: &str) -> Result<BrandKit, String> {
    let object = raw
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line).ok())
        .or_else(|| serde_json::from_str::<Value>(raw).ok())
        .or_else(|| {
            let start = raw.find('{')?;
            let end = raw.rfind('}')?;
            serde_json::from_str(&raw[start..=end]).ok()
        })
        .ok_or(
            "The provider did not return the expected brand-direction JSON. Try again.".to_string(),
        )?;
    let colors = object
        .get("colors")
        .and_then(Value::as_array)
        .ok_or("The provider response is missing its colour palette.")?
        .iter()
        .map(|item| {
            let hex = item
                .get("hex")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_uppercase();
            if !hex.starts_with('#')
                || hex.len() != 7
                || !hex.chars().skip(1).all(|x| x.is_ascii_hexdigit())
            {
                return Err("The provider returned an invalid colour value.".to_string());
            }
            Ok(crate::domain::models::BrandColor {
                name: item
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("Colour")
                    .to_string(),
                hex,
                role: item
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("Palette")
                    .to_string(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if colors.len() < 3 {
        return Err("The provider returned too few colours. Try again.".into());
    }
    let strings = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    Ok(BrandKit {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.trim().to_string(),
        sector: object
            .get("sector")
            .and_then(Value::as_str)
            .unwrap_or("New artistic direction")
            .to_string(),
        description: object
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("A proposed starting direction, ready to refine.")
            .to_string(),
        colors,
        display_font: object
            .get("display_font")
            .and_then(Value::as_str)
            .unwrap_or("Onest")
            .to_string(),
        body_font: object
            .get("body_font")
            .and_then(Value::as_str)
            .unwrap_or("Onest")
            .to_string(),
        tone: strings("tone"),
        dos: strings("dos"),
        donts: strings("donts"),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_brand_metadata_from_public_poster_copy() {
        let raw = r#"{"label":"Poster","headline":"Call for speakers","kicker":"GDG","subline":"Share your expertise","details":["Deadline · 25 October","Fond — #F2EBDD","Archivo / Onest"],"cta":"Apply now","footer":"GDG Dakar","composition":"event","scheme":"light","alignment":"left","title_size":120}"#;
        let (_, spec) = parse_spec(raw).expect("valid public poster copy");
        assert_eq!(spec.details, vec!["Deadline · 25 October"]);
        assert_eq!(spec.cta, "Apply now");
    }

    #[test]
    fn rejects_technical_headlines() {
        let raw = r##"{"headline":"#F2EBDD","composition":"event"}"##;
        assert!(parse_spec(raw).is_err());
    }
}
