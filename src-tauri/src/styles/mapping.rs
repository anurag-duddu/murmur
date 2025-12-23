//! App-to-style mapping.
//!
//! Maps application bundle IDs to appropriate styles.
//! Includes 100+ app mappings and categorical inference for unknown apps.

use super::{builtin, ActiveApp, Style};

/// Default app-to-style mappings by bundle ID.
/// Organized by category for maintainability.
const DEFAULT_MAPPINGS: &[(&str, &str)] = &[
    // =========================================================================
    // CASUAL - Messaging & Chat Apps
    // =========================================================================
    // Apple
    ("com.apple.MobileSMS", "casual"),              // Messages
    ("com.apple.iChat", "casual"),                  // iChat (legacy)

    // Slack & Discord
    ("com.tinyspeck.slackmacgap", "casual"),        // Slack
    ("com.hnc.Discord", "casual"),                  // Discord

    // Meta/Facebook
    ("com.facebook.Messenger", "casual"),           // Messenger
    ("net.whatsapp.WhatsApp", "casual"),            // WhatsApp
    ("com.instagram.Instagram", "casual"),          // Instagram

    // Telegram
    ("org.telegram.desktop", "casual"),             // Telegram
    ("ru.keepcoder.Telegram", "casual"),            // Telegram (alternative)

    // Other Messaging
    ("com.viber.osx", "casual"),                    // Viber
    ("jp.naver.line.mac", "casual"),                // LINE
    ("com.wechat.WeChat", "casual"),                // WeChat
    ("org.pqrs.Signal", "casual"),                  // Signal (variant)
    ("org.whispersystems.signal-desktop", "casual"), // Signal
    ("com.skype.skype", "casual"),                  // Skype (personal)
    ("im.riot.app", "casual"),                      // Element/Matrix
    ("com.wire.Wire", "casual"),                    // Wire
    ("com.threema.threema", "casual"),              // Threema
    ("com.keybase.Keybase", "casual"),              // Keybase

    // Video Conferencing (Casual context - chat/messaging in these apps)
    ("us.zoom.xos", "casual"),                      // Zoom
    ("com.microsoft.teams", "casual"),              // Microsoft Teams
    ("com.microsoft.teams2", "casual"),             // Microsoft Teams (new)
    ("com.google.meet", "casual"),                  // Google Meet
    ("com.cisco.webex", "casual"),                  // Webex
    ("com.cisco.webexmeetings", "casual"),          // Webex Meetings
    ("com.logmein.GoToMeeting", "casual"),          // GoToMeeting
    ("com.loom.desktop", "casual"),                 // Loom
    ("com.around.Around", "casual"),                // Around

    // =========================================================================
    // PROFESSIONAL - Email & Business Communication
    // =========================================================================
    // Apple
    ("com.apple.mail", "professional"),             // Apple Mail

    // Microsoft
    ("com.microsoft.Outlook", "professional"),      // Outlook

    // Third-party Email
    ("com.readdle.smartemail-Mac", "professional"), // Spark
    ("com.superhuman.Superhuman", "professional"),  // Superhuman
    ("com.freron.MailMate", "professional"),        // MailMate
    ("com.postbox-inc.postbox", "professional"),    // Postbox
    ("com.canary-mail.Canary-Mail", "professional"), // Canary Mail
    ("com.nylas.nylas-mail", "professional"),       // Nylas Mail
    ("com.mimestream.Mimestream", "professional"),  // Mimestream
    ("com.airmail.airmail", "professional"),        // Airmail
    ("com.newton.Newton", "professional"),          // Newton Mail
    ("me.protonmail.ProtonMail", "professional"),   // ProtonMail
    ("io.tutanota.tutanota", "professional"),       // Tutanota

    // Business/Professional Apps
    ("com.linkedin.LinkedIn", "professional"),      // LinkedIn
    ("com.linkedin.linkedin-sales-navigator-osx", "professional"), // LinkedIn Sales Navigator
    ("com.salesforce.salesforce", "professional"),  // Salesforce
    ("com.hubspot.HubSpot", "professional"),        // HubSpot

    // =========================================================================
    // TECHNICAL - Development & Engineering
    // =========================================================================
    // Terminals
    ("com.apple.Terminal", "technical"),            // Terminal
    ("com.googlecode.iterm2", "technical"),         // iTerm2
    ("dev.warp.Warp-Stable", "technical"),          // Warp
    ("co.zeit.hyper", "technical"),                 // Hyper
    ("net.kovidgoyal.kitty", "technical"),          // kitty
    ("io.alacritty", "technical"),                  // Alacritty
    ("com.tabby.Tabby", "technical"),               // Tabby
    ("com.panic.Prompt", "technical"),              // Prompt
    ("com.panic.Prompt3", "technical"),             // Prompt 3
    ("org.gnu.Emacs", "technical"),                 // Emacs
    ("org.vim.MacVim", "technical"),                // MacVim
    ("com.qvacua.VimR", "technical"),               // VimR

    // Code Editors
    ("com.microsoft.VSCode", "technical"),          // VS Code
    ("com.microsoft.VSCodeInsiders", "technical"),  // VS Code Insiders
    ("com.todesktop.230313mzl4w4u92", "technical"), // Cursor
    ("com.codeium.windsurf", "technical"),          // Windsurf
    ("com.sublimetext.4", "technical"),             // Sublime Text 4
    ("com.sublimetext.3", "technical"),             // Sublime Text 3
    ("com.panic.Nova", "technical"),                // Nova
    ("com.barebones.bbedit", "technical"),          // BBEdit
    ("com.coteditor.CotEditor", "technical"),       // CotEditor
    ("abnerworks.Typora", "technical"),             // Typora (Markdown)
    ("com.github.atom", "technical"),               // Atom (legacy)
    ("org.eclipse.eclipse", "technical"),           // Eclipse
    ("com.neovide.neovide", "technical"),           // Neovide

    // IDEs - Apple
    ("com.apple.dt.Xcode", "technical"),            // Xcode
    ("com.apple.Playgrounds", "technical"),         // Swift Playgrounds

    // IDEs - JetBrains
    ("com.jetbrains.intellij", "technical"),        // IntelliJ IDEA
    ("com.jetbrains.intellij.ce", "technical"),     // IntelliJ IDEA CE
    ("com.jetbrains.pycharm", "technical"),         // PyCharm
    ("com.jetbrains.pycharm.ce", "technical"),      // PyCharm CE
    ("com.jetbrains.WebStorm", "technical"),        // WebStorm
    ("com.jetbrains.PhpStorm", "technical"),        // PhpStorm
    ("com.jetbrains.CLion", "technical"),           // CLion
    ("com.jetbrains.GoLand", "technical"),          // GoLand
    ("com.jetbrains.RubyMine", "technical"),        // RubyMine
    ("com.jetbrains.rider", "technical"),           // Rider
    ("com.jetbrains.AppCode", "technical"),         // AppCode
    ("com.jetbrains.datagrip", "technical"),        // DataGrip
    ("com.jetbrains.fleet", "technical"),           // Fleet

    // IDEs - Other
    ("com.google.android.studio", "technical"),     // Android Studio
    ("com.visualstudio.code.oss", "technical"),     // VSCodium

    // Database & API Tools
    ("com.sequelpro.SequelPro", "technical"),       // Sequel Pro
    ("com.tinyapp.TablePlus", "technical"),         // TablePlus
    ("com.dbeaver.product", "technical"),           // DBeaver
    ("com.insomnia.Insomnia", "technical"),         // Insomnia
    ("com.postmanlabs.Postman", "technical"),       // Postman
    ("com.paw.Paw", "technical"),                   // Paw/RapidAPI
    ("com.apple.CoreData.lab", "technical"),        // Core Data Lab
    ("com.mongodb.compass", "technical"),           // MongoDB Compass
    ("com.redis.RedisInsight", "technical"),        // Redis Insight

    // DevOps & Infrastructure
    ("com.docker.docker", "technical"),             // Docker Desktop
    ("com.electron.fork", "technical"),             // Fork (Git)
    ("com.git.Tower", "technical"),                 // Tower
    ("com.sourcetreeapp.SourceTree", "technical"),  // SourceTree
    ("com.github.GitHubDesktop", "technical"),      // GitHub Desktop
    ("com.sublimemerge.Sublime-Merge", "technical"), // Sublime Merge
    ("com.gitkraken.gitkraken", "technical"),       // GitKraken
    // Note: Typora is already listed above at line 109
    ("io.podman-desktop.PodmanDesktop", "technical"), // Podman Desktop

    // Generic Electron (fallback)
    ("com.github.Electron", "technical"),           // Electron apps

    // =========================================================================
    // NEUTRAL - Notes, Browsers & General Purpose
    // =========================================================================
    // Apple Notes & TextEdit
    ("com.apple.Notes", "neutral"),                 // Apple Notes
    ("com.apple.TextEdit", "neutral"),              // TextEdit
    ("com.apple.Preview", "neutral"),               // Preview
    ("com.apple.finder", "neutral"),                // Finder

    // Note-Taking Apps
    ("notion.id", "neutral"),                       // Notion
    ("md.obsidian", "neutral"),                     // Obsidian
    ("com.evernote.Evernote", "neutral"),           // Evernote
    ("com.couchbase.cblite.mindnode", "neutral"),   // MindNode
    ("com.toketaware.mindnodepro", "neutral"),      // MindNode Pro
    ("com.craft.craft", "neutral"),                 // Craft
    ("com.lukilabs.lukiapp", "neutral"),            // Luki
    ("com.agenda.Agenda", "neutral"),               // Agenda
    ("com.apple.reminders", "neutral"),             // Reminders
    ("com.goodlinks.GoodLinks", "neutral"),         // GoodLinks
    ("com.raywenderlich.roost", "neutral"),         // Roost

    // Browsers (neutral - can't detect web app context)
    ("com.google.Chrome", "neutral"),               // Chrome
    ("com.google.Chrome.canary", "neutral"),        // Chrome Canary
    ("com.apple.Safari", "neutral"),                // Safari
    ("org.mozilla.firefox", "neutral"),             // Firefox
    ("org.mozilla.firefoxdeveloperedition", "neutral"), // Firefox Developer
    ("company.thebrowser.Browser", "neutral"),      // Arc
    ("com.microsoft.edgemac", "neutral"),           // Edge
    ("com.brave.Browser", "neutral"),               // Brave
    ("com.vivaldi.Vivaldi", "neutral"),             // Vivaldi
    ("com.operasoftware.Opera", "neutral"),         // Opera
    ("org.chromium.Chromium", "neutral"),           // Chromium
    ("com.sigmaos.sigmaos", "neutral"),             // SigmaOS

    // Productivity & Project Management
    ("com.asana.asana", "neutral"),                 // Asana
    ("com.trello.Trello", "neutral"),               // Trello
    ("com.monday.monday", "neutral"),               // Monday.com
    ("com.todoist.mac.Todoist", "neutral"),         // Todoist
    ("com.omnigroup.OmniFocus3", "neutral"),        // OmniFocus
    ("com.culturedcode.ThingsMac3", "neutral"),     // Things 3
    ("com.ticktick.task.mac", "neutral"),           // TickTick
    ("com.wunderbucket.WunderlistMac", "neutral"),  // Wunderlist (legacy)
    ("com.apple.freeform", "neutral"),              // Freeform
    ("com.miro.Miro", "neutral"),                   // Miro
    ("com.fibery.Fibery", "neutral"),               // Fibery
    ("com.airtableapp.Airtable", "neutral"),        // Airtable
    ("com.coda.coda", "neutral"),                   // Coda
    ("com.clickup.desktop-app", "neutral"),         // ClickUp

    // Spreadsheets & Data
    ("com.microsoft.Excel", "neutral"),             // Excel
    ("com.apple.iWork.Numbers", "neutral"),         // Numbers
    ("com.google.sheets", "neutral"),               // Google Sheets (app)

    // Presentations (Neutral for notes/general use)
    ("com.microsoft.PowerPoint", "neutral"),        // PowerPoint
    ("com.apple.iWork.Keynote", "neutral"),         // Keynote

    // Microsoft Office Suite (misc)
    ("com.microsoft.OneNote", "neutral"),           // OneNote
    ("com.microsoft.onenote.mac", "neutral"),       // OneNote (alternative)
    ("com.microsoft.To-Do", "neutral"),             // Microsoft To Do

    // =========================================================================
    // CREATIVE - Writing & Design
    // =========================================================================
    // Word Processors
    ("com.apple.iWork.Pages", "creative"),          // Pages
    ("com.microsoft.Word", "creative"),             // Word
    ("com.google.android.apps.docs", "creative"),   // Google Docs (app)
    ("org.libreoffice.libreoffice", "creative"),    // LibreOffice
    ("org.openoffice.calc", "creative"),            // OpenOffice

    // Writing Apps
    ("com.ulyssesapp.mac", "creative"),             // Ulysses
    ("com.shinyfrog.bear", "creative"),             // Bear
    ("pro.writer.mac", "creative"),                 // iA Writer
    ("com.iawriter.mac", "creative"),               // iA Writer (alternative)
    ("com.literatureandlatte.scrivener3", "creative"), // Scrivener
    ("com.bloombuilt.dayone-mac", "creative"),      // Day One
    ("com.bywordapp.Byword", "creative"),           // Byword
    ("com.red-sweater.marsedit4", "creative"),      // MarsEdit
    ("com.drafts.Drafts", "creative"),              // Drafts
    ("com.agiletortoise.Drafts", "creative"),       // Drafts (alternative)
    ("com.omnigroup.OmniOutliner5", "creative"),    // OmniOutliner
    ("com.apple.garageband", "creative"),           // GarageBand

    // Design Tools
    ("com.figma.Desktop", "creative"),              // Figma
    ("com.bohemiancoding.sketch3", "creative"),     // Sketch
    ("com.adobe.illustrator", "creative"),          // Illustrator
    ("com.adobe.Photoshop", "creative"),            // Photoshop
    ("com.adobe.InDesign", "creative"),             // InDesign
    ("com.adobe.AfterEffects", "creative"),         // After Effects
    ("com.adobe.Premiere", "creative"),             // Premiere Pro
    ("com.adobe.xd", "creative"),                   // Adobe XD
    ("com.adobe.Lightroom", "creative"),            // Lightroom
    ("com.adobe.LightroomClassic", "creative"),     // Lightroom Classic
    ("com.affinity.designer", "creative"),          // Affinity Designer
    ("com.affinity.photo", "creative"),             // Affinity Photo
    ("com.affinity.publisher", "creative"),         // Affinity Publisher
    ("com.pixelmator.Pixelmator-Pro", "creative"),  // Pixelmator Pro
    ("com.canva.CanvaDesktop", "creative"),         // Canva
    ("com.icons8.Lunacy", "creative"),              // Lunacy
    ("com.invisionapp.studio", "creative"),         // InVision Studio
    ("com.principle.Principle", "creative"),        // Principle
    ("com.framer.Framer", "creative"),              // Framer
    ("io.penpot.desktop", "creative"),              // Penpot

    // Video & Audio
    ("com.apple.FinalCut", "creative"),             // Final Cut Pro
    ("com.apple.iMovieApp", "creative"),            // iMovie
    ("com.apple.Logic", "creative"),                // Logic Pro
    ("com.blackmagic-design.DaVinciResolve", "creative"), // DaVinci Resolve
    ("com.screenflow.ScreenFlow", "creative"),      // ScreenFlow
    ("com.techsmith.camtasia3", "creative"),        // Camtasia
    ("com.audacity.Audacity", "creative"),          // Audacity
];

