
### master_1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.101 | 0.100 | 0.300 | 55.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.162 | 0.200 | 0.300 | 5.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 0.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.027 | 4.800 | 7.900 | 411.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.219 | 4.900 | 5.700 | 45.800 |
| R6/5 | Hold(5) lateness | 10 | 381.050 | 303.200 | 825.900 | 825.900 |
| R6/20 | Hold(20) lateness | 10 | 324.910 | 364.500 | 586.500 | 586.500 |
| R6/50 | Hold(50) lateness | 10 | 335.820 | 313.300 | 548.800 | 548.800 |
| R6/200 | Hold(200) lateness | 10 | 363.700 | 278.200 | 593.000 | 593.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 12.550 | 10.600 | 51.000 | 223.600 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 25.706 | 16.500 | 166.100 | 333.000 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 29.209 | 19.500 | 194.300 | 423.300 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 37.459 | 27.000 | 179.600 | 346.600 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 41.105 | 30.100 | 249.500 | 550.900 |
