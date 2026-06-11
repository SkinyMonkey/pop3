/// Menu state machine for frontend screens (main menu, campaign select, etc.).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuScreen {
    MainMenu,
    CampaignSelect,
    LoadGame,
    Options,
    StatsScreen { victory: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    None,
    NavigateTo(MenuScreen),
    StartLevel(u32),
    LoadSave(String),
    Back,
    Quit,
}

pub struct MenuSystem {
    pub screen: MenuScreen,
    pub cursor: usize,
    pub game_speed: u32,
    pub max_levels: u32,
    pub completed_levels: Vec<bool>,
    pub save_filenames: Vec<String>,
    pub stats: Option<GameStats>,
}

#[derive(Debug, Clone, Default)]
pub struct GameStats {
    pub kills: u32,
    pub losses: u32,
    pub buildings_built: u32,
    pub spells_cast: u32,
    pub time_ticks: u32,
}

/// Data the HUD renderer needs to draw the current menu screen.
#[derive(Debug, Clone)]
pub struct MenuRenderData {
    pub screen: MenuScreen,
    pub items: Vec<MenuItem>,
    pub cursor: usize,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub enabled: bool,
}

impl MenuSystem {
    pub fn new(game_speed: u32) -> Self {
        Self {
            screen: MenuScreen::MainMenu,
            cursor: 0,
            game_speed,
            max_levels: 25,
            completed_levels: vec![false; 25],
            save_filenames: Vec::new(),
            stats: None,
        }
    }

    pub fn navigate_to(&mut self, screen: MenuScreen) {
        self.screen = screen;
        self.cursor = 0;
    }

