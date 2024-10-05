# TODO

1. Consider creating a session if it's bookmarked and not active.  
   This could be a setting in the session configuration yaml  
   (e.g. `auto_create = true`).

2. Add a setting in the session configuration yaml to specify the startup program.
   Things to consider: is it possible to start shell and then nvim?

3. Add recent-session set option and plug it into the scripts.
4. Add support for pane creation.
5. Capture panes when saving sessions.
6. Load panes when loading sessions.
7. Replace "select session" script with code.
8. Merge current active sessions with stored sessions when selecting a session.
9. Create a session when selecting a session that is inactive.
10. Fix window order when saving/restoring (add windo index to the toml).
11. Add 'session save' feature.
12. `tm` script should call 'session save'.
13. Refactor for testability/safety (decouple side effects from logic).

