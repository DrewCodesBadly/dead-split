extends Window

@export var submenus: Control
@export var menus_container: Control
@export var outer_toggle: Control
@export var menu_toggle: Control
@export var igt_toggle: CheckButton
@export var wayland_toggle: CheckButton

@export var save_profile_button: Button
@export var load_profile_button: Button
@export var profile_saver: FileDialog
@export var profile_loader: FileDialog

@onready var quit_index := menus_container.get_child_count()

var timer_window: Control
var current_menu: Control

func _ready() -> void:
	# so editor changes don't make it into runtime
	outer_toggle.show()
	menu_toggle.hide()
	
	for child in menus_container.get_children():
		child.hide()

func _on_close_requested() -> void:
	# Save settings!
	TimerSettings.reload_theme()
	TimerSettings.save()
	timer_window.update_settings()
	timer_window.settings_open = false
	MainTimer.run_changed.emit() # just assume run updated, it's easier than actually checking
	TimerSettings.reload_autosplitter()
	TimerSettings.reload_hotkeys()
	
	queue_free()

func _on_submenus_item_clicked(index: int, _at_position: Vector2, _mouse_button_index: int) -> void:
	if index == quit_index:
		await get_tree().physics_frame
		get_tree().quit(0)
	else:
		# Show menu at that index
		outer_toggle.hide()
		if current_menu: current_menu.hide()
		
		menu_toggle.show()
		current_menu = menus_container.get_child(index)
		current_menu.show()

func _on_back_button_pressed() -> void:
	menu_toggle.hide()
	outer_toggle.show()

func _on_save_button_pressed() -> void:
	TimerSettings.save()

func _on_use_igt_toggle_toggled(toggled_on: bool) -> void:
	TimerSettings.rta = !toggled_on


func _on_use_igt_toggle_visibility_changed() -> void:
	if igt_toggle.visible:
		igt_toggle.button_pressed = !TimerSettings.rta
	if wayland_toggle.visible:
		wayland_toggle.button_pressed = TimerSettings.wayland_hotkeys


func _on_save_profile_button_pressed() -> void:
	save_profile_button.text = "Waiting..."
	save_profile_button.disabled = true
	profile_saver.show()

func _on_profile_saver_file_selected(path: String) -> void:
	save_profile_button.text = "Save settings profile..."
	save_profile_button.disabled = false
	TimerSettings.settings_profile_path = path
	TimerSettings.save()

func _on_profile_saver_canceled() -> void:
	save_profile_button.text = "Save settings profile..."
	save_profile_button.disabled = false

func _on_profile_loader_file_selected(path: String) -> void:
	load_profile_button.text = "Load settings profile..."
	load_profile_button.disabled = false
	TimerSettings.load_profile(path)

func _on_profile_loader_canceled() -> void:
	load_profile_button.text = "Load settings profile..."
	load_profile_button.disabled = false

func _on_load_profile_button_pressed() -> void:
	load_profile_button.disabled = true
	load_profile_button.text = "Waiting..."
	profile_loader.show()

func _on_wayland_key_toggle_toggled(toggled_on: bool) -> void:
	TimerSettings.wayland_hotkeys = toggled_on
