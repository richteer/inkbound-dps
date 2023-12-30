# Changelog

All notable changes to this project will be documented in this file.

## [0.3.1] - 2023-12-30

### Bug Fixes

- *(overlay)* Disable mouse passthrough in windowed mode


### Features

- *(overlay)* Add spiked as a viewable status effect


### Miscellaneous Tasks

- *(cliff)* Add new skipped category for clippy fixes


### Styling

- *(overlay)* Change NaN% to 0% when there is no data to be displayed


## [0.3.0] - 2023-12-29

### Bug Fixes

- *(main)* Re-enable stdout/stderr logging in windows debug builds
- *(overlay)* Fix overlay crashing when minimized
- *(overlay/individual)* Fix dive individual damage window not displaying until combat has started
- *(overlay/skilltotals)* Hopefully fix some non-legendary bindings not being detected as upgraded


### Features

- *(overlay)*  **BREAKING:** Add support for creating (and deleting) multiple windows of the same type
- *(overlay)* Windows now contain their options in a collapsable header
- *(overlay)* Add button to restart the log parser, show parser state in settings
- *(overlay)* Add new stat table window
- *(overlay/extractors)* Add percent crit damage stat option
- *(overlay/group)* Add ability to select what stat the group window displays
- *(overlay/group)* Add option for custom format strings to group stats
- *(overlay/history)* Implement stat selection for history
- *(overlay/history)* Add percent mode for history
- *(overlay/skilltotals)* Add option for custom format strings to skill totals
- *(overlay/skilltotals)* Add option to merge upgraded skills
- *(overlay/updater)* Add full changelog viewer when clicking on version
- *(parser)* Add support for parsing orb pickups
- *(parser)* Add detection for point-of-view player
- *(parser)* Add support for parsing status effect applications
- Add -w/--window argument to overlay in a window


### Miscellaneous Tasks

- *(build)* Build the parser library and all dependencies with optimizations in debug builds
- *(cliff)* Adjust cliff breaking format
- *(overlay)* Serde-defaultify all the persisted structs


### Refactor

- *(main)* Move all log reading -> parsing logic into its own crate
- *(overlay)*  **BREAKING:** Window overhaul part 1: move to using a trait-based display method
- *(overlay)*  **BREAKING:** Window overhaul part 2: re-enable persisting windows
- *(overlay)*  **BREAKING:** Merge individual combat and dive windows
- *(overlay)*  **BREAKING:** Merge group damage dive and combat windows
- *(overlay)*  **BREAKING:** Factor out dive/combat selection logic
- *(overlay)* Remove now-useless default player name option
- *(overlay/extractor)* Merge status effects into one extractor, add extra status effect selection option
- *(overlay/extractors)* Add trait for stat selection, so that the options and behaviors can be reused
- *(overlay/extractors)* Have status effects applied carry the status in the enum, have the enum implement extract stat


### Styling

- *(history)* Use selectable values instead of combobox for mode, re-enable totals in percent mode
- *(overlay)* Wait until data exists to show dive/combat/etc selectors, display a waiting message when there is no data to render yet
- *(overlay/skills)* Clean up skill names
- *(overlay/updater)* Remove unnecessary duplicate separator inside updater settings
- (overlay/updater): put the fetched changelog in a scroll area to avoid a large tooltip


### Testing

- *(overlay)* Add test case for catching config breakage


## [0.2.5] - 2023-12-05

### Miscellaneous Tasks

- Rewrite actions, publish release on tag push


### Refactor

- *(updater)* Lock auto-update behavior behind a feature, disabled by default for custom builds


## [0.2.4] - 2023-12-04

### Bug Fixes

- Update egui to 0.24.1 to fix the white flash on window launch


### Features

- *(overlay/history)* Add ability to sort history bars by name or damage total
- *(overlay/updater)* Add ui to overlay for auto updating, add auto update checking option
- *(overlay/updater)* Add release notes to new version hover text
- Add self-updating functionality, starting with --update cli argument


### Performance

- *(main)* Put backlog parsing in its own thread to not slow full application load


### Styling

- Tweak order of aspects to match the selection screen


## [0.2.3] - 2023-11-30

### Bug Fixes

- *(overlay/individual)* Fix bar flickering if two skills have exactly the same damage totals


### Features

- *(overlay)* Add support for custom aspect colors


### Miscellaneous Tasks

- Update cliff template to include scope


### Refactor

- *(overlay)* Replace class_string_to_color with Aspect::default_color
- *(overlay/options)* Move OverlayOptions into its own file to contain the growth
- *(parser)* Use an Enum for handling Aspects


### Styling

- *(overlay)* Tweak default aspect colors based on icon, fix godkeeper coloring


## [0.2.2] - 2023-11-27

### Bug Fixes

- *(overlay/history)* Fix slight x-offset in split history mode


### Features

- *(overlay/individual)* Add an option to show crit damage per skill
- *(parser)* Add crit damage totals per skills


### Miscellaneous Tasks

- Add cliff.toml


## [0.1.0] - 2023-11-24

### Miscellaneous Tasks

- Replace debouncer with explicit PollWatcher
- Specify 0.23.0 branch for eframe-overlay until I update for 0.24.0


