# Import into a user-owned catalog

The picker uses saved launch profiles because one model can have several
manually configured variants. Explicit import adds DwarfStar and llama.cpp
profiles only when their runtime and resolved model path are not already
represented, independently of profile IDs, names, or arguments. Import never
rewrites or removes profiles, even when model files disappear; deleting the
last profile makes a model eligible again. oMLX has one server entry and no
per-model import.
