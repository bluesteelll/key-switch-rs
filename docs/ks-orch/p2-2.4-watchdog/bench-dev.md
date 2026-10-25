
### feat/hook-watchdog (developer run 1)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.140 | 0.100 | 0.500 | 98.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.217 | 0.200 | 0.600 | 23.200 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.106 | 0.100 | 0.200 | 2.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 68.962 | 63.900 | 184.500 | 3217.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 60.684 | 58.400 | 106.400 | 173.400 |
| R6/5 | Hold(5) lateness | 10 | 470.950 | 473.900 | 558.100 | 558.100 |
| R6/20 | Hold(20) lateness | 10 | 435.490 | 425.500 | 779.000 | 779.000 |
| R6/50 | Hold(50) lateness | 10 | 437.430 | 454.800 | 744.600 | 744.600 |
| R6/200 | Hold(200) lateness | 10 | 482.460 | 396.500 | 960.400 | 960.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 47.390 | 44.900 | 102.200 | 201.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 57.865 | 57.400 | 90.200 | 273.800 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 59.488 | 58.600 | 97.900 | 273.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 64.868 | 64.700 | 104.800 | 228.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 58.083 | 57.600 | 112.200 | 300.900 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 48.201 | 45.200 | 111.800 | 217.900 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 63.873 | 60.400 | 115.400 | 300.300 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 25.204 | 23.900 | 55.100 | 136.500 |

### master fa20b30 (developer run 1)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.100 | 0.100 | 0.200 | 104.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.159 | 0.200 | 0.300 | 5.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.085 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 61.642 | 58.700 | 134.500 | 564.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 57.690 | 57.600 | 108.300 | 172.000 |
| R6/5 | Hold(5) lateness | 10 | 485.190 | 462.100 | 987.700 | 987.700 |
| R6/20 | Hold(20) lateness | 10 | 410.020 | 344.900 | 582.800 | 582.800 |
| R6/50 | Hold(50) lateness | 10 | 348.930 | 323.400 | 574.000 | 574.000 |
| R6/200 | Hold(200) lateness | 10 | 446.140 | 410.900 | 638.800 | 638.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 51.021 | 46.900 | 127.600 | 350.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 61.478 | 58.900 | 169.300 | 358.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 64.400 | 61.200 | 163.600 | 702.700 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 65.563 | 64.800 | 114.100 | 286.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 58.772 | 57.800 | 99.700 | 228.600 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 47.678 | 44.900 | 98.300 | 236.700 |

### feat/hook-watchdog (developer run 2)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.089 | 0.100 | 0.200 | 17.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.152 | 0.200 | 0.300 | 3.300 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.075 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 58.062 | 57.500 | 106.600 | 294.600 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 55.863 | 55.500 | 79.400 | 93.500 |
| R6/5 | Hold(5) lateness | 10 | 433.970 | 461.700 | 529.500 | 529.500 |
| R6/20 | Hold(20) lateness | 10 | 429.570 | 369.000 | 820.800 | 820.800 |
| R6/50 | Hold(50) lateness | 10 | 357.140 | 272.300 | 645.000 | 645.000 |
| R6/200 | Hold(200) lateness | 10 | 434.170 | 425.700 | 636.400 | 636.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 48.828 | 45.500 | 117.900 | 232.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 58.995 | 56.800 | 137.500 | 393.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 61.910 | 59.600 | 130.600 | 383.500 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 25.843 | 5.200 | 85.200 | 298.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 57.468 | 56.900 | 103.000 | 203.300 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 46.827 | 44.400 | 92.400 | 115.900 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 61.815 | 58.800 | 107.400 | 317.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 24.711 | 23.600 | 48.500 | 144.300 |

### feat/hook-watchdog (developer run 3)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.090 | 0.100 | 0.200 | 16.700 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.155 | 0.200 | 0.300 | 3.800 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.078 | 0.100 | 0.100 | 4.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 55.818 | 57.400 | 105.000 | 2258.300 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 7.537 | 4.800 | 72.100 | 130.300 |
| R6/5 | Hold(5) lateness | 10 | 450.370 | 463.200 | 630.000 | 630.000 |
| R6/20 | Hold(20) lateness | 10 | 435.680 | 429.300 | 618.700 | 618.700 |
| R6/50 | Hold(50) lateness | 10 | 463.540 | 363.300 | 1125.900 | 1125.900 |
| R6/200 | Hold(200) lateness | 10 | 350.840 | 273.600 | 610.400 | 610.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 47.374 | 45.500 | 95.200 | 145.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 58.989 | 57.400 | 119.000 | 280.100 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 60.945 | 59.500 | 113.000 | 249.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 64.846 | 63.900 | 123.200 | 302.300 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 57.894 | 57.200 | 95.200 | 287.800 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 48.208 | 45.800 | 97.100 | 307.400 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 62.931 | 59.400 | 111.300 | 340.300 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 27.189 | 23.900 | 62.800 | 223.700 |

### feat/hook-watchdog (developer run 4)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.093 | 0.100 | 0.200 | 37.100 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.155 | 0.200 | 0.300 | 11.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.077 | 0.100 | 0.100 | 0.400 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.353 | 4.800 | 8.700 | 458.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.178 | 4.900 | 17.100 | 24.800 |
| R6/5 | Hold(5) lateness | 10 | 289.970 | 211.300 | 546.000 | 546.000 |
| R6/20 | Hold(20) lateness | 10 | 363.070 | 348.500 | 593.500 | 593.500 |
| R6/50 | Hold(50) lateness | 10 | 420.670 | 396.900 | 915.600 | 915.600 |
| R6/200 | Hold(200) lateness | 10 | 283.370 | 215.200 | 579.300 | 579.300 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 12.214 | 10.700 | 43.600 | 221.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.159 | 4.700 | 7.000 | 188.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 6.907 | 6.400 | 14.800 | 104.600 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.845 | 5.300 | 10.600 | 272.900 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.812 | 4.700 | 5.900 | 27.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.377 | 10.600 | 25.600 | 76.200 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 25.360 | 22.200 | 91.700 | 228.300 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.437 | 3.900 | 13.500 | 188.600 |