/// Get the appropriate style for an application.
///
/// Uses a multi-tier approach:
/// 1. Exact bundle ID match from hardcoded mappings
/// 2. Categorical inference from app name keywords
/// 3. Falls back to neutral style
pub fn get_style_for_app(app: &ActiveApp) -> Style {
    // Tier 1: Exact bundle ID match
    if let Some(style_id) = get_style_id_for_bundle_id(&app.bundle_id) {
        if let Some(style) = builtin::get_style_by_id(style_id) {
            return style;
        }
    }

    // Tier 2: Categorical inference from app name and bundle ID
    if let Some(style_id) = infer_style_from_category(app) {
        if let Some(style) = builtin::get_style_by_id(style_id) {
            return style;
        }
    }

    // Tier 3: Fall back to default (neutral)
    builtin::get_default_style()
}

/// Get the style ID for a bundle ID from default mappings.
fn get_style_id_for_bundle_id(bundle_id: &str) -> Option<&'static str> {
    for (bid, style_id) in DEFAULT_MAPPINGS {
        if *bid == bundle_id {
            return Some(style_id);
        }
    }
    None
}

// =============================================================================
// CATEGORICAL INFERENCE
// =============================================================================

/// Keywords that indicate a specific style category.
/// Each tuple contains (keywords, style_id).
/// Keywords are matched case-insensitively against app name and bundle ID.

