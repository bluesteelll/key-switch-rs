
### head-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 43.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.153 | 0.200 | 0.300 | 1.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 3.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.124 | 4.800 | 6.600 | 258.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.960 | 4.800 | 6.200 | 25.300 |
| R6/5 | Hold(5) lateness | 10 | 330.080 | 206.200 | 999.300 | 999.300 |
| R6/20 | Hold(20) lateness | 10 | 330.110 | 307.600 | 562.300 | 562.300 |
| R6/50 | Hold(50) lateness | 10 | 336.280 | 280.900 | 603.400 | 603.400 |
| R6/200 | Hold(200) lateness | 10 | 369.220 | 308.700 | 693.700 | 693.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.903 | 10.900 | 24.400 | 203.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.880 | 4.700 | 6.400 | 166.400 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.278 | 5.000 | 6.500 | 159.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.484 | 5.300 | 6.700 | 157.500 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.830 | 4.700 | 4.800 | 79.100 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.187 | 11.000 | 30.800 | 205.300 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 23.876 | 22.700 | 62.200 | 135.700 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 6.949 | 5.000 | 73.700 | 254.700 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.181 | 3.900 | 6.200 | 37.500 |
