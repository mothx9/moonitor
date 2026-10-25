# Operator tasks · behavioral reference

25 October 2026. Sources are official project/vendor documentation. This is a
documentary task analysis, **not a timed hands-on competitor benchmark**. Click
counts and completion time on those products were not measured and are not
fabricated. The Moonitor targets below are hypotheses to test in S3/S6.

| Task | Reference behavior / source | Moonitor contract and test target |
|---|---|---|
| 1. Find the host needing attention | Checkmk filtered object views and drilldown [1] | Slim health/filter strip → exact row; five mixed hosts visible, ≤2 activations. |
| 2. Find process using CPU | Activity Monitor search, sort headings and selected-process details [2]; btop process navigation/filter [3] | Sort once, search in place, named selected row; bounded sample and CPU denominator shown. No invented process census. |
| 3. Open selected object | Activity Monitor double-click/Info [2]; Checkmk contextual object views [1] | Select previews, Enter/double-click opens same stable ID. Filter hidden selection disables contextual Open. |
| 4. Compare CPU/GPU/physical RAM | btop simultaneous resource panels [3] | Stable host resource composition; one screen shows relevant signals; GPU-less node reclaims space. |
| 5. Change network scope | btop explicit interface controls [3] | Labeled pinned interface or documented aggregation; never switch to max traffic silently. |
| 6. Inspect a historical value | Grafana Explore time controls/query inspector [4]; Netdata chart dimensions/time navigation [5] | Shared crosshair, exact received/source timestamp, unit and source; ≤3 activations to evidence. |
| 7. Compare two hosts in same interval | Grafana Explore multiple queries/common time [4] | Same TimeCursor, labeled host/component series, missing intervals remain gaps. |
| 8. Recover from no results | Checkmk view filtering [1]; Activity Monitor search [2] | Explain active filter; clear action; preserved underlying inventory and no hidden-host opening. |
| 9. Investigate a failed source | Checkmk state/object separation [1] | Invalid thermal field with raw reason; valid host metrics continue; stale distinct from unreachable. |
| 10. Follow model residency without assuming a run | Existing Moonitor Ollama adapter is `/api/ps`, not request telemetry | Model residency labeled explicitly; queue/tokens/TTFT unavailable until measured adapters exist. |
| 11. Configure and cancel a change | Moonitor actual supported profile CRUD, inspected code and native task | All five hosts visible; explicit draft/effective distinction; failure keeps form, Cancel restores context, no default destructive action. |
| 12. Freeze inspection while collection continues | No competitor behavior assumed; Moonitor-specific trust requirement | Frozen time plus independent health; ≥60 s, DB arrivals for every healthy agent; S0 replay documented. |

References inspected:

1. [Checkmk views](https://docs.checkmk.com/latest/en/views.html)
2. [Activity Monitor process inspection](https://support.apple.com/en-ie/guide/activity-monitor/actmntr1001/mac)
3. [btop project, features and controls](https://github.com/aristocratos/btop)
4. [Grafana Explore](https://grafana.com/docs/grafana/latest/visualizations/explore/get-started-with-explore/)
5. [Netdata dashboards and chart dimensions](https://learn.netdata.cloud/docs/dashboards-and-charts/tabs/dashboards)

Next measurement sheet: task ID, build SHA, participant, OS/scale/viewport/theme,
starting selection/filter/time, exact actions, elapsed time, wrong-object actions,
backtracking, task success, retained context, optical/AX defects and evidence path.
An automated replay is developer evidence; a separate owner/operator review is
still required for composition approval. No human task timings are asserted here.
