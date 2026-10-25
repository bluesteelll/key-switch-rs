
### prev-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.096 | 0.100 | 0.200 | 25.000 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.154 | 0.200 | 0.300 | 2.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.079 | 0.100 | 0.100 | 2.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.048 | 4.800 | 6.100 | 245.800 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.934 | 4.900 | 5.000 | 20.500 |
| R6/5 | Hold(5) lateness | 10 | 334.900 | 321.700 | 535.400 | 535.400 |
| R6/20 | Hold(20) lateness | 10 | 363.310 | 361.400 | 577.500 | 577.500 |
| R6/50 | Hold(50) lateness | 10 | 369.300 | 357.200 | 598.800 | 598.800 |
| R6/200 | Hold(200) lateness | 10 | 296.430 | 252.800 | 509.800 | 509.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.912 | 10.600 | 27.800 | 253.900 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.794 | 4.800 | 4.800 | 44.400 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.472 | 5.100 | 12.500 | 81.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.501 | 5.300 | 6.700 | 56.500 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.735 | 4.700 | 4.800 | 6.000 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.972 | 10.600 | 30.600 | 64.700 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 22.204 | 21.400 | 52.900 | 176.000 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 7.292 | 5.000 | 96.800 | 302.000 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.867 | 3.800 | 4.100 | 10.100 |
