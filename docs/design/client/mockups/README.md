# Mockups of temper's client

These mockups are reference material for temper's client (`../README.md`): its look and the order of its pages' parts, not its contract. Where they and the client's documents differ, the documents hold.

Open [chats.html](chats.html) for the chat entry screen, or [index.html](index.html) for a conversation. The other views are [inbox.html](inbox.html), [board.html](board.html), [task.html](task.html), and [changes.html](changes.html). On the goal page, agent subtasks open [agent-task.html](agent-task.html) with their individual transcripts. Each file opens directly in a browser; no build step is needed.

All pages use the compact, terminal-inspired layout: top navigation, monospaced text, quiet borders, and more room for the work. The conversation has no side panels. Its one-line composer grows with its contents and shrinks after sending or deleting text. Use Enter to send and Shift+Enter for a new line. The chat entry composer works the same way.

These are browser-only mockups with sample data. You can start a chat, send words, accept or reject the shared proposal, answer an inbox choice, release a held task, filter lists, set a goal, and steer T21. Requests show a brief pending state, then update local sample state. No request is sent to temper or the forge. Use the **More** button on the list pages to reset the demo.

The agent subtask page has examples of a completed run (T22/T23), a simulated live transcript (T26), a held review (T27), and a task waiting to start (T28). On T26, text streams into an uncommitted turn and becomes a committed turn when complete. You can write to the agent, amend its instructions, or stop and release it; these actions update local sample state.

Forge change links from the goal page point to illustrative GitHub pull request URLs; there is no separate forge subtask transcript.

For state to carry reliably between pages, serve this directory locally and open `http://localhost:8765/chats.html`:

```sh
python3 -m http.server 8765 --directory docs/design/client/mockups
```

Several sample tasks and chats have no mockup page of their own.

The only external asset is the optional Google Fonts stylesheet. System fonts are used if it cannot load.
