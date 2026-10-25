
### master-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.096 | 0.100 | 0.200 | 49.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.158 | 0.200 | 0.300 | 3.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.446 | 4.800 | 8.800 | 277.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 8.038 | 7.900 | 8.500 | 43.300 |
| R6/5 | Hold(5) lateness | 10 | 291.540 | 296.000 | 533.400 | 533.400 |
| R6/20 | Hold(20) lateness | 10 | 428.570 | 483.700 | 714.100 | 714.100 |
| R6/50 | Hold(50) lateness | 10 | 398.170 | 388.600 | 623.300 | 623.300 |
| R6/200 | Hold(200) lateness | 10 | 338.530 | 371.300 | 760.900 | 760.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.431 | 10.700 | 24.700 | 203.600 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.866 | 4.800 | 6.000 | 120.300 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.136 | 5.100 | 5.200 | 100.500 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.417 | 5.400 | 5.500 | 104.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.345 | 4.800 | 9.500 | 324.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.451 | 10.900 | 23.900 | 132.500 |
