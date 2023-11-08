use egui::{Ui, Window};
use egui_plot::{Text, PlotPoint, BarChart, Plot, Bar};
use inkbound_parser::parser::{PlayerStats, DataLog};

use crate::{class_string_to_color, Overlay};

#[inline]
pub fn draw_combat_damage_window(overlay: &Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.enabled_windows.show_combat_group_damage {
        return;
    }

    let name = "Combat Group Damage";
    Window::new(name).show(ctx, |ui| {
        let statlist: Vec<PlayerStats> = {
            if let Some(dive) = datalog.dives.get(0) {
                if let Some(combat) = dive.combats.get(0) {
                    combat.player_stats.player_stats.values().cloned().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }; // Close here to release lock

        draw_group_damage_plot(ui, statlist, name);
    });
}

#[inline]
pub fn draw_dive_damage_window(overlay: &Overlay, ctx: &egui::Context, datalog: &DataLog) {
    if !overlay.enabled_windows.show_dive_group_damage {
        return;
    }

    let name = "Dive Group Damage";
    Window::new(name).show(ctx, |ui| {
        let statlist: Vec<PlayerStats> = {
            if let Some(dive) = datalog.dives.get(0) {
                dive.player_stats.player_stats.values().cloned().collect()
            } else {
                Vec::new()
            }
        }; // Close here to release lock

        draw_group_damage_plot(ui, statlist, name);
    });    
}

/// Helper to draw the plot for group damage stats
#[inline]
fn draw_group_damage_plot(ui: &mut Ui, mut statlist: Vec<PlayerStats>, name: &str) {
    // TODO: Precalculate this in the DiveLog probably
    let party_damage = statlist.iter().fold(0, |acc, player| acc + player.total_damage_dealt) as f64;
    
    statlist.sort_by_key(|e| e.total_damage_dealt);
    let bars = {
        statlist.iter().enumerate().map(|(index, stats)| 
            Bar::new(index as f64, stats.total_damage_dealt as f64)
                .width(1.0)
                .fill(class_string_to_color(stats.player_data.class.as_str()))
        ).collect()
    };
    let texts: Vec<Text> = {
        statlist.iter().enumerate().map(|(index, stats)| 
            Text::new(
                PlotPoint { x: 0.0, y: index as f64 },
                // TODO: consider number seperatoring
                format!("  {} - {} ({:.2}%)",
                    stats.player_data.name,
                    stats.total_damage_dealt,
                    stats.total_damage_dealt as f64 / party_damage * 100.0,
                )
            )
            .anchor(egui::Align2::LEFT_CENTER)
            .color(egui::Color32::WHITE)
        ).collect()
    };
    // let bars: Vec<Bar> = values.iter().enumerate().map(|(c, e)| 
    //     Bar::new(c as f64, *e as f64).width(1.0).name("foo")
    // ).collect();
    
    let chart = BarChart::new(bars)
        .horizontal()
    ;
    Plot::new(format!("{} Plot", name))
        .allow_boxed_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .allow_zoom(false)
        .show_grid(false)
        .show_axes(false)
        .show_background(false)
        .show_x(false)
        .show_y(false)
        .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                for text in texts {
                    plot_ui.text(text);
                }
                // plot_ui.text(Text::new(PlotPoint{ x:0.0, y: 1.0}, "  meep").anchor(egui::Align2::LEFT_CENTER))
            }
        );
}
