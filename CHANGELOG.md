# Changelog

All notable changes to this project will be documented in this file.

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


