
### master-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.102 | 0.100 | 0.200 | 64.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.158 | 0.200 | 0.300 | 7.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 20.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.105 | 4.800 | 8.400 | 170.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.888 | 4.800 | 5.900 | 10.300 |
| R6/5 | Hold(5) lateness | 10 | 311.860 | 360.600 | 517.200 | 517.200 |
| R6/20 | Hold(20) lateness | 10 | 361.790 | 333.500 | 522.100 | 522.100 |
| R6/50 | Hold(50) lateness | 10 | 411.150 | 351.900 | 578.700 | 578.700 |
| R6/200 | Hold(200) lateness | 10 | 288.290 | 276.800 | 660.800 | 660.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.505 | 10.500 | 28.700 | 210.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.825 | 4.700 | 6.600 | 93.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.056 | 5.000 | 5.300 | 81.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.617 | 5.300 | 6.700 | 199.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.709 | 4.700 | 4.800 | 17.900 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.869 | 10.500 | 19.600 | 36.600 |