    pub fn back(&mut self) {
        match self.screen {
            MenuScreen::CampaignSelect | MenuScreen::LoadGame | MenuScreen::Options => {
                self.screen = MenuScreen::MainMenu;
                self.cursor = 0;
            }
            MenuScreen::StatsScreen { .. } => {
                self.screen = MenuScreen::MainMenu;
                self.cursor = 0;
            }
            MenuScreen::MainMenu => {} // no-op at root
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        let count = self.item_count();
        if count == 0 {
            return;
        }
        self.cursor = ((self.cursor as i32 + delta).rem_euclid(count as i32)) as usize;
    }

    pub fn select_item(&mut self) -> MenuAction {
        match &self.screen {
            MenuScreen::MainMenu => match self.cursor {
                0 => MenuAction::NavigateTo(MenuScreen::CampaignSelect),
                1 => MenuAction::NavigateTo(MenuScreen::LoadGame),
                2 => MenuAction::NavigateTo(MenuScreen::Options),
                3 => MenuAction::Quit,
                _ => MenuAction::None,
            },
            MenuScreen::CampaignSelect => {
                let level = self.cursor as u32 + 1; // levels are 1-indexed
                if level <= self.max_levels {
                    MenuAction::StartLevel(level)
                } else {
                    MenuAction::None
                }
            }
            MenuScreen::LoadGame => {
                if self.cursor < self.save_filenames.len() {
                    MenuAction::LoadSave(self.save_filenames[self.cursor].clone())
                } else {
                    MenuAction::None
                }
            }
            MenuScreen::Options => {
                match self.cursor {
                    0 => {
                        self.game_speed = self.game_speed.saturating_sub(2).max(2);
                        MenuAction::None
                    }
                    1 => {
                        self.game_speed = (self.game_speed + 2).min(20);
                        MenuAction::None
                    }
                    2 => MenuAction::Back,
                    _ => MenuAction::None,
                }
            }
            MenuScreen::StatsScreen { .. } => match self.cursor {
                0 => MenuAction::StartLevel(0), // Continue (next level, resolved by caller)
                1 => MenuAction::StartLevel(0), // Retry (same level, resolved by caller)
                2 => MenuAction::Back,          // Menu
                _ => MenuAction::None,
            },
        }
    }

    fn item_count(&self) -> usize {
        match &self.screen {
            MenuScreen::MainMenu => 4,
            MenuScreen::CampaignSelect => self.max_levels as usize,
            MenuScreen::LoadGame => self.save_filenames.len().max(1),
            MenuScreen::Options => 3,
            MenuScreen::StatsScreen { .. } => 3,
        }
    }

    pub fn render_data(&self) -> MenuRenderData {
        let (title, items) = match &self.screen {
            MenuScreen::MainMenu => (
                "Populous: The Beginning".to_string(),
                vec![
                    MenuItem { label: "Campaign".to_string(), enabled: true },
                    MenuItem { label: "Load Game".to_string(), enabled: true },
                    MenuItem { label: "Options".to_string(), enabled: true },
                    MenuItem { label: "Quit".to_string(), enabled: true },
                ],
            ),
            MenuScreen::CampaignSelect => {
                let items = (1..=self.max_levels)
                    .map(|i| {
                        let completed = self
                            .completed_levels
                            .get(i as usize - 1)
                            .copied()
                            .unwrap_or(false);
                        let check = if completed { "[X]" } else { "[ ]" };
                        MenuItem {
                            label: format!("{} Level {:02}", check, i),
                            enabled: true,
                        }
                    })
                    .collect();
                ("Select Level".to_string(), items)
            }
            MenuScreen::LoadGame => {
                if self.save_filenames.is_empty() {
                    (
                        "Load Game".to_string(),
                        vec![MenuItem {
                            label: "No saved games found".to_string(),
                            enabled: false,
                        }],
                    )
                } else {
                    let items = self
                        .save_filenames
                        .iter()
                        .map(|f| MenuItem {
                            label: f.clone(),
                            enabled: true,
                        })
                        .collect();
                    ("Load Game".to_string(), items)
                }
            }
            MenuScreen::Options => (
                "Options".to_string(),
                vec![
                    MenuItem {
                        label: format!("Game Speed: {} [-]", self.game_speed),
                        enabled: true,
                    },
                    MenuItem {
                        label: format!("Game Speed: {} [+]", self.game_speed),
                        enabled: true,
                    },
                    MenuItem {
                        label: "Back".to_string(),
                        enabled: true,
                    },
                ],
            ),
            MenuScreen::StatsScreen { victory } => {
                let title = if *victory { "Victory!" } else { "Defeat" };
                let stats = self
                    .stats
                    .as_ref()
                    .map(|s| {
                        vec![
                            MenuItem {
                                label: format!("Kills: {}", s.kills),
                                enabled: false,
                            },
                            MenuItem {
                                label: format!("Losses: {}", s.losses),
                                enabled: false,
                            },
                            MenuItem {
                                label: format!("Buildings: {}", s.buildings_built),
                                enabled: false,
                            },
                            MenuItem {
                                label: "---".to_string(),
                                enabled: false,
                            },
                        ]
                    })
                    .unwrap_or_default();
                let mut items = stats;
                if *victory {
                    items.push(MenuItem {
                        label: "Continue".to_string(),
                        enabled: true,
                    });
                }
                items.push(MenuItem {
                    label: "Retry".to_string(),
                    enabled: true,
                });
                items.push(MenuItem {
                    label: "Main Menu".to_string(),
                    enabled: true,
                });
                (title.to_string(), items)
            }
        };
        MenuRenderData {
            screen: self.screen.clone(),
            items,
            cursor: self.cursor,
            title,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_main_menu() {
        let menu = MenuSystem::new(8);
        assert_eq!(menu.screen, MenuScreen::MainMenu);
        assert_eq!(menu.cursor, 0);
    }

    #[test]
    fn navigate_main_to_campaign() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::CampaignSelect);
        assert_eq!(menu.screen, MenuScreen::CampaignSelect);
        assert_eq!(menu.cursor, 0);
    }

    #[test]
    fn back_from_campaign_to_main() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::CampaignSelect);
        menu.back();
        assert_eq!(menu.screen, MenuScreen::MainMenu);
    }

    #[test]
    fn navigate_main_to_load_game() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::LoadGame);
        assert_eq!(menu.screen, MenuScreen::LoadGame);
    }

    #[test]
    fn select_item_campaign_returns_start_level() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::CampaignSelect);
        menu.cursor = 4; // level 5
        let action = menu.select_item();
        assert_eq!(action, MenuAction::StartLevel(5));
    }

    #[test]
    fn select_item_main_menu_returns_navigate() {
        let mut menu = MenuSystem::new(8);
        menu.cursor = 0; // Campaign
        let action = menu.select_item();
        assert_eq!(action, MenuAction::NavigateTo(MenuScreen::CampaignSelect));
    }

    #[test]
    fn options_screen_game_speed() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::Options);

        // Decrease speed
        menu.cursor = 0;
        menu.select_item();
        assert_eq!(menu.game_speed, 6);

        // Increase speed
        menu.cursor = 1;
        menu.select_item();
        assert_eq!(menu.game_speed, 8);
    }

    #[test]
    fn cursor_wraps_around() {
        let mut menu = MenuSystem::new(8);
        // MainMenu has 4 items
        menu.move_cursor(-1); // wrap to last
        assert_eq!(menu.cursor, 3);
        menu.move_cursor(1); // wrap to first
        assert_eq!(menu.cursor, 0);
    }

    #[test]
    fn render_data_main_menu() {
        let menu = MenuSystem::new(8);
        let data = menu.render_data();
        assert_eq!(data.title, "Populous: The Beginning");
        assert_eq!(data.items.len(), 4);
        assert_eq!(data.items[0].label, "Campaign");
    }

    #[test]
    fn render_data_campaign_25_levels() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::CampaignSelect);
        let data = menu.render_data();
        assert_eq!(data.items.len(), 25);
        assert!(data.items[0].label.contains("Level 01"));
    }

    #[test]
    fn back_from_stats_goes_to_main() {
        let mut menu = MenuSystem::new(8);
        menu.navigate_to(MenuScreen::StatsScreen { victory: true });
        menu.back();
        assert_eq!(menu.screen, MenuScreen::MainMenu);
    }

    #[test]
    fn back_at_main_is_noop() {
        let mut menu = MenuSystem::new(8);
        menu.back();
        assert_eq!(menu.screen, MenuScreen::MainMenu);
    }

    #[test]
    fn game_speed_clamps() {
        let mut menu = MenuSystem::new(2);
        menu.navigate_to(MenuScreen::Options);
        menu.cursor = 0; // decrease
        menu.select_item();
        assert_eq!(menu.game_speed, 2); // min clamp

        let mut menu = MenuSystem::new(20);
        menu.navigate_to(MenuScreen::Options);
        menu.cursor = 1; // increase
        menu.select_item();
        assert_eq!(menu.game_speed, 20); // max clamp
    }
}
