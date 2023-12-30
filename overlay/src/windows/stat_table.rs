use egui::RichText;
use egui_extras::{Column, TableBuilder};
use inkbound_parser::parser::DataLog;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{windows::WindowDisplay, OverlayOptions};

use super::{extractors::StatExtractionFunc, DiveCombatSplit, DiveCombatSelection, DiveCombatSelectionState};

fn gen_extractors() -> Vec<StatExtractionFunc> {
    StatExtractionFunc::iter()
        .filter(|e| !matches!(e, StatExtractionFunc::StatusEffectApplied(_))).chain(
            super::ALLOWED_STATUS_EFFECTS.iter()
                .map(|se| StatExtractionFunc::StatusEffectApplied(se.to_string()))
        ).collect()
}

lazy_static::lazy_static! {
    static ref STAT_ROWS: Vec<StatExtractionFunc> = gen_extractors();
}

#[derive(Default, Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct StatTableWindow {
    #[serde(skip)]
    state: DiveCombatSelectionState,
    mode: DiveCombatSelection,
}

impl DiveCombatSplit for StatTableWindow {
    fn mode(&mut self) -> &mut super::DiveCombatSelection {
        &mut self.mode
    }

    fn set_mode(&mut self, mode: super::DiveCombatSelection) {
        self.mode = mode
    }

    fn state(&mut self) -> &mut super::DiveCombatSelectionState {
        &mut self.state
    }
}

#[typetag::serde]
impl WindowDisplay for StatTableWindow {
    fn show(&mut self, ui: &mut egui::Ui, _options: &OverlayOptions, data: &DataLog) {
        ui.collapsing("⛭", |ui| {
            self.mode_selection(ui);
            self.show_selection_boxes(ui, data);
        });

        let player_stats = if let Some(player_stats) = self.get_current_player_stat_list(data) {
            player_stats
        } else {
            ui.label(super::NO_DATA_MSG.to_string());
            return
        };
        
        TableBuilder::new(ui)
            .striped(true)
            .columns(Column::auto().resizable(true), player_stats.len() + 1)
            .header(15.0, |mut header| {
                header.col(|_ui| {}); // Empty column to ensure alignment
                for player in player_stats.keys() {
                    header.col(|ui| {
                        ui.label(RichText::new(player).strong());
                    });
                }
            })
            .body(|body| {
                body.rows(15.0, STAT_ROWS.len(), |index, mut row| {
                    row.col(|ui| {
                        ui.label(STAT_ROWS[index].to_string());
                    });
                    for player in player_stats.values() {
                        row.col(|ui| {
                            ui.label(STAT_ROWS[index].extract_formatted_stat(player).to_string());
                        });                    }
                });
            });
    }

    fn name(&self) -> String {
        "Table".into()
    }
}


