extends DeadSplitTimer

@warning_ignore("unused_signal")
signal run_changed
signal comparison_changed

var autosplitter: Autosplitter = null
@onready var autosplitter_ticks: Timer = Timer.new()

func _ready() -> void:
	new_run()
	self.hotkey_pressed.connect(_hotkey_pressed)
	autosplitter_ticks.wait_time = 1.0/120.0
	autosplitter_ticks.one_shot = false
	autosplitter_ticks.timeout.connect(update_autosplitter)
	# Is started when autosplitter loads
	# This is not needed for webassembly autosplitters

func _hotkey_pressed(hotkey_id: int) -> void:
	match hotkey_id:
		0:
			start_split()
		1:
			reset()
		2:
			skip_split()
		3:
			undo_split()
		4:
			pause()
		5:
			resume()
		6:
			undo_all_pauses()
		7:
			toggle_pause()
		8:
			toggle_timing_method()
		9:
			var comp_list := get_comparisons()
			TimerSettings.active_comp_idx = (TimerSettings.active_comp_idx + 1) % comp_list.size()
			var comp := comp_list[TimerSettings.active_comp_idx]
			TimerSettings.active_comparison = comp
			comparison_changed.emit(comp)
		10:
			var comp_list := get_comparisons()
			TimerSettings.active_comp_idx = (TimerSettings.active_comp_idx - 1) % comp_list.size()
			var comp := comp_list[TimerSettings.active_comp_idx]
			TimerSettings.active_comparison = comp
			comparison_changed.emit(comp)

func update_autosplitter() -> void:
	if autosplitter:
		autosplitter.update()
	else:
		autosplitter_ticks.stop()
