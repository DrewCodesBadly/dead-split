extends Node

enum ComparisonType {
	COMPARISON_TYPE_NONE,
	COMPARISON_TYPE_TIME,
	COMPARISON_TYPE_DELTA
}

enum TimerPhase {
	NOT_RUNNING,
	RUNNING,
	ENDED,
	PAUSED
}

var rta := true
var wayland_hotkeys := true
var show_title := true
var show_splits := true
var show_attempt_count := true
var show_finished_runs := true
var title_one_line := false
var shown_splits := 10
var shown_upcoming_splits := 1
var last_split_pinned := true
var time_rounding := 2

var active_comparison: String = "Personal Best"
var active_comp_idx: int = 0

var working_directory_path: String = ""
var current_file_path: String = "None"
var autosplitter_path: String = ""
var autosplitter_settings_dict: Dictionary[String, Variant] = {}

var timer_theme_path: String = ""
var settings_profile_path: String = ""

var window_size: Vector2i = Vector2i(750, 750)
var split_time_min_size := 125 #TODO: Add setting for this

# DO NOT SERIALIZE THIS
var theme: TimerTheme = null

# Takes a val in seconds and turns it into hh:mm:ss.ss
func round_off(val: float) -> String:
	var out_string := ""
	if val < 0.0:
		out_string += "-"
		val = abs(val)
	# Hours
	var hours := floori(val / 3600.0)
	if hours > 0:
		out_string += str(hours) + ":"
	# Minutes
	var minutes := floori(val / 60) % 60
	var minutes_str := str(minutes) + ":"
	while hours > 0 and minutes_str.length() < 3:
		minutes_str = "0" + minutes_str
	if minutes > 0:
		out_string += minutes_str
	# Seconds
	var seconds_str := str(floori(val) % 60) + "."
	while minutes > 0 and seconds_str.length() < 3:
		seconds_str = "0" + seconds_str
	out_string += seconds_str
	# Decimals
	var decimals_str = str(fmod(val, 1.0)).substr(2)
	while decimals_str.length() < time_rounding:
		decimals_str += "0"
	out_string += decimals_str.substr(0, 2)
	return out_string

func round_off_no_decimal(val: float) -> String:
	var out_string := ""
	if val > 3600.0:
		out_string += str(floori(val / 3600)) + ":"
		if fmod(val, 3600) < 600.0:
			out_string += "0"
	if val > 60.0:
		out_string += str(floori(val / 60) % 60) + ":"
		if fmod(val, 60.0) < 10.0:
			out_string += "0"
	else:
		out_string += "0:"
		if val < 10.0:
			out_string += "0"
	
	out_string += str(floori(val) % 60)
	
	return out_string

func reload_theme() -> void:
	if timer_theme_path != "" and timer_theme_path.is_absolute_path():
		var zip := ZIPReader.new()
		var err := zip.open(timer_theme_path)
		if err != OK:
			theme = load("res://DefaultTheming/DefaultTimerTheme.tres")
		else:
			var new_theme := TimerTheme.new()
			# jank here because actually saving TimerTheme would save a script -
			# this causes the script to load twice and break :(
			for property in new_theme.get_property_list():
				# once again using temp files due to jank zip interface
				# thankfully the filenames are the same as the properties so we can for loop this
				var file_name: String = property["name"] + ".tres"
				if zip.file_exists(file_name):
					var temp_file := FileAccess.create_temp(
						FileAccess.READ_WRITE, "resource_load_temp", ".tres", false)
					temp_file.store_string(zip.read_file(file_name).get_string_from_utf8()) # copy
					temp_file.close()
					var resource: Resource = ResourceLoader.load(temp_file.get_path_absolute())
					new_theme.set(property["name"], resource)
			
			theme = new_theme
	else:
		theme = load("res://DefaultTheming/DefaultTimerTheme.tres")

func save() -> void:
	var settings := TimerSettingsSerializable.new()
	
	settings.rta = rta
	settings.wayland_hotkeys = wayland_hotkeys
	settings.show_title = show_title
	settings.show_splits = show_splits
	settings.show_attempt_count = show_attempt_count
	settings.show_finished_runs = show_finished_runs
	settings.title_one_line = title_one_line
	settings.shown_splits = shown_splits
	settings.shown_upcoming_splits = shown_upcoming_splits
	settings.last_split_pinned = last_split_pinned
	settings.time_rounding = time_rounding
	settings.active_comparison = active_comparison
	settings.active_comp_idx = active_comp_idx
	settings.working_directory_path = working_directory_path
	settings.current_file_path = current_file_path
	settings.timer_theme_path = timer_theme_path
	settings.autosplitter_path = autosplitter_path
	settings.autosplitter_settings_dict = autosplitter_settings_dict
	settings.window_size = window_size
	settings.split_time_min_size = split_time_min_size
	
	# Finally, we save hotkeys
	settings.hotkeys_dict = MainTimer.get_hotkeys_dict()
	
	# Debug
	#print("Saving!")
	#for property in settings.get_property_list():
			#print(property["name"] + ": " + str(settings.get(property["name"])))
	
	ResourceSaver.save(settings, "user://deadsplit_settings.tres")
	if settings_profile_path != "" and settings_profile_path.is_absolute_path():
		ResourceSaver.save(settings, settings_profile_path)

