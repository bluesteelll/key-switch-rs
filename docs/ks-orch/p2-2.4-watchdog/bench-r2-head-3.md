
### head-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.094 | 0.100 | 0.200 | 27.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.151 | 0.100 | 0.300 | 3.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.082 | 0.100 | 0.100 | 1.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 4.983 | 4.700 | 6.000 | 208.500 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.852 | 4.800 | 6.400 | 11.900 |
| R6/5 | Hold(5) lateness | 10 | 254.000 | 58.600 | 842.600 | 842.600 |
| R6/20 | Hold(20) lateness | 10 | 265.320 | 180.800 | 515.300 | 515.300 |
| R6/50 | Hold(50) lateness | 10 | 288.370 | 290.300 | 535.300 | 535.300 |
| R6/200 | Hold(200) lateness | 10 | 303.050 | 299.500 | 555.900 | 555.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.947 | 10.600 | 16.300 | 96.600 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.841 | 4.700 | 5.900 | 148.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.299 | 5.000 | 6.400 | 144.800 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.401 | 5.300 | 6.700 | 137.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.685 | 4.700 | 4.800 | 6.900 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.874 | 10.700 | 15.200 | 26.200 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 6.316 | 4.800 | 66.900 | 217.000 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 6.088 | 4.800 | 56.400 | 313.700 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.932 | 3.800 | 4.700 | 15.600 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 37.353 | 46.100 | 121.900 | 154.400 |
