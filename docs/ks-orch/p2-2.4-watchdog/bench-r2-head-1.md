
### head-1

| Row | What | n | mean us | p50 us | p99 us | max us |
|---|---|---|---|---|---|---|
| R1 | Engine::on_key, typing script | 50000 | 0.097 | 0.100 | 0.200 | 52.200 |
| R2 | Engine::on_key, Tap/Hold and Hold presses | 10000 | 0.159 | 0.200 | 0.300 | 11.000 |
| R3 | Engine::on_timer, armed Holds | 10000 | 0.081 | 0.100 | 0.100 | 0.500 |
| R4 | keyboard_hook_callback, F13 DoNothing + A unbound | 10000 | 5.096 | 4.800 | 6.700 | 253.100 |
| R5 | keyboard_hook_callback, F13 Hold(50) key-down | 200 | 5.043 | 4.800 | 6.200 | 32.800 |
| R6/5 | Hold(5) lateness | 10 | 173.040 | 131.100 | 449.800 | 449.800 |
| R6/20 | Hold(20) lateness | 10 | 325.860 | 358.600 | 516.700 | 516.700 |
| R6/50 | Hold(50) lateness | 10 | 386.230 | 342.000 | 675.800 | 675.800 |
| R6/200 | Hold(200) lateness | 10 | 346.420 | 358.700 | 516.600 | 516.600 |
| R7 | SendMessageTimeoutW(WM_NULL) round trip to the loop | 1000 | 10.908 | 10.500 | 17.100 | 175.000 |
| R9 | keyboard_hook_callback, F13 key-down, when: AppEquals (no match) | 2000 | 4.790 | 4.700 | 4.800 | 99.500 |
| R10 | keyboard_hook_callback, F13 key-down, when: TitleContains (no match) | 2000 | 5.078 | 5.000 | 5.300 | 46.800 |
| R11 | keyboard_hook_callback, key-up of a swallowed F13 | 2000 | 5.288 | 5.300 | 5.400 | 7.100 |
| R12 | keyboard_hook_callback, unbound A down and up | 2000 | 4.699 | 4.700 | 4.800 | 6.400 |
| R13 | SendMessageTimeoutW(WM_NULL) round trip to the loop with the control window | 1000 | 11.105 | 10.600 | 20.600 | 31.100 |
| R14 | keyboard_hook_callback, unbound A down and up every 1 ms, hook watchdog ticking | 2000 | 5.335 | 4.800 | 8.600 | 209.000 |
| R14c | R14 with no watchdog: same pacing, same post to a queue nobody reads | 2000 | 5.405 | 4.800 | 9.100 | 199.600 |
| R15 | post + peek + dispatch of one message whose procedure reads raw input (rejected design) | 2000 | 3.922 | 3.800 | 4.800 | 15.900 |
| R16 | keyboard_hook_callback, A key-down that wakes the idle hook watchdog | 1000 | 52.744 | 54.600 | 107.600 | 355.000 |
