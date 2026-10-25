
### prev-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.096 | 0.100 | 0.200 | 47.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.147 | 0.100 | 0.300 | 1.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 3.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.079 | 4.800 | 6.600 | 335.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.932 | 4.900 | 5.000 | 20.500 |
| R6/5 | Hold(5) lateness | 10 | 334.760 | 461.200 | 516.100 | 516.100 |
| R6/20 | Hold(20) lateness | 10 | 326.430 | 303.300 | 518.300 | 518.300 |
| R6/50 | Hold(50) lateness | 10 | 265.180 | 212.300 | 486.600 | 486.600 |
| R6/200 | Hold(200) lateness | 10 | 343.040 | 261.800 | 583.400 | 583.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.926 | 10.600 | 16.500 | 142.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.774 | 4.700 | 4.800 | 43.000 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.096 | 5.100 | 5.200 | 37.900 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.599 | 5.300 | 6.700 | 230.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.729 | 4.700 | 4.800 | 7.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.964 | 10.800 | 13.300 | 55.000 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 22.746 | 22.300 | 39.600 | 76.900 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 5.508 | 5.000 | 7.500 | 211.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.165 | 3.900 | 7.800 | 62.200 |
