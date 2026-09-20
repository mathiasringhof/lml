# Import into a user-owned catalog

The picker uses saved launch profiles rather than discovering models whenever
it opens, because one model can have several deliberately configured variants.
For DwarfStar and llama.cpp, explicit, repeatable import adds a profile only when
no existing profile refers to that runtime and resolved model path; profile
names, IDs, and arguments do not determine whether a model is already represented.
Import never rewrites or
removes profiles, preserving manual edits even when model files disappear;
deleting the last profile makes that model eligible for import again. oMLX uses
one server-launch entry rather than importing individual models.