/// Messaging/Chat keywords → casual
const CASUAL_KEYWORDS: &[&str] = &[
    // Messaging
    "chat", "messenger", "message", "messaging", "msg",
    "slack", "discord", "telegram", "whatsapp", "signal",
    "viber", "wechat", "line", "imessage", "sms",
    // Social
    "social", "twitter", "mastodon", "bluesky", "threads",
    // Video chat (casual context)
    "zoom", "meet", "facetime", "webex", "teams",
    // Gaming/Community
    "gaming", "community", "guild",
];

/// Email/Business keywords → professional
const PROFESSIONAL_KEYWORDS: &[&str] = &[
    // Email
    "mail", "email", "outlook", "inbox", "smtp", "imap",
    "postbox", "airmail", "spark", "superhuman",
    // CRM/Sales
    "salesforce", "hubspot", "crm", "sales", "linkedin",
    "recruiter", "hiring", "hr", "talent",
    // Enterprise
    "enterprise", "corporate", "business", "invoice",
    "payroll", "accounting", "quickbooks", "freshbooks",
];

/// Development/Technical keywords → technical
const TECHNICAL_KEYWORDS: &[&str] = &[
    // Terminals
    "terminal", "term", "console", "shell", "bash", "zsh",
    "iterm", "hyper", "warp", "alacritty", "kitty",
    // Editors/IDEs
    "code", "editor", "ide", "studio", "vim", "emacs", "neovim",
    "vscode", "xcode", "intellij", "pycharm", "webstorm",
    "sublime", "atom", "nova", "bbedit", "cursor", "windsurf",
    // Development
    "dev", "developer", "develop", "debug", "debugger",
    "compiler", "build", "make", "cmake", "gradle",
    // Git/Version Control
    "git", "github", "gitlab", "bitbucket", "sourcetree",
    "tower", "fork", "gitkraken", "merge",
    // Database/API
    "database", "sql", "postgres", "mysql", "mongo", "redis",
    "api", "postman", "insomnia", "graphql", "rest",
    "sequel", "tableplus", "dbeaver", "datagrip",
    // DevOps/Infra
    "docker", "kubernetes", "k8s", "podman", "container",
    "terraform", "ansible", "jenkins", "ci", "deploy",
    // Languages (in bundle IDs)
    "python", "rust", "golang", "nodejs", "ruby", "java",
];

