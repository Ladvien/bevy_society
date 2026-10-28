# Research

No open bug or enhancement issue (only release issues #24, #27, seen and skipped). Item picked from the log's top-5: a person re-chooses the same unreachable place every stall window (follow-up to #26).

Not a social-psychology model. The mechanism is the failure memory every navigating agent needs: do not re-issue a goal that just failed (the stuck-watchdog of #26 ends the walk; this keeps the *decision* from reissuing it). No home-still corpus paper fixes a retry interval, so `failed_place_memory_hours` is declared a **design default** (`None`) in `Tuning`. Papers were not searched further: the quantity is a host's judgement of how long a closed route stays closed.

Mechanism: `Place` targets all tie (no `Distance` term) and the nearest wins, so the same place is picked deterministically after every failure. Remove the failed place from the candidate list for `h` game-hours.
