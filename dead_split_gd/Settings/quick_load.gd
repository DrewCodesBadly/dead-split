extends ScrollContainer

@export var no_dir_label: Label
@export var file_list: ItemList
@export var window: Window
@export var file_endings: Array[String]
@export var search: LineEdit

var files: Dictionary[String, bool]

func _on_item_list_item_clicked(index: int, _at_position: Vector2, mouse_button_index: int) -> void:
	if mouse_button_index == MOUSE_BUTTON_LEFT:
		if files[file_list.get_item_text(index)]:
			load_file("res://Autosplitters/" + file_list.get_item_text(index))
		else:
			load_file(TimerSettings.working_directory_path + "/" + file_list.get_item_text(index))

func _on_visibility_changed() -> void:
	if visible:
		no_dir_label.hide()
		file_list.hide()
		
		var dir = DirAccess.open(TimerSettings.working_directory_path)
		if dir == null or TimerSettings.working_directory_path == "":
			no_dir_label.show()
			return
		
		# Focus on the search bar
		# No idea why this error happens here but it does sometimes so just to be safe
		if !search.is_inside_tree():
			await search.tree_entered
		search.grab_focus()
		
		file_list.show()
		file_list.clear()
		files.clear()
		
		# Add everything in the working directory (Recursive)
		for file in dir.get_files():
			for file_ending in file_endings:
				if file.ends_with(file_ending):
					files[file] = false
		
		# Also add autosplitters
		dir = DirAccess.open("res://Autosplitters")
		for file in dir.get_files():
			if file.ends_with(".gd"):
				files[file] = true
		
		files.sort()
		
		for file in files:
			file_list.add_item(file)

# Boring linear search because i'm lazy womp womp
func _on_search_text_changed(new_text: String) -> void:
	file_list.clear()
	for file in files:
		if file.to_lower().contains(new_text.to_lower()):
			file_list.add_item(file)

# Picks the first file from the list when you submit text into the serach bar
func _on_search_text_submitted(_new_text: String) -> void:
	load_file(TimerSettings.working_directory_path + "/" + file_list.get_item_text(0))

func load_file(path: String) -> void:
	
	# Check which type of file it is and set paths appropriately.
	# Everything should get reloaded when the window is closed except profiles and runs.
	if path.ends_with(".gd") or path.ends_with(".wasm"): # autosplitter
		TimerSettings.autosplitter_path = path
	elif path.ends_with(".zip"): # theme
		TimerSettings.timer_theme_path = path
	elif path.ends_with(".lss"): # run
		TimerSettings.current_file_path = path
		MainTimer.try_load_run(TimerSettings.current_file_path)
	elif path.ends_with(".tres"):
		TimerSettings.load_profile(path)
	
	window._on_close_requested()
