## Environment — Windows

This project runs on Windows. Two shells are in play; never mix them.

- **A command the user runs in their own terminal is PowerShell.** Write
  `Get-Content`, `Select-String`, `Measure-Object -Line`, `$env:TEMP`, `;`
  chaining. Never `&&`, `/tmp/...`, `grep`, `wc`, `head`.
- **A command the agent runs through its own Bash tool is Git Bash**, and
  stays POSIX. So is anything the user types with the `!` prefix — that runs
  in the session shell, not in PowerShell.
- Default to PowerShell for anything handed to the user. Assume Bash only
  when the user has said they are in Git Bash or WSL.

Three Windows facts break silently:

- A path inside a Bash-tool command uses forward slashes — `/c/Users/...`,
  never `C:\Users\...`, where the backslash is an escape character.
- A file may carry CRLF. Anchor a pattern with `\r?$`, never a bare `$`.
- `chmod`, the exec bit and symlinks do not mean on NTFS what they mean on
  Linux. bee probes both `.bee/bin/bee` and `.bee/bin/bee.exe` for that
  reason.

This section renders because the resolved host shell is PowerShell. Set
`host_shell` in `.bee/config.json` to `powershell` or `posix` to decide it
per repository instead of per machine.
