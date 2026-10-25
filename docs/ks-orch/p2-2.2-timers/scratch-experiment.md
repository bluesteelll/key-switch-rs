
### EXPERIMENT master + message queue, run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.115 | 0.100 | 0.300 | 64.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.172 | 0.200 | 0.400 | 48.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.079 | 0.100 | 0.200 | 2.800 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.741 | 5.400 | 46.300 | 671.900 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 69.958 | 45.400 | 326.700 | 379.200 |
| R6/5 | Hold(5) lateness | 10 | 734.500 | 653.500 | 1717.000 | 1717.000 |
| R6/20 | Hold(20) lateness | 10 | 684.230 | 558.500 | 2305.500 | 2305.500 |
| R6/50 | Hold(50) lateness | 10 | 654.420 | 519.600 | 1372.300 | 1372.300 |
| R6/200 | Hold(200) lateness | 10 | 774.310 | 637.600 | 1450.700 | 1450.700 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 20.567 | 14.300 | 148.400 | 445.900 |

### EXPERIMENT master + message queue, run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.151 | 0.100 | 0.400 | 168.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.250 | 0.200 | 0.500 | 80.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.091 | 0.100 | 0.300 | 21.600 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 8.489 | 5.900 | 36.800 | 1218.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 53.923 | 45.000 | 225.000 | 382.800 |
| R6/5 | Hold(5) lateness | 10 | 587.990 | 549.800 | 964.100 | 964.100 |
| R6/20 | Hold(20) lateness | 10 | 608.220 | 497.600 | 1567.200 | 1567.200 |
| R6/50 | Hold(50) lateness | 10 | 711.240 | 653.800 | 1271.400 | 1271.400 |
| R6/200 | Hold(200) lateness | 10 | 730.960 | 728.900 | 1392.000 | 1392.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 16.704 | 13.800 | 69.300 | 175.000 |

### EXPERIMENT HEAD without the fast-action record, run 1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.204 | 0.200 | 0.500 | 176.900 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.303 | 0.300 | 0.700 | 10.700 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.137 | 0.100 | 0.400 | 18.000 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 10.713 | 7.500 | 68.500 | 2443.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 14.151 | 7.400 | 110.800 | 721.900 |
| R6/5 | Hold(5) lateness | 10 | 482.440 | 372.500 | 951.300 | 951.300 |
| R6/20 | Hold(20) lateness | 10 | 504.210 | 497.400 | 688.200 | 688.200 |
| R6/50 | Hold(50) lateness | 10 | 663.270 | 618.000 | 1137.800 | 1137.800 |
| R6/200 | Hold(200) lateness | 10 | 375.920 | 366.700 | 648.500 | 648.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 25.040 | 15.400 | 202.500 | 1189.600 |

### EXPERIMENT HEAD without the fast-action record, run 2

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.155 | 0.100 | 0.500 | 98.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.195 | 0.200 | 0.400 | 66.600 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.094 | 0.100 | 0.200 | 7.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 7.129 | 5.400 | 31.000 | 834.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 7.291 | 6.900 | 34.500 | 53.100 |
| R6/5 | Hold(5) lateness | 10 | 543.000 | 528.600 | 1248.600 | 1248.600 |
| R6/20 | Hold(20) lateness | 10 | 542.490 | 532.600 | 915.800 | 915.800 |
| R6/50 | Hold(50) lateness | 10 | 491.210 | 488.600 | 693.000 | 693.000 |
| R6/200 | Hold(200) lateness | 10 | 478.840 | 418.700 | 941.300 | 941.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 17.440 | 13.500 | 75.000 | 163.000 |
