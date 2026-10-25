
### dev e69ce7f run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.103 | 0.100 | 0.300 | 62.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.165 | 0.200 | 0.400 | 9.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.100 | 1.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.244 | 4.900 | 7.900 | 203.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.139 | 4.900 | 8.100 | 24.800 |
| R6/5 | Hold(5) lateness | 10 | 260.320 | 249.800 | 601.900 | 601.900 |
| R6/20 | Hold(20) lateness | 10 | 290.130 | 248.500 | 505.700 | 505.700 |
| R6/50 | Hold(50) lateness | 10 | 355.990 | 336.700 | 557.800 | 557.800 |
| R6/200 | Hold(200) lateness | 10 | 371.000 | 372.600 | 552.400 | 552.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.927 | 10.500 | 30.000 | 79.700 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.190 | 4.700 | 5.900 | 330.600 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.301 | 5.000 | 5.400 | 288.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.345 | 5.300 | 5.400 | 62.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.956 | 4.700 | 5.900 | 234.600 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.135 | 10.500 | 22.900 | 240.400 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 7.422 | 4.800 | 102.300 | 327.100 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 7.520 | 4.800 | 98.500 | 366.000 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.069 | 3.900 | 6.000 | 20.500 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 47.419 | 53.500 | 168.400 | 314.300 |
