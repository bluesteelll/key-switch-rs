
### head-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 28.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.152 | 0.200 | 0.300 | 2.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.075 | 0.100 | 0.100 | 1.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.062 | 4.800 | 6.700 | 325.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.897 | 4.900 | 5.400 | 10.900 |
| R6/5 | Hold(5) lateness | 10 | 230.800 | 162.300 | 381.700 | 381.700 |
| R6/20 | Hold(20) lateness | 10 | 297.140 | 285.500 | 395.500 | 395.500 |
| R6/50 | Hold(50) lateness | 10 | 262.630 | 236.000 | 365.300 | 365.300 |
| R6/200 | Hold(200) lateness | 10 | 315.260 | 246.500 | 679.000 | 679.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.653 | 10.500 | 15.600 | 89.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.759 | 4.700 | 4.800 | 26.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.132 | 5.100 | 5.200 | 88.500 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.526 | 5.300 | 6.700 | 74.200 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.848 | 4.700 | 4.900 | 24.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.247 | 10.300 | 12.800 | 25.500 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.404 | 4.800 | 9.200 | 189.900 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.425 | 4.800 | 9.300 | 253.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.954 | 3.900 | 4.800 | 15.700 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 37.804 | 47.700 | 91.800 | 170.800 |
