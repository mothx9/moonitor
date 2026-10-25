#!/bin/sh
# Renderer-specific candidates on first run; pixel comparisons on later runs.
# Each viewport owns a process/device so large Vulkan snapshots release all allocations.
set -eu
cd "$(dirname "$0")/.."
export MOONITOR_QA_DIR="${MOONITOR_QA_DIR:-$PWD/.local/qa/w1/visual}"
mkdir -p "$MOONITOR_QA_DIR"
for test in trust_regressions_have_visible_feedback_and_scoped_actions virtualized_scroll_and_filter_keep_the_real_widget_state_in_sync setup_form_keyboard_and_save fleet_preview_sort_filter_open_and_empty_host theme_switch_preserves_device_camera_and_resource_context list_alternative_selection_and_theme changing_metrics_keep_visible_glyphs metric_rendering scene_picking_filter_keyboard_and_scroll visual_matrix_and_interactions; do
  MOONITOR_QA_CASE=fleet cargo test --locked -p moonitor-desktop "ui::qa::$test" -- --ignored --exact --test-threads=1
done
for size in 1280x800 1440x900 3840x2160; do
  for tab in overview fleet host metrics workloads energy events explore settings ai history processes; do
    for theme in light dark; do
      MOONITOR_QA_CASE="$tab-$size-$theme" cargo test --locked -p moonitor-desktop ui::qa::visual_matrix_and_interactions -- --ignored --exact --test-threads=1
    done
  done
done