func try_load() -> void:
	if ResourceLoader.exists("user://deadsplit_settings.tres", "TimerSettingsSerializable"):
		var settings = ResourceLoader.load("user://deadsplit_settings.tres")
		
		# Debug
		#print("Loading!")
		#for property in settings.get_property_list():
			#print(property["name"] + ": " + str(settings.get(property["name"])))
		
		rta = settings.rta 
		wayland_hotkeys = settings.wayland_hotkeys
		show_title = settings.show_title 
		show_splits = settings.show_splits
		show_attempt_count = settings.show_attempt_count
		show_finished_runs = settings.show_finished_runs
		title_one_line = settings.title_one_line
		shown_splits = settings.shown_splits
		shown_upcoming_splits = settings.shown_upcoming_splits
		last_split_pinned = settings.last_split_pinned
		time_rounding = settings.time_rounding
		active_comparison = settings.active_comparison
		active_comp_idx = settings.active_comp_idx
		working_directory_path = settings.working_directory_path
		current_file_path = settings.current_file_path
		timer_theme_path = settings.timer_theme_path
		autosplitter_path = settings.autosplitter_path
		autosplitter_settings_dict = settings.autosplitter_settings_dict
		window_size = settings.window_size
		split_time_min_size = settings.split_time_min_size
		
		MainTimer.reset_hotkey_manager(wayland_hotkeys)
		
		# Load hotkeys
		for k in settings.hotkeys_dict:
			MainTimer.add_hotkey(settings.hotkeys_dict[k], k)

func reload_autosplitter() -> void:
	# Stop the autosplitter from running, it will restart if a new GDScript autosplitter loads
	MainTimer.autosplitter = null
	MainTimer.autosplitter_ticks.stop()
	MainTimer.detach_process()
	MainTimer.unload_wasm_autosplitter()
	
	if autosplitter_path == "" or !autosplitter_path.is_absolute_path(): return
	if autosplitter_path.ends_with(".gd"): # GDScript autosplitters
		var autosplitter := Autosplitter.new()
		var script = ResourceLoader.load(autosplitter_path, "Script")
		if script and autosplitter:
			autosplitter.set_script(script)
			MainTimer.autosplitter = autosplitter
			autosplitter.setup()
			
			# Handle settings
			for setting in autosplitter_settings_dict:
				if autosplitter.settings.has(setting):
					autosplitter.settings[setting] = autosplitter_settings_dict[setting]
				else:
					autosplitter_settings_dict.erase(setting)
			for setting in autosplitter.settings:
				if !autosplitter_settings_dict.has(setting):
					autosplitter_settings_dict[setting] = autosplitter.settings[setting]
			
			autosplitter.read_settings()
			
			# Start the autosplitter
			MainTimer.autosplitter_ticks.start()
	else: # Must be wasm autosplitter
		MainTimer.load_wasm_autosplitter(autosplitter_path)
		# Make sure the autosplitter finishes loading and sets settings
		await get_tree().create_timer(0.1).timeout
		# Then set the settings on the autosplitter
		var dict := MainTimer.get_wasm_settings_dict()
		for key in dict.keys():
			if autosplitter_settings_dict.has(key):
				dict[key] = autosplitter_settings_dict[key]
			else:
				autosplitter_settings_dict[key] = dict[key]
		
		for key in autosplitter_settings_dict.keys():
			if !dict.has(key):
				autosplitter_settings_dict.erase(key)
		
		MainTimer.set_wasm_settings_dict(dict)

func load_profile(path: String) -> void:
	if path != "" and path.is_absolute_path() and ResourceLoader.exists(path, "TimerSettingsSerializable"):
		var settings = ResourceLoader.load("user://deadsplit_settings.tres")
		
		rta = settings.rta 
		show_title = settings.show_title 
		show_splits = settings.show_splits
		show_attempt_count = settings.show_attempt_count
		show_finished_runs = settings.show_finished_runs
		title_one_line = settings.title_one_line
		shown_splits = settings.shown_splits
		shown_upcoming_splits = settings.shown_upcoming_splits
		last_split_pinned = settings.last_split_pinned
		time_rounding = settings.time_rounding
		active_comparison = settings.active_comparison
		active_comp_idx = settings.active_comp_idx
		working_directory_path = settings.working_directory_path
		current_file_path = settings.current_file_path
		timer_theme_path = settings.timer_theme_path
		autosplitter_path = settings.autosplitter_path
		autosplitter_settings_dict = settings.autosplitter_settings_dict
		window_size = settings.window_size
		split_time_min_size = settings.split_time_min_size
		
		# Load hotkeys
		MainTimer.reset_hotkey_manager(wayland_hotkeys)
		for k in settings.hotkeys_dict:
			MainTimer.add_hotkey(settings.hotkeys_dict[k], k)

func reload_hotkeys() -> void:
	var hotkeys_dict := MainTimer.get_hotkeys_dict()
	
	MainTimer.reset_hotkey_manager(wayland_hotkeys)
	for k in hotkeys_dict:
		MainTimer.add_hotkey(hotkeys_dict[k], k)
