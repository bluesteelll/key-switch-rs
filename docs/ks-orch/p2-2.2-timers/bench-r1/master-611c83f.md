
### master-611c83f run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.138 | 0.100 | 0.300 | 141.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.141 | 0.100 | 0.300 | 69.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.300 | 29.200 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.545 | 5.300 | 64.000 | 1321.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 47.640 | 39.500 | 158.900 | 491.500 |
| R6/5 | Hold(5) lateness | 10 | 685.760 | 676.400 | 1090.600 | 1090.600 |
| R6/20 | Hold(20) lateness | 10 | 714.080 | 695.200 | 957.700 | 957.700 |
| R6/50 | Hold(50) lateness | 10 | 848.400 | 778.300 | 1334.600 | 1334.600 |
| R6/200 | Hold(200) lateness | 10 | 624.200 | 558.700 | 909.100 | 909.100 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 15.218 | 12.600 | 66.400 | 457.900 |

### master-611c83f run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.095 | 0.100 | 0.200 | 48.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.176 | 0.200 | 0.500 | 21.100 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.061 | 0.100 | 0.100 | 0.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.169 | 5.200 | 34.100 | 791.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 41.119 | 30.500 | 161.500 | 779.800 |
| R6/5 | Hold(5) lateness | 10 | 762.850 | 579.900 | 1589.500 | 1589.500 |
| R6/20 | Hold(20) lateness | 10 | 526.790 | 546.500 | 719.800 | 719.800 |
| R6/50 | Hold(50) lateness | 10 | 634.240 | 607.800 | 1086.700 | 1086.700 |
| R6/200 | Hold(200) lateness | 10 | 910.240 | 784.300 | 1545.800 | 1545.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 20.464 | 14.300 | 108.700 | 945.500 |

### master-611c83f run 3

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.168 | 0.100 | 0.300 | 687.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.173 | 0.200 | 0.500 | 13.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.200 | 3.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.642 | 5.400 | 54.600 | 2285.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 38.372 | 35.300 | 133.700 | 137.000 |
| R6/5 | Hold(5) lateness | 10 | 943.690 | 740.300 | 3013.000 | 3013.000 |
| R6/20 | Hold(20) lateness | 10 | 614.330 | 673.300 | 922.000 | 922.000 |
| R6/50 | Hold(50) lateness | 10 | 564.440 | 542.200 | 1047.000 | 1047.000 |
| R6/200 | Hold(200) lateness | 10 | 790.060 | 773.500 | 1472.500 | 1472.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 15.438 | 12.300 | 74.500 | 556.700 |
