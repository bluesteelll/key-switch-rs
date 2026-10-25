
### head_3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.098 | 0.100 | 0.200 | 24.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.154 | 0.200 | 0.300 | 8.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.080 | 0.100 | 0.100 | 2.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.287 | 4.800 | 9.600 | 246.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.898 | 4.900 | 28.400 | 36.500 |
| R6/5 | Hold(5) lateness | 10 | 363.810 | 306.500 | 581.000 | 581.000 |
| R6/20 | Hold(20) lateness | 10 | 274.470 | 183.100 | 573.200 | 573.200 |
| R6/50 | Hold(50) lateness | 10 | 376.300 | 379.500 | 591.400 | 591.400 |
| R6/200 | Hold(200) lateness | 10 | 414.150 | 397.500 | 672.800 | 672.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 13.631 | 10.800 | 52.600 | 377.300 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 31.427 | 17.100 | 260.800 | 417.000 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 31.833 | 20.900 | 218.700 | 333.600 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 45.065 | 28.200 | 356.600 | 427.300 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 43.379 | 31.500 | 269.500 | 344.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.827 | 4.900 | 19.500 | 155.300 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 7.629 | 6.500 | 29.700 | 267.100 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.986 | 5.400 | 9.600 | 459.600 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.052 | 4.800 | 9.000 | 83.900 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.257 | 10.500 | 35.500 | 79.100 |
