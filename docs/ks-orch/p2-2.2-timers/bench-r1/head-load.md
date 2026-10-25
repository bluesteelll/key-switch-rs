
### head-d738632 load run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.167 | 0.100 | 0.400 | 304.500 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.175 | 0.200 | 0.400 | 17.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.085 | 0.100 | 0.100 | 2.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.554 | 5.200 | 39.000 | 1441.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.391 | 5.300 | 41.600 | 56.200 |
| R6/5 | Hold(5) lateness | 10 | 487.680 | 496.000 | 778.900 | 778.900 |
| R6/20 | Hold(20) lateness | 10 | 433.370 | 544.500 | 598.600 | 598.600 |
| R6/50 | Hold(50) lateness | 10 | 499.270 | 440.000 | 888.000 | 888.000 |
| R6/200 | Hold(200) lateness | 10 | 557.140 | 575.200 | 777.500 | 777.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.606 | 13.700 | 94.700 | 344.100 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 41.606 | 19.300 | 482.900 | 892.000 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 44.541 | 23.700 | 372.200 | 551.400 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 55.497 | 30.600 | 533.200 | 985.600 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 59.952 | 34.400 | 536.200 | 603.500 |

### head-d738632 load run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.130 | 0.100 | 0.400 | 92.300 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.180 | 0.200 | 0.400 | 20.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.102 | 0.100 | 0.200 | 1.800 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.280 | 5.300 | 33.200 | 658.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 10.395 | 7.000 | 64.500 | 353.400 |
| R6/5 | Hold(5) lateness | 10 | 586.730 | 586.900 | 1007.600 | 1007.600 |
| R6/20 | Hold(20) lateness | 10 | 456.530 | 394.700 | 791.700 | 791.700 |
| R6/50 | Hold(50) lateness | 10 | 543.590 | 465.800 | 1226.500 | 1226.500 |
| R6/200 | Hold(200) lateness | 10 | 477.600 | 393.500 | 888.400 | 888.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.612 | 12.300 | 98.100 | 345.800 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 40.205 | 18.600 | 537.300 | 1018.100 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 38.495 | 23.400 | 281.800 | 322.500 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 54.791 | 29.100 | 561.300 | 1032.000 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 54.820 | 35.400 | 355.400 | 488.500 |

### head-d738632 load run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.361 | 0.100 | 0.400 | 10348.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.258 | 0.300 | 0.500 | 24.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.368 | 0.100 | 0.200 | 1236.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 12.562 | 7.100 | 121.300 | 3395.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 6.681 | 5.400 | 25.900 | 64.000 |
| R6/5 | Hold(5) lateness | 10 | 791.110 | 536.400 | 2336.500 | 2336.500 |
| R6/20 | Hold(20) lateness | 10 | 509.560 | 507.600 | 750.200 | 750.200 |
| R6/50 | Hold(50) lateness | 10 | 537.520 | 509.000 | 820.200 | 820.200 |
| R6/200 | Hold(200) lateness | 10 | 490.140 | 410.900 | 763.600 | 763.600 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 16.854 | 11.300 | 126.600 | 427.200 |
| R8/normal | one-way delivery under load, loop at NORMAL | 300 | 48.926 | 18.900 | 467.500 | 1631.200 |
| R8/highest | one-way delivery under load, loop at HIGHEST | 300 | 39.409 | 20.900 | 386.700 | 499.800 |
| R8/normal-rt | round trip under load, loop at NORMAL | 300 | 67.220 | 29.900 | 597.700 | 1837.500 |
| R8/highest-rt | round trip under load, loop at HIGHEST | 300 | 53.637 | 31.500 | 411.600 | 671.200 |
