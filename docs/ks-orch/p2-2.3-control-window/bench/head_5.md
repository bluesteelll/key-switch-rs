
### head_5

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.099 | 0.100 | 0.200 | 26.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.153 | 0.200 | 0.300 | 8.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.555 | 4.900 | 10.500 | 512.300 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.242 | 4.900 | 7.100 | 53.800 |
| R6/5 | Hold(5) lateness | 10 | 378.520 | 308.900 | 807.900 | 807.900 |
| R6/20 | Hold(20) lateness | 10 | 382.870 | 390.500 | 571.000 | 571.000 |
| R6/50 | Hold(50) lateness | 10 | 341.400 | 303.900 | 574.800 | 574.800 |
| R6/200 | Hold(200) lateness | 10 | 444.510 | 383.000 | 681.100 | 681.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 14.807 | 13.300 | 35.400 | 211.900 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 6.703 | 6.200 | 9.600 | 234.700 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 6.983 | 6.200 | 14.800 | 596.000 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 6.156 | 5.400 | 16.500 | 69.600 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.358 | 4.800 | 7.900 | 39.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.838 | 10.900 | 27.400 | 76.400 |
