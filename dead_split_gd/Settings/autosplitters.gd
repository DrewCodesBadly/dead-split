extends ScrollContainer

@export var load_button: Button
@export var load_label: Label
@export var autosplit_picker: FileDialog
@export var container: GridContainer

func _on_load_button_pressed() -> void:
	load_button.disabled = true
	load_button.text = "Waiting..."
	autosplit_picker.show()

func _on_autosplit_picker_file_selected(path: String) -> void:
	TimerSettings.autosplitter_path = path
	load_button.disabled = false
	load_button.text = "Open..."
	update_label()
	refresh()

func _on_autosplit_picker_canceled() -> void:
	load_button.disabled = false
	load_button.text = "Open..."
	update_label()

func _on_visibility_changed() -> void:
	if visible:
		update_label()
		refresh()

func update_label() -> void:
	load_label.text = "Autosplitter File: " + TimerSettings.autosplitter_path

# Reloads the autosplitter and its settings
func refresh() -> void:
	TimerSettings.reload_autosplitter()
	
	for child in container.get_children():
		child.queue_free()
	
	for key in TimerSettings.autosplitter_settings_dict:
		var label := Label.new()
		label.text = key
		container.add_child(label)
		var val = TimerSettings.autosplitter_settings_dict[key]
		match typeof(val):
			
			# Writes appropriate data to the dictionary in TimerSettings
			# This overwrites changes to the actual autosplitter settings during TimerSettings.reload_autosplitter()
			TYPE_BOOL:
				var check := CheckBox.new()
				check.button_pressed = val
				check.toggled.connect(func(b: bool): TimerSettings.autosplitter_settings_dict[key] = b)
				container.add_child(check)
			TYPE_INT:
				var spin := SpinBox.new()
				spin.value = val
				spin.step = 1.0
				spin.value_changed.connect(func(b: bool): TimerSettings.autosplitter_settings_dict[key] = floori(b))
				container.add_child(spin)
			TYPE_FLOAT:
				var spin := SpinBox.new()
				spin.value = val
				spin.step = 0.001
				spin.value_changed.connect(func(b: bool): TimerSettings.autosplitter_settings_dict[key] = b)
				container.add_child(spin)


func _on_clear_as_button_pressed() -> void:
	TimerSettings.autosplitter_path = ""
	TimerSettings.reload_autosplitter()
	update_label()
	refresh()
