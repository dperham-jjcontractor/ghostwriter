# Owner setup guide

This fork turns Ghostwriter into a notes coach for one reMarkable 2. You maintain it from a Windows PC; nothing needs to be installed on the PC except what Windows already ships (PowerShell, `ssh`, `scp`). GitHub builds the tablet binary for you.

## What you need

- The tablet's root password: on the tablet, Settings > Help > Copyrights and licenses, scroll to the bottom. The same screen shows the tablet's Wi-Fi address.
- An OpenAI API key from <https://platform.openai.com/api-keys>. Set a monthly spending limit there while you are at it; a tap costs about a cent on `gpt-6-sol`.
- A USB cable (the tablet is `10.11.99.1` over USB) or the tablet's Wi-Fi address. If `ssh root@10.11.99.1` does not answer over USB, Windows may be missing the "Remote NDIS" driver; use the Wi-Fi address instead and come back to USB later.

## 1. Get a binary

Every push to the `coach-v1` branch builds one. On GitHub: **Actions** > the latest green **Build rm2 binary** run > **Artifacts** > `ghostwriter-rm2`. It downloads as a zip; unzip it. The file inside, also called `ghostwriter-rm2`, is the program.

## 2. Install on the tablet

From the repo folder in PowerShell:

```powershell
.\deploy\deploy.ps1 -Binary C:\path\to\ghostwriter-rm2 -SetKey
```

Add `-Tablet 192.168.1.50` (the tablet's Wi-Fi address) if you are not on the USB cable. It asks for the tablet password a few times, then for the API key once, and ends with `Ghostwriter is running.`

What it installs, all under `/home/root/ghostwriter` on the tablet: the binary, the service, the prompt, `.env` with the key, and `/home/root/.ghostwriter.toml` with the settings. It also copies the service into `/etc/systemd/system`, which is the one thing a firmware update wipes.

## 3. Check that capture works

Do this once now, and again after any firmware update. On the PC:

```powershell
ssh root@10.11.99.1 "systemctl stop ghostwriter; cd /home/root/ghostwriter; ./ghostwriter --no-submit --no-loop --save-screenshot /home/root/shot.png"
```

Open a notebook page on the tablet with some writing on it and tap the top-right corner. The command exits. Then:

```powershell
scp root@10.11.99.1:/home/root/shot.png .
ssh root@10.11.99.1 "systemctl start ghostwriter"
```

Open `shot.png`. It should look like the page. A black or scrambled image means the framebuffer offset is wrong for this firmware; the log line `RM2 framebuffer: firmware X.Y ... skip=N` tells you what was used, and `GHOSTWRITER_FB_SKIP=<bytes>` in front of the command tries another value without a rebuild.

## 4. Check the model without the tablet screen

```powershell
ssh root@10.11.99.1 "cd /home/root/ghostwriter; set -a; . ./.env; set +a; ./ghostwriter --input-png /home/root/shot.png --no-trigger --no-loop --no-draw --output-file /home/root/reply.txt; cat /home/root/reply.txt"
```

This sends the saved page to the model and prints the reply instead of typing it. A `401` in the output means the key is wrong; `No API key configured` means `.env` is empty.

## 5. The real thing

Open a notebook, write three lines such as "Fitting room: max 6 items, ask for size first", hide the toolbar, tap the top-right corner once. Watch the log from the PC if you like:

```powershell
ssh root@10.11.99.1 "journalctl -u ghostwriter -f"
```

## Firmware upgrade

The tablet is on an old firmware. Upgrading is recommended so the capture path is the one other people run. Order matters:

1. Run the capture check (step 3) on the current firmware and keep `shot.png`.
2. On the tablet: Settings > General > Software, install the update, reboot.
3. Settings > General > Software: turn **automatic updates off**.
4. The update wiped the service and changed the tablet's ssh key. On the PC run `ssh-keygen -R 10.11.99.1` (and the same for the Wi-Fi address), then run `deploy.ps1` again.
5. Run the capture check again on the new firmware.
6. Newer firmware puts the tablet's own close button in the top-right corner. If tapping the corner closes the notebook, change `trigger_corner` in `/home/root/.ghostwriter.toml` to `LL` (lower-left, out of the way of a right hand), restart with `ssh root@10.11.99.1 "systemctl restart ghostwriter"`, and update the card.

## Tuning the prompt

The coach's instructions are two text files:

- `prompts/coach.txt`: the generic coaching rules (in git).
- `prompts/store-context.txt`: background about her store and her job (kept out of git, because this repository is public).

Edit either in Notepad, then:

```powershell
.\tools\make-prompt-json.ps1
scp prompts\coach.local.json root@10.11.99.1:/home/root/ghostwriter/prompts/coach.json
```

The next tap uses the new text; no rebuild, no restart. `deploy.ps1` sends the same personalised file whenever it runs. Keep replies under about 700 characters: the keyboard types roughly 100 characters per second and the reply stays on the page.

To try the cheaper model, set `model = "gpt-6-luna"` in `/home/root/.ghostwriter.toml` and restart the service.

## After any tablet software update

Run `deploy.ps1` again (the service is gone), then the capture check. Nothing else is lost: the key, settings and prompt live in `/home/root`.

## Troubleshooting

| Symptom | Look at |
|---|---|
| Nothing happens on tap | `ssh root@10.11.99.1 "systemctl status ghostwriter"`; if stopped, `systemctl start ghostwriter` |
| "not set up yet" on the page | `.env` on the tablet has no key; run `deploy.ps1 -SetKey` again |
| "Something went wrong" every time | `journalctl -u ghostwriter -n 50`; the API error text is in there |
| Replies read a different page | capture check (step 3) |
| Two taps needed to start | should be fixed; if it returns, report it with the debug log (`log_level = "debug"` in the TOML) |

## Privacy notes

- Every tap sends the whole page image to OpenAI. Keep customer details off coached pages.
- Keep `web_server = false`: the settings page has no password.
- Keep `log_level = "info"`: debug logs include request details.
