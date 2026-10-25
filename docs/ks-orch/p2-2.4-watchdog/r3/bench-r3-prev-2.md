
### prev-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.093 | 0.100 | 0.200 | 28.000 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.148 | 0.100 | 0.200 | 4.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 4.980 | 4.700 | 6.600 | 283.300 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.875 | 4.800 | 7.500 | 18.900 |
| R6/5 | Hold(5) lateness | 10 | 305.150 | 242.000 | 636.900 | 636.900 |
| R6/20 | Hold(20) lateness | 10 | 327.300 | 227.600 | 547.900 | 547.900 |
| R6/50 | Hold(50) lateness | 10 | 342.490 | 287.500 | 581.300 | 581.300 |
| R6/200 | Hold(200) lateness | 10 | 405.910 | 394.900 | 567.700 | 567.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 14.130 | 10.500 | 42.800 | 93.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.838 | 4.700 | 5.900 | 115.900 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.100 | 4.900 | 6.300 | 68.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.275 | 5.200 | 5.300 | 66.500 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.681 | 4.700 | 4.800 | 7.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.756 | 10.400 | 17.200 | 35.200 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 6.026 | 4.700 | 48.500 | 293.700 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.324 | 4.700 | 7.900 | 262.700 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.947 | 3.800 | 4.800 | 17.600 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 41.546 | 50.700 | 124.800 | 236.900 |
