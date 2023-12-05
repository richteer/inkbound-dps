# Changelog

All notable changes to this project will be documented in this file.

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


