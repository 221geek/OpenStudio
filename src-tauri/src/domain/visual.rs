use super::models::{BrandKit, VisualSpec};

pub fn render_svg(spec: &VisualSpec, brand: &BrandKit, format: &str) -> Result<String, String> {
    let (width, height) = match format {
        "story" => (1080, 1920),
        "square" => (1080, 1080),
        "landscape" => (1920, 1080),
        _ => (1080, 1350),
    };
    if !spec.image_data.is_empty() {
        return Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}"><image href="{}" width="{width}" height="{height}" preserveAspectRatio="xMidYMid slice"/></svg>"#,
            escape(&spec.image_data)
        ));
    }
    let colors: Vec<&str> = brand
        .colors
        .iter()
        .map(|color| color.hex.as_str())
        .collect();
    if colors.len() < 3 {
        return Err("A brand kit requires at least three colors.".to_string());
    }
    let (background, foreground, accent, secondary) = match spec.scheme.as_str() {
        "dark" => (
            colors[1],
            colors[0],
            colors[2],
            colors.get(3).copied().unwrap_or(colors[0]),
        ),
        "accent" => (
            colors[2],
            colors[1],
            colors[0],
            colors.get(3).copied().unwrap_or(colors[1]),
        ),
        "secondary" if colors.len() > 3 => (colors[3], colors[1], colors[2], colors[0]),
        _ => (
            colors[0],
            colors[1],
            colors[2],
            colors.get(3).copied().unwrap_or(colors[2]),
        ),
    };
    let pad = (width as f32 * 0.075) as u32;
    let is_typographic = spec.composition == "typographic";
    let centered = is_typographic && spec.alignment == "center";
    let left = if centered { width / 2 } else { pad };
    let anchor = if centered { "middle" } else { "start" };
    let title_lines = wrap_headline(&spec.headline, if width > height { 20 } else { 14 });
    let line_count = title_lines.len().min(3) as u32;
    let title_size = spec.title_size.clamp(64, 154).min((width / 7).max(64));
    let title_y = match spec.composition.as_str() {
        "event" => height * 35 / 100,
        "split" => height * 40 / 100,
        "typographic" => height * 43 / 100,
        _ => height * 42 / 100,
    };
    let title = title_lines
        .iter()
        .take(3)
        .enumerate()
        .map(|(index, line)| {
            format!(
                "<tspan x=\"{left}\" dy=\"{}\">{}</tspan>",
                if index == 0 { 0 } else { title_size * 88 / 100 },
                escape(line)
            )
        })
        .collect::<String>();
    let title_block = title_size * line_count;
    let details = spec.details.iter().filter(|value| is_public_copy(value)).take(3).enumerate().map(|(index, detail)| {
        let y = title_y + title_block + 124 + index as u32 * 36;
        format!("<text x=\"{left}\" y=\"{y}\" text-anchor=\"{anchor}\" font-family=\"{body}\" font-size=\"24\" font-weight=\"500\">{}</text>", escape(detail), body=escape(&brand.body_font))
    }).collect::<String>();
    let kicker = public_or_empty(&spec.kicker);
    let subline = public_or_empty(&spec.subline);
    let cta = public_or_empty(&spec.cta);
    let footer = public_or_empty(&spec.footer);
    let top_label = if kicker.is_empty() {
        escape(&brand.name)
    } else {
        escape(&kicker)
    };
    let safe_footer = if footer.is_empty() {
        escape(&brand.name)
    } else {
        escape(&footer)
    };
    let separator_y = height - pad;
    let decor = decorations(
        &spec.composition,
        width,
        height,
        pad,
        accent,
        secondary,
        background,
    );
    let subline_y = title_y + title_block + 66;
    let cta_y = height - pad - 54;
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="100%" height="100%" fill="{background}"/>{decor}
<g fill="{foreground}">
  <text x="{pad}" y="{pad}" font-family="{display}" font-size="22" font-weight="700" letter-spacing="2">{brand_name}</text>
  <line x1="{pad}" y1="{}" x2="{}" y2="{}" stroke="{foreground}" stroke-opacity=".28"/>
  <text x="{pad}" y="{}" font-family="{body}" font-size="18" font-weight="700" letter-spacing="2">{top_label}</text>
  <text x="{left}" y="{title_y}" text-anchor="{anchor}" font-family="{display}" font-size="{title_size}" font-weight="800" letter-spacing="-4">{title}</text>
  <text x="{left}" y="{subline_y}" text-anchor="{anchor}" font-family="{body}" font-size="30" font-weight="500" opacity=".82">{subline}</text>
  {details}
  <text x="{left}" y="{cta_y}" text-anchor="{anchor}" font-family="{display}" font-size="26" font-weight="700">{cta}</text>
  <line x1="{pad}" y1="{separator_y}" x2="{}" y2="{separator_y}" stroke="{foreground}" stroke-opacity=".28"/>
  <text x="{pad}" y="{}" font-family="{body}" font-size="18" font-weight="600">{safe_footer}</text>
  <text x="{}" y="{}" text-anchor="end" font-family="{body}" font-size="18" font-weight="600">{ratio}</text>
