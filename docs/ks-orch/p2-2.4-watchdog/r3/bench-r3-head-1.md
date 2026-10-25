
### head-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.100 | 0.100 | 0.200 | 55.000 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.198 | 0.200 | 0.400 | 5.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 2.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.186 | 4.900 | 6.700 | 204.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.946 | 4.900 | 5.100 | 8.600 |
| R6/5 | Hold(5) lateness | 10 | 378.980 | 435.900 | 487.300 | 487.300 |
| R6/20 | Hold(20) lateness | 10 | 309.350 | 243.100 | 604.600 | 604.600 |
| R6/50 | Hold(50) lateness | 10 | 423.430 | 441.400 | 476.600 | 476.600 |
| R6/200 | Hold(200) lateness | 10 | 358.830 | 370.200 | 586.700 | 586.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.006 | 10.700 | 19.800 | 40.100 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.117 | 4.900 | 7.000 | 168.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.203 | 5.100 | 6.200 | 136.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.466 | 5.400 | 6.300 | 77.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.124 | 4.800 | 6.000 | 281.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.230 | 10.600 | 19.600 | 193.400 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.690 | 4.900 | 8.100 | 286.200 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.421 | 4.900 | 9.200 | 154.000 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.256 | 3.900 | 6.500 | 151.300 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 38.109 | 44.200 | 114.800 | 230.100 |