### feat/hook-watchdog (developer run 5)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.098 | 0.100 | 0.200 | 98.600 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.160 | 0.200 | 0.300 | 29.500 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.100 | 23.000 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.258 | 4.800 | 9.100 | 304.200 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 4.942 | 4.800 | 8.500 | 18.000 |
| R6/5 | Hold(5) lateness | 10 | 199.310 | 89.300 | 545.600 | 545.600 |
| R6/20 | Hold(20) lateness | 10 | 347.000 | 333.300 | 544.000 | 544.000 |
| R6/50 | Hold(50) lateness | 10 | 370.020 | 425.500 | 556.000 | 556.000 |
| R6/200 | Hold(200) lateness | 10 | 303.370 | 224.500 | 549.500 | 549.500 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 12.437 | 10.500 | 50.900 | 237.500 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.041 | 4.700 | 7.900 | 65.900 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 7.029 | 6.400 | 19.600 | 89.400 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.824 | 5.400 | 8.400 | 281.700 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.164 | 4.800 | 10.900 | 180.200 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 14.666 | 11.400 | 55.500 | 438.600 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 26.017 | 22.800 | 92.100 | 287.400 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 4.378 | 3.900 | 12.400 | 253.200 |

### master fa20b30 (developer run 2)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.096 | 0.100 | 0.200 | 26.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.155 | 0.200 | 0.300 | 2.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.080 | 0.100 | 0.100 | 2.300 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.376 | 4.800 | 10.400 | 443.000 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.398 | 4.900 | 16.400 | 33.900 |
| R6/5 | Hold(5) lateness | 10 | 441.620 | 477.000 | 579.100 | 579.100 |
| R6/20 | Hold(20) lateness | 10 | 407.670 | 387.500 | 876.100 | 876.100 |
| R6/50 | Hold(50) lateness | 10 | 363.040 | 364.100 | 658.700 | 658.700 |
| R6/200 | Hold(200) lateness | 10 | 324.210 | 293.000 | 560.000 | 560.000 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 11.772 | 10.400 | 27.600 | 238.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 5.437 | 4.800 | 14.900 | 200.400 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 7.433 | 6.300 | 15.400 | 281.800 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.605 | 5.300 | 6.700 | 293.400 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 5.127 | 4.800 | 11.000 | 132.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 12.569 | 10.700 | 46.900 | 182.700 |

### feat/hook-watchdog (developer run 6, with R14/settled)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.185 | 0.200 | 0.400 | 257.400 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.287 | 0.300 | 0.500 | 14.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.133 | 0.100 | 0.200 | 1.900 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 9.655 | 8.100 | 25.400 | 1342.400 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 12.388 | 8.100 | 75.900 | 676.200 |
| R6/5 | Hold(5) lateness | 10 | 235.560 | 128.100 | 516.800 | 516.800 |
| R6/20 | Hold(20) lateness | 10 | 270.740 | 232.900 | 513.700 | 513.700 |
| R6/50 | Hold(50) lateness | 10 | 331.650 | 254.900 | 514.700 | 514.700 |
| R6/200 | Hold(200) lateness | 10 | 467.960 | 478.200 | 626.800 | 626.800 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.369 | 16.700 | 49.100 | 204.400 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 8.905 | 8.000 | 26.100 | 250.400 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 12.350 | 11.200 | 22.100 | 810.000 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 10.751 | 9.000 | 26.400 | 1013.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 9.599 | 8.000 | 23.700 | 670.500 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 17.733 | 16.700 | 41.800 | 118.600 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 47.256 | 27.600 | 696.600 | 1670.600 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 16.152 | 8.500 | 151.100 | 1151.200 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 7.877 | 6.300 | 29.400 | 1154.000 |

### feat/hook-watchdog (developer run 7, with R14/settled)

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.167 | 0.100 | 0.400 | 45.800 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.286 | 0.300 | 0.500 | 13.400 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.140 | 0.100 | 0.200 | 6.000 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 10.253 | 8.200 | 44.000 | 1514.700 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 8.643 | 8.300 | 10.600 | 71.600 |
| R6/5 | Hold(5) lateness | 10 | 313.230 | 330.000 | 488.200 | 488.200 |
| R6/20 | Hold(20) lateness | 10 | 400.590 | 448.800 | 589.400 | 589.400 |
| R6/50 | Hold(50) lateness | 10 | 325.600 | 235.900 | 586.100 | 586.100 |
| R6/200 | Hold(200) lateness | 10 | 302.690 | 233.900 | 563.400 | 563.400 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 18.830 | 17.300 | 55.900 | 147.800 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 8.457 | 8.000 | 18.100 | 217.200 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 11.522 | 11.200 | 20.800 | 85.300 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 10.806 | 9.000 | 24.100 | 904.800 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 10.359 | 8.000 | 50.700 | 1117.700 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 17.861 | 16.900 | 40.500 | 172.700 |
| R14 | keyboard_hook_callback, unbound A down and up, hook watchdog running | 2000 | 42.645 | 29.000 | 360.900 | 2190.700 |
| R14/settled | R14 with the watchdog done with each raw event before the call | 2000 | 13.847 | 8.500 | 141.700 | 1399.900 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 6.743 | 6.400 | 16.700 | 129.000 |
