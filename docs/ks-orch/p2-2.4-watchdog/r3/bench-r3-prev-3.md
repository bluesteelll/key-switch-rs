
### prev-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 26.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.150 | 0.200 | 0.200 | 3.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.080 | 0.100 | 0.100 | 1.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 4.981 | 4.700 | 6.000 | 338.500 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.854 | 4.800 | 4.900 | 12.800 |
| R6/5 | Hold(5) lateness | 10 | 247.060 | 189.500 | 555.600 | 555.600 |
| R6/20 | Hold(20) lateness | 10 | 315.820 | 320.100 | 546.700 | 546.700 |
| R6/50 | Hold(50) lateness | 10 | 334.560 | 301.800 | 513.300 | 513.300 |
| R6/200 | Hold(200) lateness | 10 | 304.040 | 289.500 | 502.700 | 502.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.790 | 10.600 | 17.100 | 22.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.686 | 4.700 | 4.700 | 37.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.026 | 5.000 | 5.100 | 14.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.298 | 5.300 | 5.600 | 45.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.852 | 4.700 | 6.700 | 80.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.035 | 10.400 | 14.900 | 161.400 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.895 | 4.800 | 15.100 | 386.800 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.421 | 4.800 | 8.300 | 230.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.884 | 3.900 | 4.100 | 34.500 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 38.820 | 49.200 | 107.400 | 164.000 |
