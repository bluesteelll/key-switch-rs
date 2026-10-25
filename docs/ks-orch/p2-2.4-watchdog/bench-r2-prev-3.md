
### prev-3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 26.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.157 | 0.200 | 0.300 | 1.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 2.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.059 | 4.800 | 6.600 | 221.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.901 | 4.800 | 5.000 | 16.000 |
| R6/5 | Hold(5) lateness | 10 | 233.170 | 249.300 | 449.600 | 449.600 |
| R6/20 | Hold(20) lateness | 10 | 410.400 | 367.000 | 595.000 | 595.000 |
| R6/50 | Hold(50) lateness | 10 | 346.610 | 316.200 | 548.700 | 548.700 |
| R6/200 | Hold(200) lateness | 10 | 269.790 | 223.500 | 542.800 | 542.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.020 | 10.600 | 18.300 | 97.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.703 | 4.700 | 5.400 | 80.100 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.192 | 5.000 | 6.300 | 191.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.348 | 5.300 | 6.600 | 43.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.955 | 4.700 | 5.900 | 169.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.160 | 10.500 | 20.100 | 195.500 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 21.463 | 21.100 | 46.700 | 150.200 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 7.201 | 5.000 | 95.100 | 384.700 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.306 | 3.900 | 5.200 | 185.600 |
