# Preserve existing oMLX settings when launching

The user configures oMLX in oMLX itself. lml does not expose oMLX setting
overrides, copy its configuration, or manage isolated settings for launch
profiles. We choose this limited integration to keep lml simple and avoid
overwriting the user's saved oMLX choices, including through model-selection
arguments that oMLX persists. oMLX therefore has one picker entry,
"oMLX — configured models," that starts its foreground server with its existing
configuration; client requests select models from that collection. DwarfStar
and llama.cpp retain per-model launch profiles, while oMLX has no per-model
import or selection in lml.
