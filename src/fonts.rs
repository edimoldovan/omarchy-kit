//! Font setup.
//!
//! egui's proportional fonts cover Latin text and a small emoji set, but not
//! every geometric and technical symbol an app shows. Append the system's
//! symbol fonts, then egui's own Hack, as fallbacks: the symbols on the
//! README's list then draw everywhere, with no system font installed.

use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

/// Symbol fonts to append, in priority order: the Linux system paths, then
/// macOS's own symbol font. Missing files are skipped, so this degrades
/// quietly on any system.
const CANDIDATES: &[&str] = &[
    "/usr/share/fonts/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/truetype/noto/NotoSansSymbols2-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/truetype/noto/NotoSansSymbols-Regular.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/System/Library/Fonts/Apple Symbols.ttf",
];

/// Installs symbol fallbacks. Returns the names of the fonts actually loaded.
pub fn install(ctx: &Context) -> Vec<String> {
    let (fonts, loaded) = definitions(CANDIDATES);
    ctx.set_fonts(fonts);
    loaded
}

/// egui's fonts with the candidates that exist behind them, and egui's
/// bundled Hack last behind the proportional face, so its symbols are there
/// where no system font is.
fn definitions(candidates: &[&str]) -> (FontDefinitions, Vec<String>) {
    let mut fonts = FontDefinitions::default();
    let mut loaded = Vec::new();

    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let name = std::path::Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| (*path).to_string());
        if fonts.font_data.contains_key(&name) {
            continue;
        }
        fonts
            .font_data
            .insert(name.clone(), Arc::new(FontData::from_owned(bytes)));
        // Fallbacks go last so the bundled font still wins for Latin text.
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            if let Some(list) = fonts.families.get_mut(&family) {
                list.push(name.clone());
            }
        }
        loaded.push(name);
    }
    if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
        list.push("Hack".to_string());
    }
    (fonts, loaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::FontId;

    /// The symbols an app may draw as text: each is in the bundled fonts,
    /// so it draws where no system font is installed.
    const SYMBOLS: &str = "×✔…—–·•←→↑↓↖↗↘↙⇆⇄⇔⇥⇩⇧↵▾▴▸◂▲▼◀▶◆◇●○■□⌘⏸⏮⏭⏪⏩✂⊢⊣°±µ½∩∞≈≠≤≥✚➕";

    /// With no system font at all, every symbol on the list draws.
    #[test]
    fn the_listed_symbols_draw_with_no_system_fonts() {
        let ctx = Context::default();
        ctx.set_fonts(definitions(&[]).0);
        let _ = ctx.run(Default::default(), |_| {});
        let missing: String = SYMBOLS
            .chars()
            .filter(|&c| !ctx.fonts_mut(|f| f.has_glyph(&FontId::proportional(14.0), c)))
            .collect();
        assert!(missing.is_empty(), "no bundled font has {missing}");
    }
}
