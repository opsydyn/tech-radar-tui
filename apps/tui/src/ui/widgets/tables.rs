pub const fn scroll_offset(
    total_rows: usize,
    max_visible_rows: usize,
    selected_index: usize,
) -> usize {
    if total_rows <= max_visible_rows {
        return 0;
    }

    if selected_index >= max_visible_rows {
        return selected_index.saturating_sub(max_visible_rows) + 1;
    }

    selected_index
}
