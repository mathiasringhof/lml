# Hand the terminal to the selected server

After selection, lml restores the ordinary terminal and starts the server in
the foreground with native input and output. This preserves the familiar
shell workflow without an in-TUI log viewer or persistent server manager:
Ctrl-C stops the server and returns to the shell. Direct launch by profile ID
uses the same behavior without opening the picker.
