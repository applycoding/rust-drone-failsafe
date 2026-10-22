# Supervisor pipeline

Each 50 ms tick:

1. Validate inputs and clock
2. Glitch-filter position; compute `position_valid`
3. Debounced triggers → active requests
4. Max action rank + display reason tie order
5. Persistence / recovery / latches
6. Feasibility (Hold/RTL + invalid position → Land)
7. Recompute `disarm_eligible` (clears immediately when not on ground)

There is no `Disarm` action; disarm is advisory only.
