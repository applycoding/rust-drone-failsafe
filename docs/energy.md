# Energy heuristic

Symbols: horizontal distance to home `d`, altitude `h`, RTL altitude `h_rtl`.

```
E_climb   = max(0, h_rtl − h) / v_climb · P_climb
E_cruise  = d / v_cruise · P_cruise · k_wind
E_descent = max(h, h_rtl) / v_desc · P_desc
E_rtl     = (E_climb + E_cruise + E_descent) / 3600   [Wh]
E_land    = h / v_desc · P_desc / 3600                  [Wh]
margin_wh = remaining_wh − E_rtl − reserve_wh
```

This is a flat-ground, constant-power model. Required RTL energy is **not** monotonic in altitude alone (piecewise at `h_rtl`).
