#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WindowKind {
    Splash,
    Launcher,
    Preview,
    Timeline,
    Properties,
    SystemSettings,
    ProjectSettings,
    SceneSettings,
    Keybindings,
    Export,
    EffectAdd,
    EasingEditor,
}

impl WindowKind {
    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Splash => "NeoUtl",
            Self::Launcher => "NeoUtl - プロジェクトランチャー",
            Self::Preview => "NeoUtl",
            Self::Timeline => "NeoUtl - 拡張編集",
            Self::Properties => "NeoUtl - オブジェクト設定",
            Self::SystemSettings => "NeoUtl - システム設定",
            Self::ProjectSettings => "プロジェクト設定",
            Self::SceneSettings => "シーン設定",
            Self::Keybindings => "ショートカット設定",
            Self::Export => "メディアの書き出し",
            Self::EffectAdd => "エフェクト追加",
            Self::EasingEditor => "NeoUtl - イージング編集",
        }
    }

    pub(super) fn size(self) -> (u32, u32) {
        match self {
            Self::Splash => (0, 0),
            Self::Launcher => (860, 640),
            Self::Preview | Self::Timeline | Self::Properties => (720, 540),
            Self::SystemSettings => (720, 540),
            Self::ProjectSettings => (520, 360),
            Self::SceneSettings => (520, 700),
            Self::Keybindings => (720, 540),
            Self::Export => (620, 560),
            Self::EffectAdd => (420, 560),
            Self::EasingEditor => (580, 460),
        }
    }

    pub(super) fn min_size(self) -> Option<(u32, u32)> {
        match self {
            Self::Launcher => Some((720, 520)),
            Self::EffectAdd => Some((400, 240)),
            Self::SceneSettings => Some((600, 240)),
            Self::SystemSettings => Some((680, 240)),
            _ => None,
        }
    }

    pub(super) fn is_lazy_dialog(self) -> bool {
        matches!(
            self,
            Self::SystemSettings
                | Self::ProjectSettings
                | Self::SceneSettings
                | Self::Keybindings
                | Self::Export
                | Self::EffectAdd
                | Self::EasingEditor
        )
    }
}
