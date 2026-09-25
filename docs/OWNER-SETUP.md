# Owner setup guide

This fork turns Ghostwriter into a notes coach for one reMarkable 2. You maintain it from a Windows PC with nothing installed beyond what Windows ships (PowerShell, `ssh`, `scp`). GitHub builds the tablet program for you.

The examples use the tablet's Wi-Fi address, `192.168.199.110`. Over the USB cable the tablet is `10.11.99.1`, which is also what the scripts use when you leave out `-Tablet`.

## What you need

- The tablet's root password and Wi-Fi address: on the tablet, Settings > Help > Copyrights and licenses, at the bottom.
- An OpenAI API key from <https://platform.openai.com/api-keys>, with a monthly spending limit set on the account. A typed reply costs roughly two to three cents, counting the drawing check and the memory step; a drawing costs a few cents more.
- The PC and the tablet on the same Wi-Fi, or the USB cable.

## 1. Get the program

On GitHub: **Releases** > the newest release > `ghostwriter-rm2`. No GitHub login is needed. Save it anywhere, for example your Downloads folder.

## 2. First-time install

All commands run in PowerShell from the repo folder, `C:\Users\dperham\source\ghostwriter`.

Let this PC log in to the tablet without a password (asks for the tablet password once):

```powershell
.\deploy\authorize-pc.ps1 -Tablet 192.168.199.110
```

Install the program, its service, the prompt and the settings file:

```powershell
.\deploy\deploy.ps1 -Binary $HOME\Downloads\ghostwriter-rm2 -Tablet 192.168.199.110
```

Copy the OpenAI key to the clipboard, then store it on the tablet:

```powershell
.\deploy\set-key.ps1 -Tablet 192.168.199.110
```

Add the page template with the tap icon (restarts the tablet interface for about ten seconds):

```powershell
.\deploy\add-template.ps1 -Tablet 192.168.199.110
```

Then, on the tablet, open her notebook, open the page template menu, and pick **Lined medium + coach** under Lines. New pages keep the template of the page before them.

## 3. Updating to a newer version

Download the newest release and run `deploy.ps1` again. The key, settings, prompt and memory on the tablet are kept.

## How it behaves

- She taps the face icon at the bottom centre of the page once, quickly. The tablet adds a page, shows "Thinking...", and types the reply there, starting with `-- COACH --`. The on-screen keyboard closes by itself afterwards, and she swipes right to get back to her notes.
- A tap while the on-screen keyboard is open is ignored on purpose: the keyboard's space bar sits on top of the icon. She closes the keyboard, then taps.
- A request for a picture ("draw me a table layout with three zones") is drawn with the pen on the new page instead of typed.
- Questions get direct answers from what it knows about Talbots, her store and general retail practice, marked `(published policy)` or `(general)`.

## Checks

Capture check: what the coach sees. Run it with a notebook page open on the tablet:

```powershell
ssh root@192.168.199.110 "systemctl stop ghostwriter; cd /home/root/ghostwriter; ./ghostwriter --no-submit --no-loop --no-trigger --save-screenshot /home/root/shot.png; systemctl start ghostwriter"
scp root@192.168.199.110:/home/root/shot.png .
```

`shot.png` should look like the page. A black or scrambled image means the screen-capture offset is wrong; the log line `RM2 framebuffer: ... skip=N` shows what was used, and putting `GHOSTWRITER_FB_SKIP=<bytes>` in front of `./ghostwriter` tries another value without a rebuild.

Model check: send that saved page to the model and print the reply instead of typing it:

```powershell
ssh root@192.168.199.110 "cd /home/root/ghostwriter; ./ghostwriter --input-png /home/root/shot.png --no-trigger --no-loop --no-draw --output-file /home/root/reply.txt; cat /home/root/reply.txt"
```

`API 401` means the key is wrong; `No API key configured` means it was never stored.

