
### head-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 25.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.148 | 0.100 | 0.300 | 1.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 2.200 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.095 | 4.800 | 6.700 | 174.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.860 | 4.800 | 4.900 | 17.200 |
| R6/5 | Hold(5) lateness | 10 | 319.610 | 299.600 | 676.200 | 676.200 |
| R6/20 | Hold(20) lateness | 10 | 365.450 | 249.200 | 1020.100 | 1020.100 |
| R6/50 | Hold(50) lateness | 10 | 455.910 | 497.700 | 615.800 | 615.800 |
| R6/200 | Hold(200) lateness | 10 | 290.690 | 283.800 | 546.800 | 546.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 12.346 | 10.700 | 30.800 | 323.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.011 | 4.800 | 6.000 | 155.200 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 7.118 | 7.400 | 11.300 | 227.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 8.132 | 8.200 | 9.100 | 332.600 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.032 | 4.800 | 7.000 | 10.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.865 | 10.500 | 39.500 | 262.900 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 24.777 | 23.100 | 65.900 | 342.700 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 8.678 | 5.000 | 153.500 | 533.100 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.018 | 3.800 | 5.300 | 115.800 |
