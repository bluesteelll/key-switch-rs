
### head_1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.109 | 0.100 | 0.300 | 92.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.175 | 0.200 | 0.400 | 48.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.088 | 0.100 | 0.200 | 10.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 6.099 | 5.300 | 14.600 | 414.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.668 | 5.400 | 46.300 | 64.900 |
| R6/5 | Hold(5) lateness | 10 | 439.000 | 349.600 | 690.500 | 690.500 |
| R6/20 | Hold(20) lateness | 10 | 473.050 | 464.900 | 685.700 | 685.700 |
| R6/50 | Hold(50) lateness | 10 | 436.350 | 435.900 | 633.400 | 633.400 |
| R6/200 | Hold(200) lateness | 10 | 420.090 | 382.000 | 714.000 | 714.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.887 | 14.600 | 68.900 | 188.700 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 24.164 | 15.600 | 280.000 | 423.400 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 35.144 | 19.300 | 316.100 | 1013.800 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 35.896 | 26.400 | 325.000 | 471.500 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 47.890 | 30.400 | 367.300 | 1037.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 7.730 | 6.300 | 36.700 | 395.700 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 10.221 | 8.300 | 43.200 | 1141.200 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 8.520 | 7.100 | 41.900 | 375.600 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 8.427 | 6.200 | 40.700 | 460.100 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 23.958 | 17.100 | 141.400 | 560.400 |
