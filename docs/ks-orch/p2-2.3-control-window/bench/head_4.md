
### head_4

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.096 | 0.100 | 0.200 | 87.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.155 | 0.200 | 0.300 | 4.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.079 | 0.100 | 0.100 | 1.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.161 | 4.800 | 8.200 | 142.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.967 | 4.800 | 38.900 | 102.100 |
| R6/5 | Hold(5) lateness | 10 | 460.950 | 490.700 | 747.700 | 747.700 |
| R6/20 | Hold(20) lateness | 10 | 354.450 | 370.900 | 603.900 | 603.900 |
| R6/50 | Hold(50) lateness | 10 | 319.120 | 363.600 | 471.500 | 471.500 |
| R6/200 | Hold(200) lateness | 10 | 412.810 | 416.100 | 816.300 | 816.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 13.620 | 10.700 | 49.700 | 173.300 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.453 | 4.800 | 12.800 | 296.300 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 7.250 | 6.400 | 27.400 | 273.900 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 6.043 | 5.400 | 15.600 | 145.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.838 | 4.800 | 5.600 | 33.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.319 | 10.700 | 23.800 | 81.200 |
