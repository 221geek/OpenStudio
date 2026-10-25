use super::models::{BrandKit, VisualSpec};

pub fn render_svg(spec: &VisualSpec, brand: &BrandKit, format: &str) -> Result<String, String> {
    let (width, height) = match format {
        "story" => (1080, 1920),
        "square" => (1080, 1080),
        "landscape" => (1920, 1080),
        _ => (1080, 1350),
    };
    let colors: Vec<&str> = brand.colors.iter().map(|c| c.hex.as_str()).collect();
    if colors.len() < 3 {
        return Err("A brand kit requires at least three colors.".to_string());
    }
    let (bg, fg, accent) = match spec.scheme.as_str() {
        "dark" => (colors[1], colors[0], colors[2]),
        "accent" => (colors[2], colors[1], colors[0]),
        "secondary" if colors.len() > 3 => (colors[3], colors[1], colors[0]),
        _ => (colors[0], colors[1], colors[2]),
    };
    let pad = (width as f32 * 0.075) as u32;
    let left = if spec.alignment == "center" {
        width / 2
    } else {
        pad
    };
    let anchor = if spec.alignment == "center" {
        "middle"
    } else {
        "start"
    };
    let font_size = spec.title_size.clamp(38, 180);
    let title_lines = wrap_headline(&spec.headline, 16);
    let title = title_lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            format!(
                "<tspan x=\"{left}\" dy=\"{}\">{}</tspan>",
                if index == 0 { 0 } else { font_size },
                escape(line)
            )
        })
        .collect::<String>();
    let title_block = font_size * title_lines.len() as u32;
    let title_y = height / 2 - title_block / 3;
    let kicker = escape(&spec.kicker);
    let subline = escape(&spec.subline);
    let name = escape(&brand.name);
    let details = spec
        .details
        .iter()
        .map(|detail| format!("<tspan x=\"{left}\" dy=\"34\">{}</tspan>", escape(detail)))
        .collect::<String>();
    let cta = escape(&spec.cta);
    let footer = escape(if spec.footer.is_empty() {
        "Créé avec OpenStudio"
    } else {
        &spec.footer
    });
    let decor = match spec.composition.as_str() {
        "event" => format!("<rect x=\"{}\" y=\"0\" width=\"{}\" height=\"{height}\" fill=\"{accent}\"/><path d=\"M{} 0 L{width} {}\" stroke=\"{bg}\" stroke-width=\"12\" opacity=\".8\"/>", width * 72 / 100, width * 28 / 100, width * 72 / 100, height * 28 / 100),
        "split" => format!("<path d=\"M{} 0 H{width} V{height} H{} Z\" fill=\"{accent}\"/>", width * 58 / 100, width * 42 / 100),
        "typographic" => format!("<rect x=\"{pad}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"18\" fill=\"{accent}\" opacity=\".18\"/>", height * 18 / 100, width - pad * 2, height * 64 / 100),
        _ => format!("<path d=\"M{} {} C{} {}, {} {}, {width} {}\" fill=\"none\" stroke=\"{accent}\" stroke-width=\"{}\" opacity=\".8\"/>", width * 8 / 100, height * 74 / 100, width * 34 / 100, height * 48 / 100, width * 62 / 100, height * 90 / 100, height * 30 / 100, width / 28),
    };
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
<rect width="100%" height="100%" fill="{bg}"/>{decor}
<g fill="{fg}"><text x="{pad}" y="{pad}" font-family="{display}" font-size="26" font-weight="700" letter-spacing="2">{name}</text>
<text x="{left}" y="{kicker_y}" text-anchor="{anchor}" font-family="{body}" font-size="22" font-weight="700" letter-spacing="2">{kicker}</text>
<text x="{left}" y="{}" text-anchor="{anchor}" font-family="{display}" font-size="{font_size}" font-weight="800" letter-spacing="-4">{title}</text>
<text x="{left}" y="{}" text-anchor="{anchor}" font-family="{body}" font-size="30" opacity=".85">{subline}</text>
<text x="{left}" y="{detail_y}" text-anchor="{anchor}" font-family="{body}" font-size="24">{details}</text>
<text x="{left}" y="{cta_y}" text-anchor="{anchor}" font-family="{display}" font-size="26" font-weight="700">{cta}</text>
<line x1="{pad}" y1="{}" x2="{}" y2="{}" stroke="{fg}" stroke-opacity=".25"/>
<text x="{pad}" y="{footer_y}" font-family="{body}" font-size="20">{footer}</text></g></svg>"#,
        title_y,
        title_y + title_block + 56,
        width - pad,
        height - 105,
        height - 105,
        kicker_y = title_y.saturating_sub(50),
        detail_y = title_y + title_block + 100,
        cta_y = height - 145,
        footer_y = height - 72,
        decor = decor,
        kicker = kicker,
        details = details,
        cta = cta,
        footer = footer,
        display = escape(&brand.display_font),
        body = escape(&brand.body_font)
    ))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{BrandColor, BrandKit};
    #[test]
    fn renders_a_safe_svg_with_brand_tokens() {
        let brand = BrandKit {
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
        };
        let spec = VisualSpec {
            headline: "A < B".into(),
            kicker: "".into(),
            subline: "".into(),
            details: vec![],
            cta: "".into(),
            footer: "".into(),
            composition: "editorial".into(),
            reference_note: "".into(),
            scheme: "dark".into(),
            alignment: "left".into(),
            title_size: 72,
            shape: "circle".into(),
        };
        let svg = render_svg(&spec, &brand, "post").unwrap();
        assert!(svg.contains("Studio &amp; Co"));
        assert!(svg.contains("A &lt; B"));
        assert!(svg.contains("fill=\"#111\""));
    }
}
