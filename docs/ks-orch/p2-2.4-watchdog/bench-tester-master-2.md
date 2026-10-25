
### master-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 45.500 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.159 | 0.200 | 0.300 | 2.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.152 | 4.800 | 6.700 | 472.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.900 | 4.800 | 5.400 | 18.500 |
| R6/5 | Hold(5) lateness | 10 | 402.100 | 310.400 | 1098.500 | 1098.500 |
| R6/20 | Hold(20) lateness | 10 | 325.020 | 345.500 | 547.000 | 547.000 |
| R6/50 | Hold(50) lateness | 10 | 357.690 | 311.800 | 642.900 | 642.900 |
| R6/200 | Hold(200) lateness | 10 | 938.050 | 366.000 | 5831.700 | 5831.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.189 | 10.500 | 22.600 | 41.300 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.019 | 4.800 | 6.000 | 186.900 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.353 | 5.000 | 6.700 | 328.800 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 6.410 | 5.300 | 28.600 | 503.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.198 | 4.700 | 6.600 | 320.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.531 | 10.500 | 21.400 | 122.700 |
