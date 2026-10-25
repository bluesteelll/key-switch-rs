
### prev-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 52.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.157 | 0.200 | 0.300 | 9.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.079 | 0.100 | 0.100 | 1.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.134 | 4.900 | 6.700 | 149.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.310 | 5.000 | 30.400 | 127.200 |
| R6/5 | Hold(5) lateness | 10 | 301.350 | 195.300 | 555.900 | 555.900 |
| R6/20 | Hold(20) lateness | 10 | 281.630 | 218.300 | 551.500 | 551.500 |
| R6/50 | Hold(50) lateness | 10 | 357.960 | 329.900 | 580.800 | 580.800 |
| R6/200 | Hold(200) lateness | 10 | 336.220 | 306.000 | 506.100 | 506.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.992 | 10.700 | 15.100 | 90.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.071 | 4.800 | 6.300 | 88.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.163 | 5.100 | 5.600 | 11.700 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.470 | 5.400 | 6.200 | 38.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.844 | 4.800 | 5.900 | 27.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.482 | 10.900 | 20.800 | 132.100 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.664 | 4.900 | 13.900 | 176.500 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.820 | 4.900 | 10.900 | 296.900 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.928 | 3.900 | 4.100 | 67.200 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 37.812 | 43.900 | 119.400 | 178.400 |
