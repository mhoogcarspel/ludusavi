use crate::{
    config::Config,
    gui::{
        badge::Badge,
        common::{Message, Screen},
        file_tree::FileTree,
        icon::Icon,
        search::SearchComponent,
        style,
    },
    lang::Translator,
    manifest::Manifest,
    prelude::{BackupInfo, DuplicateDetector, ScanInfo},
};

use fuzzy_matcher::FuzzyMatcher;
use iced::{
    button, scrollable, Align, Button, Checkbox, Column, Container, HorizontalAlignment, Length, Row, Scrollable,
    Space, Text,
};

#[derive(Default)]
pub struct GameListEntry {
    pub scan_info: ScanInfo,
    pub backup_info: Option<BackupInfo>,
    pub expand_button: button::State,
    pub wiki_button: button::State,
    pub customize_button: button::State,
    pub expanded: bool,
    pub tree: FileTree,
    pub tree_should_reload: bool,
}

impl GameListEntry {
    fn view(
        &mut self,
        restoring: bool,
        translator: &Translator,
        config: &Config,
        manifest: &Manifest,
        duplicate_detector: &DuplicateDetector,
    ) -> Container<Message> {
        let successful = match &self.backup_info {
            Some(x) => x.successful(),
            _ => true,
        };

        if self.expanded {
            if self.tree_should_reload {
                self.tree = FileTree::new(self.scan_info.clone(), &config, &self.backup_info, &duplicate_detector);
                self.tree_should_reload = false;
            }
        } else {
            self.tree.clear();
            self.tree_should_reload = true;
        }

        let enabled = if restoring {
            config.is_game_enabled_for_restore(&self.scan_info.game_name)
        } else {
            config.is_game_enabled_for_backup(&self.scan_info.game_name)
        };
        let customized = config.is_game_customized(&self.scan_info.game_name);
        let customized_pure = customized && !manifest.0.contains_key(&self.scan_info.game_name);
        let name_for_checkbox = self.scan_info.game_name.clone();

        Container::new(
            Column::new()
                .padding(5)
                .spacing(5)
                .align_items(Align::Center)
                .push(
                    Row::new()
                        .push(Checkbox::new(enabled, "", move |enabled| {
                            Message::ToggleGameListEntryEnabled {
                                name: name_for_checkbox.clone(),
                                enabled,
                                restoring,
                            }
                        }))
                        .push(
                            Button::new(
                                &mut self.expand_button,
                                Text::new(self.scan_info.game_name.clone())
                                    .horizontal_alignment(HorizontalAlignment::Center),
                            )
                            .on_press(Message::ToggleGameListEntryExpanded {
                                name: self.scan_info.game_name.clone(),
                            })
                            .style(if !enabled {
                                style::Button::GameListEntryTitleDisabled
                            } else if successful {
                                style::Button::GameListEntryTitle
                            } else {
                                style::Button::GameListEntryTitleFailed
                            })
                            .width(Length::Fill)
                            .padding(2),
                        )
                        .push(if duplicate_detector.is_game_duplicated(&self.scan_info) {
                            Badge::new(&translator.badge_duplicates()).left_margin(15).view()
                        } else {
                            Container::new(Space::new(Length::Shrink, Length::Shrink))
                        })
                        .push(if !successful {
                            Badge::new(&translator.badge_failed()).left_margin(15).view()
                        } else {
                            Container::new(Space::new(Length::Shrink, Length::Shrink))
                        })
                        .push(Space::new(
                            Length::Units(if restoring { 0 } else { 15 }),
                            Length::Shrink,
                        ))
                        .push(if restoring {
                            Container::new(Space::new(Length::Shrink, Length::Shrink))
                        } else {
                            Container::new(
                                Button::new(
                                    &mut self.customize_button,
                                    Icon::Edit.as_text().width(Length::Units(45)),
                                )
                                .on_press(if customized {
                                    Message::Ignore
                                } else {
                                    Message::CustomizeGame {
                                        name: self.scan_info.game_name.clone(),
                                    }
                                })
                                .style(if customized {
                                    style::Button::Disabled
                                } else {
                                    style::Button::Primary
                                })
                                .padding(2),
                            )
                        })
                        .push(Space::new(Length::Units(15), Length::Shrink))
                        .push(Container::new(
                            Button::new(&mut self.wiki_button, Icon::Language.as_text().width(Length::Units(45)))
                                .on_press(if customized_pure {
                                    Message::Ignore
                                } else {
                                    Message::OpenWiki {
                                        game: self.scan_info.game_name.clone(),
                                    }
                                })
                                .style(if customized_pure {
                                    style::Button::Disabled
                                } else {
                                    style::Button::Primary
                                })
                                .padding(2),
                        ))
                        .push(
                            Container::new(Text::new(
                                translator.adjusted_size(self.scan_info.sum_bytes(&self.backup_info)),
                            ))
                            .width(Length::Units(115))
                            .center_x(),
                        ),
                )
                .push(
                    self.tree
                        .view(&translator, &self.scan_info.game_name)
                        .width(Length::Fill),
                ),
        )
        .style(style::Container::GameListEntry)
    }
}

#[derive(Default)]
pub struct GameList {
    pub entries: Vec<GameListEntry>,
    scroll: scrollable::State,
    pub search: SearchComponent,
}

impl GameList {
    pub fn view(
        &mut self,
        restoring: bool,
        translator: &Translator,
        config: &Config,
        manifest: &Manifest,
        duplicate_detector: &DuplicateDetector,
    ) -> Container<Message> {
        let use_search = self.search.show;
        let search_game_name = self.search.game_name.clone();

        self.entries.sort_by_key(|x| x.scan_info.game_name.clone());
        Container::new(
            Column::new()
                .push(
                    self.search
                        .view(if restoring { Screen::Restore } else { Screen::Backup }, &translator),
                )
                .push({
                    self.entries.iter_mut().enumerate().fold(
                        Scrollable::new(&mut self.scroll)
                            .width(Length::Fill)
                            .padding(10)
                            .style(style::Scrollable),
                        |parent: Scrollable<'_, Message>, (_i, x)| {
                            if !use_search
                                || fuzzy_matcher::skim::SkimMatcherV2::default()
                                    .fuzzy_match(&x.scan_info.game_name, &search_game_name)
                                    .is_some()
                            {
                                parent
                                    .push(x.view(restoring, translator, &config, &manifest, &duplicate_detector))
                                    .push(Space::new(Length::Units(0), Length::Units(10)))
                            } else {
                                parent
                            }
                        },
                    )
                }),
        )
    }

    pub fn all_entries_selected(&self, config: &Config, restoring: bool) -> bool {
        self.entries.iter().all(|x| {
            if restoring {
                config.is_game_enabled_for_restore(&x.scan_info.game_name)
            } else {
                config.is_game_enabled_for_backup(&x.scan_info.game_name)
            }
        })
    }

    pub fn count_selected_entries(&self, config: &Config, restoring: bool) -> (usize, u64) {
        let mut games = 0;
        let mut bytes = 0;
        for entry in self.entries.iter() {
            if (restoring && config.is_game_enabled_for_restore(&entry.scan_info.game_name))
                || (!restoring && config.is_game_enabled_for_backup(&entry.scan_info.game_name))
            {
                games += 1;
                bytes += entry.scan_info.sum_bytes(&None);
            }
        }
        (games, bytes)
    }
}
