
### head-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.094 | 0.100 | 0.200 | 24.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.152 | 0.200 | 0.300 | 3.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.075 | 0.100 | 0.100 | 2.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.045 | 4.800 | 6.600 | 130.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.839 | 4.800 | 5.100 | 10.000 |
| R6/5 | Hold(5) lateness | 10 | 268.900 | 252.200 | 473.200 | 473.200 |
| R6/20 | Hold(20) lateness | 10 | 272.600 | 295.500 | 427.800 | 427.800 |
| R6/50 | Hold(50) lateness | 10 | 1332.310 | 324.400 | 9051.400 | 9051.400 |
| R6/200 | Hold(200) lateness | 10 | 375.800 | 425.200 | 610.300 | 610.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.230 | 10.300 | 14.600 | 27.700 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.828 | 4.600 | 5.900 | 188.200 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.041 | 4.900 | 6.300 | 85.700 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.227 | 5.200 | 5.900 | 23.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.818 | 4.600 | 5.900 | 104.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.536 | 10.300 | 15.400 | 58.700 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.753 | 4.800 | 14.800 | 257.600 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.976 | 4.700 | 26.500 | 436.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.048 | 3.800 | 5.100 | 210.600 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 45.294 | 52.700 | 145.800 | 237.400 |