</g></svg>"#,
        height - pad - 54,
        width - pad,
        height - pad - 54,
        height - pad - 70,
        width - pad,
        height - pad + 36,
        width - pad,
        height - pad + 36,
        brand_name = escape(&brand.name),
        display = escape(&brand.display_font),
        body = escape(&brand.body_font),
        ratio = ratio(format),
    ))
}

fn decorations(
    composition: &str,
    width: u32,
    height: u32,
    pad: u32,
    accent: &str,
    secondary: &str,
    background: &str,
) -> String {
    match composition {
        "event" => format!(
            r#"<rect x="{}" y="0" width="{}" height="{height}" fill="{accent}"/><path d="M{} 0 L{width} {}" stroke="{background}" stroke-width="12" opacity=".78"/><circle cx="{}" cy="{}" r="{}" fill="none" stroke="{background}" stroke-width="18" opacity=".72"/>"#,
            width * 72 / 100,
            width * 28 / 100,
            width * 72 / 100,
            height * 25 / 100,
            width * 86 / 100,
            height * 24 / 100,
            width / 11
        ),
        "split" => format!(
            r#"<rect x="{}" y="0" width="{}" height="{height}" fill="{accent}"/><rect x="{}" y="{}" width="{}" height="{}" fill="{secondary}" opacity=".44"/><path d="M{} {} H{width}" stroke="{background}" stroke-width="12" opacity=".82"/>"#,
            width * 61 / 100,
            width * 39 / 100,
            width * 69 / 100,
            height * 13 / 100,
            width * 24 / 100,
            height * 42 / 100,
            width * 61 / 100,
            height * 72 / 100
        ),
        "typographic" => format!(
            r#"<text x="{}" y="{}" font-family="Arial, sans-serif" font-size="{}" font-weight="800" fill="{accent}" opacity=".16">01</text><path d="M{pad} {} H{}" stroke="{accent}" stroke-width="16"/>"#,
            width * 60 / 100,
            height * 77 / 100,
            width / 2,
            height * 82 / 100,
            width - pad
        ),
        _ => format!(
            r#"<path d="M{pad} {} C{} {}, {} {}, {width} {}" fill="none" stroke="{accent}" stroke-width="{}" opacity=".9"/><rect x="{}" y="{}" width="{}" height="{}" fill="{secondary}" opacity=".32"/>"#,
            height * 78 / 100,
            width * 30 / 100,
            height * 58 / 100,
            width * 63 / 100,
            height * 91 / 100,
            height * 64 / 100,
            width / 30,
            width * 74 / 100,
            height * 16 / 100,
            width * 16 / 100,
            height * 18 / 100
        ),
    }
}

fn public_or_empty(value: &str) -> String {
    if is_public_copy(value) {
        value.trim().to_owned()
    } else {
        String::new()
    }
}

fn is_public_copy(value: &str) -> bool {
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

fn wrap_headline(value: &str, max_characters: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let next_length = current.len() + usize::from(!current.is_empty()) + word.len();
        if next_length > max_characters && !current.is_empty() {
            lines.push(current);
            current = word.to_owned();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        vec!["OpenStudio".to_owned()]
    } else {
        lines
    }
}

fn ratio(format: &str) -> &'static str {
    match format {
        "story" => "9:16",
        "square" => "1:1",
        "landscape" => "16:9",
        _ => "4:5",
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{BrandColor, BrandKit};

    fn brand() -> BrandKit {
        BrandKit {
            id: "b".into(),
            name: "Studio & Co".into(),
            sector: "".into(),
            description: "".into(),
            colors: vec![
                BrandColor {
                    name: "".into(),
                    hex: "#eee".into(),
                    role: "".into(),
                },
                BrandColor {
                    name: "".into(),
                    hex: "#111".into(),
                    role: "".into(),
                },
                BrandColor {
                    name: "".into(),
                    hex: "#f60".into(),
                    role: "".into(),
                },
            ],
            display_font: "Arial".into(),
            body_font: "Arial".into(),
            tone: vec![],
            dos: vec![],
            donts: vec![],
            created_at: "".into(),
        }
    }
    fn spec() -> VisualSpec {
        VisualSpec {
            headline: "Call for speakers".into(),
            image_data: "".into(),
            kicker: "GDG Dakar".into(),
            subline: "Share your expertise".into(),
            details: vec!["Deadline · 25 October".into(), "Fond — #F2EBDD".into()],
            cta: "Apply now".into(),
            footer: "GDG Dakar".into(),
            composition: "event".into(),
            reference_note: "".into(),
            scheme: "light".into(),
            alignment: "left".into(),
            title_size: 132,
            shape: "circle".into(),
        }
    }
    #[test]
    fn renders_an_editorial_svg_without_brand_metadata_as_copy() {
        let svg = render_svg(&spec(), &brand(), "post").unwrap();
        assert!(svg.contains("Call for"));
        assert!(svg.contains("Deadline · 25 October"));
        assert!(!svg.contains("F2EBDD"));
        assert!(svg.contains("Studio &amp; Co"));
    }
    #[test]
    fn wraps_headlines_without_breaking_words() {
        assert_eq!(
            wrap_headline("CALL FOR SPEAKERS TODAY", 14),
            vec!["CALL FOR".to_string(), "SPEAKERS TODAY".to_string()]
        );
    }
}
