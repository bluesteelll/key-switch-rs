
### head_6

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.100 | 0.100 | 0.200 | 25.000 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.155 | 0.200 | 0.300 | 4.200 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 1.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.426 | 4.900 | 11.300 | 269.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.014 | 4.900 | 6.800 | 16.400 |
| R6/5 | Hold(5) lateness | 10 | 355.320 | 421.500 | 576.800 | 576.800 |
| R6/20 | Hold(20) lateness | 10 | 434.700 | 368.400 | 908.900 | 908.900 |
| R6/50 | Hold(50) lateness | 10 | 440.650 | 466.500 | 563.500 | 563.500 |
| R6/200 | Hold(200) lateness | 10 | 396.740 | 462.300 | 581.900 | 581.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.914 | 10.700 | 31.000 | 263.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.314 | 4.800 | 9.800 | 68.100 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.632 | 5.100 | 15.200 | 107.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.941 | 5.400 | 13.500 | 174.700 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.992 | 4.800 | 7.100 | 40.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.060 | 10.700 | 39.100 | 63.200 |
