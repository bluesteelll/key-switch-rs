
### master_2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.165 | 0.100 | 0.500 | 152.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.279 | 0.200 | 1.200 | 16.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.117 | 0.100 | 0.400 | 5.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.527 | 6.200 | 28.000 | 749.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.484 | 4.900 | 7.800 | 37.200 |
| R6/5 | Hold(5) lateness | 10 | 534.850 | 503.100 | 823.200 | 823.200 |
| R6/20 | Hold(20) lateness | 10 | 459.830 | 502.700 | 616.800 | 616.800 |
| R6/50 | Hold(50) lateness | 10 | 410.840 | 340.100 | 726.100 | 726.100 |
| R6/200 | Hold(200) lateness | 10 | 420.140 | 313.000 | 650.400 | 650.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 19.001 | 16.400 | 52.800 | 130.100 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 25.103 | 15.900 | 213.900 | 467.000 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 35.424 | 20.000 | 250.500 | 436.700 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 37.992 | 27.600 | 226.900 | 522.800 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 49.653 | 31.500 | 292.100 | 496.700 |
