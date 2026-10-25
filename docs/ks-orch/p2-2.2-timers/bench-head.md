
### HEAD 2f11917, run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.150 | 0.100 | 0.500 | 127.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.295 | 0.200 | 0.500 | 339.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.120 | 0.100 | 0.200 | 3.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.531 | 7.000 | 61.500 | 1031.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 9.140 | 5.900 | 115.500 | 198.100 |
| R6/5 | Hold(5) lateness | 10 | 491.740 | 502.700 | 711.400 | 711.400 |
| R6/20 | Hold(20) lateness | 10 | 545.340 | 525.000 | 931.700 | 931.700 |
| R6/50 | Hold(50) lateness | 10 | 470.330 | 432.300 | 702.200 | 702.200 |
| R6/200 | Hold(200) lateness | 10 | 388.850 | 378.000 | 662.600 | 662.600 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 25.142 | 16.300 | 193.200 | 1173.200 |

### HEAD 2f11917, run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.130 | 0.100 | 0.400 | 301.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.191 | 0.200 | 0.400 | 14.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.127 | 0.100 | 0.200 | 93.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.396 | 7.300 | 54.900 | 1165.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 13.345 | 7.600 | 72.600 | 926.700 |
| R6/5 | Hold(5) lateness | 10 | 663.990 | 499.500 | 2197.200 | 2197.200 |
| R6/20 | Hold(20) lateness | 10 | 427.680 | 421.200 | 638.000 | 638.000 |
| R6/50 | Hold(50) lateness | 10 | 577.360 | 585.200 | 1047.300 | 1047.300 |
| R6/200 | Hold(200) lateness | 10 | 482.810 | 390.700 | 803.900 | 803.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 22.635 | 14.600 | 242.500 | 430.300 |

### HEAD 2f11917, run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.197 | 0.100 | 0.700 | 202.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.210 | 0.200 | 0.500 | 20.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.136 | 0.100 | 0.300 | 19.100 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.150 | 6.800 | 58.600 | 1324.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 8.420 | 5.900 | 54.200 | 82.500 |
| R6/5 | Hold(5) lateness | 10 | 597.620 | 412.900 | 1478.100 | 1478.100 |
| R6/20 | Hold(20) lateness | 10 | 621.060 | 508.100 | 1121.700 | 1121.700 |
| R6/50 | Hold(50) lateness | 10 | 528.960 | 592.200 | 720.400 | 720.400 |
| R6/200 | Hold(200) lateness | 10 | 404.640 | 294.700 | 706.100 | 706.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 24.160 | 15.800 | 228.600 | 659.200 |
