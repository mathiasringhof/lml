# Local model launching

Language for selecting local models and starting their servers with lml.

## Language

**Local model**:
A model whose files are stored on the user's machine for use by a compatible runtime.

**Runtime**:
The software that loads a local model and serves inference requests.
_Avoid_: Backend, engine

**Server**:
A running instance of a runtime that accepts inference requests.

**Launch profile**:
A saved catalog entry with a stable ID, display name, and runtime, describing
either a single-model server launch or a runtime's configured model collection.
Several launch profiles may refer to the same local model.
_Avoid_: Model (when referring to a selectable configuration)

**Catalog**:
The collection of saved launch profiles shown in the picker.
_Avoid_: Model directory

**Import**:
An explicit scan of configured model sources that adds per-model launch
profiles for models not yet represented for that runtime in the catalog.

**Model source**:
A configured directory searched for local models during import for a runtime.
Several sources can belong to one runtime, and runtimes can share a source.
