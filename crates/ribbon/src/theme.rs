use gpui::App;

use gpui_base::SemanticThemeTokens;

pub(crate) fn tokens(cx: &App) -> SemanticThemeTokens {
    gpui_base::Theme::global(cx).tokens
}