/// Writing/Creative keywords → creative
const CREATIVE_KEYWORDS: &[&str] = &[
    // Writing
    "writer", "writing", "write", "document", "doc", "word",
    "pages", "scrivener", "ulysses", "bear", "drafts", "byword",
    "novel", "story", "blog", "article", "journal", "diary",
    // Design
    "design", "designer", "figma", "sketch", "illustrator",
    "photoshop", "affinity", "canva", "draw", "paint", "art",
    "graphic", "layout", "prototype", "wireframe", "mockup",
    // Video/Audio
    "video", "movie", "film", "premiere", "finalcut", "davinci",
    "audio", "sound", "music", "podcast", "recording",
    "logic", "garageband", "audacity", "ableton", "pro tools",
    // 3D/Animation
    "3d", "blender", "cinema4d", "maya", "render", "animation",
];

/// Infer style from app name and bundle ID keywords.
///
/// Performs case-insensitive matching against category keywords.
/// Returns the first matching category, or None if no match.
fn infer_style_from_category(app: &ActiveApp) -> Option<&'static str> {
    let name_lower = app.name.to_lowercase();
    let bundle_lower = app.bundle_id.to_lowercase();

    // Helper to check if any keyword matches
    let matches_any = |keywords: &[&str]| -> bool {
        keywords.iter().any(|kw| {
            name_lower.contains(kw) || bundle_lower.contains(kw)
        })
    };

    // Check categories in order of specificity
    // Technical first (most specific keywords)
    if matches_any(TECHNICAL_KEYWORDS) {
        return Some("technical");
    }

    // Professional (email/business)
    if matches_any(PROFESSIONAL_KEYWORDS) {
        return Some("professional");
    }

    // Casual (messaging/social)
    if matches_any(CASUAL_KEYWORDS) {
        return Some("casual");
    }

    // Creative (writing/design)
    if matches_any(CREATIVE_KEYWORDS) {
        return Some("creative");
    }

    // No match - let caller fall back to default
    None
}

