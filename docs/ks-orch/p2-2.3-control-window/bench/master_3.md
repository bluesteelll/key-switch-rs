
### master_3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 28.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.152 | 0.200 | 0.300 | 3.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.076 | 0.100 | 0.100 | 0.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.022 | 4.700 | 8.500 | 152.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.222 | 4.800 | 19.100 | 25.900 |
| R6/5 | Hold(5) lateness | 10 | 390.780 | 423.100 | 614.200 | 614.200 |
| R6/20 | Hold(20) lateness | 10 | 374.660 | 325.900 | 843.500 | 843.500 |
| R6/50 | Hold(50) lateness | 10 | 395.210 | 383.600 | 643.100 | 643.100 |
| R6/200 | Hold(200) lateness | 10 | 408.760 | 367.900 | 682.100 | 682.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 12.917 | 10.800 | 51.600 | 131.500 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 19.239 | 13.900 | 208.400 | 297.200 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 36.188 | 22.600 | 298.500 | 478.500 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 30.459 | 24.000 | 242.500 | 370.400 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 50.649 | 34.000 | 355.800 | 605.100 |
