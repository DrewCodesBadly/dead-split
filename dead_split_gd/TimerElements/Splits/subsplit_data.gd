extends SplitData

class_name SubsplitData

var start_index: int
# `index` is the end index
var subsplits: Array[SplitData] = []

# Override to return all subsplits needed if current, else just the subsplit header
func generate(focus_index: int = -1, shown_splits: int = 0, shown_splits_after_current: int = 0) -> Array[Node]:
	var arr: Array[Node] = []
	var current := MainTimer.current_split_index >= start_index and MainTimer.current_split_index <= index
	
	# Split focus is inside the subsplit so we need to render out subsplits
	if focus_index >= start_index and focus_index <= index:
		# Subsplit header
		arr = []
		var header_split := split_scene.instantiate()
		header_split.idx = index
		header_split.current = current
		header_split.set_split_name(split_name)
		header_split.update()
		arr.append(header_split)
		
		# Find the index we start rendering splits at
		var iterate_idx = focus_index + shown_splits_after_current - shown_splits + 1 # extra 1 for the header
		iterate_idx = max(iterate_idx, start_index)
		
		# Render as many splits as we need
		var rendered_splits := 1 # 1 for header
		while iterate_idx <= index and rendered_splits < shown_splits:
			var split := split_scene.instantiate()
			split.idx = iterate_idx
			split.set_split_name(subsplits[iterate_idx - start_index].split_name)
			if iterate_idx == MainTimer.current_split_index:
				split.current = true
			split.update()
			arr.append(split)
			
			rendered_splits += 1
			iterate_idx += 1
	
	else:
		# Just the header
		arr = []
		var split := split_scene.instantiate()
		split.current = current
		split.idx = index
		split.set_split_name(split_name)
		split.update()
		arr.append(split)
	
	return arr
