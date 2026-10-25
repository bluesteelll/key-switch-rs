
### head_2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.094 | 0.100 | 0.200 | 22.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.152 | 0.200 | 0.300 | 7.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.076 | 0.100 | 0.100 | 0.800 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.203 | 4.800 | 9.800 | 309.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.605 | 4.900 | 25.300 | 71.300 |
| R6/5 | Hold(5) lateness | 10 | 437.870 | 507.400 | 600.400 | 600.400 |
| R6/20 | Hold(20) lateness | 10 | 339.750 | 273.400 | 601.400 | 601.400 |
| R6/50 | Hold(50) lateness | 10 | 436.800 | 439.800 | 572.200 | 572.200 |
| R6/200 | Hold(200) lateness | 10 | 347.590 | 278.400 | 582.300 | 582.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.734 | 10.500 | 43.100 | 74.700 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 20.609 | 14.400 | 209.900 | 333.500 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 29.048 | 18.200 | 288.700 | 387.400 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 31.691 | 24.700 | 234.500 | 390.200 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 40.430 | 28.900 | 319.300 | 403.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.490 | 4.800 | 8.100 | 303.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 6.759 | 6.300 | 14.800 | 90.100 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.661 | 5.400 | 10.200 | 57.700 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.894 | 4.800 | 6.400 | 32.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.721 | 10.300 | 36.500 | 119.600 |
