
### head-2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 24.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.154 | 0.200 | 0.300 | 1.900 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 2.700 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.040 | 4.800 | 6.500 | 413.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.859 | 4.800 | 4.900 | 17.200 |
| R6/5 | Hold(5) lateness | 10 | 283.890 | 266.400 | 548.000 | 548.000 |
| R6/20 | Hold(20) lateness | 10 | 349.140 | 318.900 | 539.000 | 539.000 |
| R6/50 | Hold(50) lateness | 10 | 266.310 | 206.100 | 562.100 | 562.100 |
| R6/200 | Hold(200) lateness | 10 | 385.060 | 323.200 | 723.900 | 723.900 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.212 | 10.500 | 26.700 | 251.900 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.889 | 4.700 | 5.900 | 215.800 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.219 | 5.000 | 6.400 | 191.800 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.635 | 5.300 | 7.700 | 185.500 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.717 | 4.700 | 4.800 | 19.100 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 10.951 | 10.700 | 18.500 | 28.100 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 22.283 | 21.500 | 53.700 | 141.800 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 7.406 | 4.900 | 104.600 | 502.800 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.189 | 3.900 | 6.400 | 148.800 |
