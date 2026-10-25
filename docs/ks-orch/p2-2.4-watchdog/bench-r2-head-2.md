
### head-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 29.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.156 | 0.200 | 0.300 | 7.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.091 | 4.800 | 6.600 | 296.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.916 | 4.800 | 5.200 | 20.400 |
| R6/5 | Hold(5) lateness | 10 | 362.730 | 363.900 | 556.600 | 556.600 |
| R6/20 | Hold(20) lateness | 10 | 300.160 | 206.900 | 571.500 | 571.500 |
| R6/50 | Hold(50) lateness | 10 | 234.600 | 189.600 | 442.900 | 442.900 |
| R6/200 | Hold(200) lateness | 10 | 446.430 | 426.900 | 672.500 | 672.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.879 | 10.500 | 17.200 | 70.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.743 | 4.700 | 4.800 | 30.700 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.172 | 5.100 | 5.900 | 118.700 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.348 | 5.300 | 5.400 | 41.900 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.948 | 4.700 | 4.800 | 239.100 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.225 | 10.600 | 18.300 | 246.800 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 6.035 | 4.800 | 38.500 | 261.300 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 7.007 | 4.800 | 107.100 | 303.100 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.980 | 3.900 | 5.400 | 20.000 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 45.445 | 52.700 | 101.200 | 329.600 |
