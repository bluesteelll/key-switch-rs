# Developer diagnostics of bench row R16 (fix round 1; not committed code)

Rows of temporary ignored tests appended to src/bench.rs (and, for D-pre, to a scratch worktree at 0e7165b), run once each in release builds and removed again. D-A..D-D at e69ce7f; D-pre at 0e7165b; the priority rows at e69ce7f.

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R12 | R12 | 2000 | 5.100 | 4.700 | 6.200 | 170.800 |
| D-A | R16 as is | 200 | 45.550 | 51.700 | 146.900 | 234.000 |
| D-B | R16, hook not installed (no raw_batch, no timer set) | 200 | 57.270 | 53.600 | 136.900 | 178.700 |
| D-C | R16, call 200 us after the post | 200 | 25.156 | 16.500 | 200.200 | 380.000 |
| D-D | R16, warm thread | 20 | 83.490 | 79.700 | 216.300 | 216.300 |

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R12 | R12 | 2000 | 5.123 | 4.800 | 6.100 | 105.800 |
| D-pre | R16's construction on 0e7165b (one wake per key) | 200 | 38.615 | 35.600 | 110.900 | 127.500 |
### no pump

No table: the diagnostic loop without the pump never took its posted WM_QUIT (a filtered peek leaves it in the queue), so its first stop timed out and the run failed. See fix_r1.md and the test a_range_filtered_peek_leaves_a_posted_wm_quit_for_the_pump.

### priority diagnostic

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R12 | normal priority | 2000 | 5.067 | 4.800 | 7.300 | 78.900 |
| R16 | normal priority | 200 | 58.259 | 58.000 | 156.800 | 201.700 |
| R12/high | bench thread HIGHEST, as the daemon's main | 2000 | 5.179 | 4.700 | 7.800 | 247.300 |
| R14/high | bench thread HIGHEST | 2000 | 7.667 | 4.800 | 89.400 | 602.100 |
| R14c/high | bench thread HIGHEST | 2000 | 7.428 | 4.900 | 57.300 | 977.500 |
| R16/high | bench thread HIGHEST | 200 | 57.053 | 58.500 | 149.700 | 164.100 |