/// Get all default app mappings (for UI display).
pub fn get_all_mappings() -> Vec<(&'static str, &'static str)> {
    DEFAULT_MAPPINGS.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Tier 1: Exact bundle ID match tests
    // =========================================================================

    #[test]
    fn test_slack_is_casual() {
        let app = ActiveApp {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            name: "Slack".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "casual");
    }

    #[test]
    fn test_mail_is_professional() {
        let app = ActiveApp {
            bundle_id: "com.apple.mail".to_string(),
            name: "Mail".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "professional");
    }

    #[test]
    fn test_vscode_is_technical() {
        let app = ActiveApp {
            bundle_id: "com.microsoft.VSCode".to_string(),
            name: "Visual Studio Code".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "technical");
    }

    #[test]
    fn test_figma_is_creative() {
        let app = ActiveApp {
            bundle_id: "com.figma.Desktop".to_string(),
            name: "Figma".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "creative");
    }

    // =========================================================================
    // Tier 2: Categorical inference tests
    // =========================================================================

    #[test]
    fn test_infer_unknown_chat_app_as_casual() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.SuperChat".to_string(),
            name: "SuperChat Messenger".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "casual", "Chat app should be inferred as casual");
    }

    #[test]
    fn test_infer_unknown_mail_app_as_professional() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.FastMail".to_string(),
            name: "FastMail Pro".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "professional", "Mail app should be inferred as professional");
    }

    #[test]
    fn test_infer_unknown_terminal_as_technical() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.UltraTerminal".to_string(),
            name: "Ultra Terminal".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "technical", "Terminal app should be inferred as technical");
    }

    #[test]
    fn test_infer_unknown_ide_as_technical() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.CodeEditor".to_string(),
            name: "Super Code Editor".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "technical", "Code editor should be inferred as technical");
    }

    #[test]
    fn test_infer_unknown_design_app_as_creative() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.DesignTool".to_string(),
            name: "Amazing Designer Pro".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "creative", "Design app should be inferred as creative");
    }

    #[test]
    fn test_infer_music_app_as_creative() {
        let app = ActiveApp {
            bundle_id: "com.newstartup.Synthesizer".to_string(),
            name: "Podcast Recording".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "creative", "Podcast/music app should be inferred as creative");
    }

    #[test]
    fn test_infer_from_bundle_id_keyword() {
        let app = ActiveApp {
            bundle_id: "com.company.dockerhelper".to_string(),
            name: "Container Manager".to_string(), // Name doesn't have keywords
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "technical", "Should infer from 'docker' in bundle ID");
    }

    // =========================================================================
    // Tier 3: Fallback tests
    // =========================================================================

    #[test]
    fn test_truly_unknown_app_is_neutral() {
        let app = ActiveApp {
            bundle_id: "com.unknown.randomapp123".to_string(),
            name: "Random App".to_string(),
        };
        let style = get_style_for_app(&app);
        assert_eq!(style.id, "neutral", "Unknown app with no keywords should be neutral");
    }

    // =========================================================================
    // Validation tests
    // =========================================================================

    #[test]
    fn test_all_mappings_have_valid_styles() {
        for (bundle_id, style_id) in DEFAULT_MAPPINGS {
            assert!(
                builtin::get_style_by_id(style_id).is_some(),
                "Bundle {} maps to invalid style {}",
                bundle_id,
                style_id
            );
        }
    }

    #[test]
    fn test_mapping_count() {
        // Ensure we have a substantial number of mappings
        assert!(
            DEFAULT_MAPPINGS.len() >= 100,
            "Expected 100+ app mappings, got {}",
            DEFAULT_MAPPINGS.len()
        );
    }
}
