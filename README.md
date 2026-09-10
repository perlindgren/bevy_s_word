# AI Workflow Notes

## Arch linux

To start ollama service:

- `ollama serve`, to start the ollama server.
- if using systemd service 
  - `journalctl -u ollama --no-pager --follow --pager-end`
- if on mac
  - `tail -f ~/.ollama/logs/server.log`
