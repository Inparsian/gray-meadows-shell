use crate::widgets::windows::overview::{item::{OverviewSearchItem, OverviewSearchItemAction}, modules::OverviewSearchModule};

pub struct OverviewTerminalModule;

/// These commands aren't usually run in the background; if the command is one of these,
/// the items will be reversed to present "Run in terminal" first.
const COMMON_FOREGROUND_COMMANDS: &[&str] = &[
    "watch",

    // system info
    "neofetch",
    "fastfetch",
    "screenfetch",
    "inxi",

    // system monitoring
    "nmon",
    "glances",
    "iotop",
    "iftop",
    "htop",
    "btop",
    "bpytop",
    "ctop",
    "gotop",
    "nethogs",
    "vnstat",
    "powertop",

    // file managers
    "ranger",
    "nnn",
    "lf",
    "vifm",
    "mc",
    "yazi",

    // text editors
    "nano",
    "vim",
    "nvim", "neovim",
    "hx", "helix",
    "kak", "kakoune",
    "emacs",
    "micro",
    "ne",
    "joe",

    // disk & filesystem tools
    "ncdu",
    "fdisk",
    "cfdisk",
    "lsblk",
    "parted",
    "duf",

    // network & system configuration
    "nmtui",
    "alsamixer",
    "pulsemixer",
    "bluetuith",
    
    // media & entertainment
    "cmus",
    "ncmpcpp",
    "rmpc",
    "moc", "mocp",

    // communication
    "toot",
    "weechat",
    "irssi",
    "mutt", "neomutt",
];

impl OverviewSearchModule for OverviewTerminalModule {
    fn extensions(&self) -> Vec<&str> {
        vec!["terminal", "term", "cmd", "t", "$"]
    }

    fn icon(&self) -> &str {
        "terminal"
    }

    fn run(&self, query: &str) -> Vec<OverviewSearchItem> {
        let items = vec![
            OverviewSearchItem::new(
                "command-run-in-background".to_owned(),
                query.to_owned(),
                Some("Run in background".to_owned()),
                "utilities-terminal".to_owned(),
                "run".to_owned(),
                OverviewSearchItemAction::RunCommand(query.to_owned()),
                None
            ),

            OverviewSearchItem::new(
                "command-run-in-terminal".to_owned(),
                query.to_owned(),
                Some("Run in terminal".to_owned()),
                "utilities-terminal".to_owned(),
                "run".to_owned(),
                OverviewSearchItemAction::Launch(format!("foot fish -C \"{}\"", query.replace('"', "\\\""))),
                None
            )
        ];

        if COMMON_FOREGROUND_COMMANDS.iter().any(|&cmd| query.starts_with(cmd)) {
            items.into_iter().rev().collect()
        } else {
            items
        }
    }
}