# Hand the terminal to the selected server

lml's interactive role ends when the user selects a launch profile: it restores
the ordinary terminal and starts the runtime server in the foreground with
native terminal input and output. We choose this boundary over an in-TUI log
viewer or persistent server manager because the intended workflow is the same
as launching a server directly from the shell; Ctrl+C stops the server and
returns the user to the shell, rather than to the picker. Future launch by
profile ID should use the same terminal behavior without opening the picker.
