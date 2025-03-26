extends Resource

class_name TimerSettingsSerializable

# Silly situation i got myself into

@export var rta := true
@export var wayland_hotkeys := true
@export var show_title := true
@export var show_splits := true
@export var show_attempt_count := true
@export var show_finished_runs := true
@export var title_one_line := false
@export var shown_splits := 10
@export var shown_upcoming_splits := 1
@export var last_split_pinned := true
@export var time_rounding := 0.01

@export var active_comparison: String = "Personal Best"
@export var active_comp_idx: int = 0

@export var working_directory_path: String = ""
@export var current_file_path: String = "None"
@export var autosplitter_path: String = ""
@export var timer_theme_path: String = ""

@export var hotkeys_dict: Dictionary

@export var autosplitter_settings_dict: Dictionary[String, Variant] = {}

@export var window_size: Vector2i = Vector2i(750, 750)
@export var split_time_min_size: int = 125
