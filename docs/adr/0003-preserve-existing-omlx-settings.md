# Preserve existing oMLX settings when launching

oMLX has one picker entry, "oMLX — configured models," that starts `omlx serve`
with its existing configuration. lml supplies no setting or model-selection
overrides and makes no configuration copies, keeping the integration simple
and avoiding overrides that oMLX would persist. Users configure oMLX itself;
client requests select models from its configured collection. DwarfStar and
llama.cpp retain per-model launch profiles.