Drawing check: draw an SVG file on the current page through the normal drawing path, with no model call:

```powershell
ssh root@192.168.199.110 "systemctl stop ghostwriter; cd /home/root/ghostwriter; ./ghostwriter --test-draw-svg /home/root/picture.svg; systemctl start ghostwriter"
```

Live log, to watch a real tap:

```powershell
ssh root@192.168.199.110 "journalctl -u ghostwriter -f"
```

## Tuning the prompt

The coach's instructions are two text files:

- `prompts/coach.txt`: the coaching rules (in git).
- `prompts/store-context.txt`: background about Talbots, her store and her job (kept out of git, because this repository is public).

Edit either in Notepad, then:

```powershell
.\tools\make-prompt-json.ps1
scp prompts\coach.local.json root@192.168.199.110:/home/root/ghostwriter/prompts/coach.json
```

The next tap uses the new text; no rebuild, no restart. `deploy.ps1` sends the same personalised file whenever it runs.

## What it learns over time

After each reply the coach rewrites a short memory file, `/home/root/ghostwriter/memory.txt`, with durable facts from her pages: her duties, the first names of her manager and coworkers as she writes them, store terms, how she likes replies, projects she keeps coming back to, and anything she wrote "remember ..." about. Every later page gets that memory in its prompt. The file is replaced each time and capped at 1,500 characters, so it consolidates rather than grows; every version is also kept in `memory-log.txt`.

```powershell
.\deploy\show-memory.ps1 -Tablet 192.168.199.110          # read it
.\deploy\show-memory.ps1 -Tablet 192.168.199.110 -Reset   # forget everything
```

Glance at it after her first week. If it picked up something wrong, edit the file (the script's help shows how) or reset it. Customer details are excluded by rule.

## Settings

The settings file is `/home/root/.ghostwriter.toml` on the tablet. After changing it, run `ssh root@192.168.199.110 "systemctl restart ghostwriter"`. The lines worth knowing:

| Setting | What it does |
|---|---|
| `model` | `gpt-6-sol` (default) or the cheaper `gpt-6-luna` |
| `trigger_corner` | Where the tap goes: `BC` bottom centre (default). The corners and the top centre collide with the tablet's own buttons. |
| `reply_on_new_page` | `true`: replies go on a new page, never over her notes |
| `no_svg` | `false`: pictures allowed; `true`: typed replies only |
| `svg_renderer` | `strokes` (default) traces each shape as one pen stroke; `pressure` fills row by row |
| `memory_enabled` | `true`: the coach keeps and uses its memory about her |

## If a tablet software update is ever installed

Automatic updates are off. If an update does get installed, it removes the service and the page template and gives the tablet a new ssh identity. Nothing in `/home/root` is lost. Then:

1. `ssh-keygen -R 192.168.199.110` (and `ssh-keygen -R 10.11.99.1`), so Windows accepts the tablet's new identity.
2. Run `deploy.ps1` and `add-template.ps1` again.
3. Run the capture check.

## Troubleshooting

| Symptom | Look at |
|---|---|
| Nothing happens on tap | Is the on-screen keyboard open? Close it and tap again. Otherwise `ssh root@192.168.199.110 "systemctl status ghostwriter"`, and `systemctl start ghostwriter` if it stopped. |
| "The assistant is not set up yet" | The key is missing or expired: run `set-key.ps1` again. |
| "Something went wrong" every time | `ssh root@192.168.199.110 "journalctl -u ghostwriter -n 50"`; the API error text is in there. |
| Letters typed where a drawing should be | The keyboard was not closed before drawing; the log says why. If the keyboard layout changed, fix it in Settings > Language and keyboard. |
| Replies describe a different page | Run the capture check. |

## Privacy notes

- Every tap sends the whole page image to OpenAI. Keep customer details off coached pages.
- Keep `web_server = false`: the settings web page has no password.
- Keep `log_level = "info"`: debug logs include request details.
