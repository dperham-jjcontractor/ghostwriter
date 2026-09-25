#!/bin/sh
# Install or update Ghostwriter on the reMarkable. Runs ON the tablet as root:
#   sh /home/root/ghostwriter/install.sh
# Safe to re-run. Re-run it after every firmware update: updates wipe /etc,
# which is where the service file has to live.
set -eu

DIR=/home/root/ghostwriter
UNIT=/etc/systemd/system/ghostwriter.service

cd "$DIR"

# Files copied from Windows may carry CR line endings; strip them.
for f in install.sh ghostwriter.service ghostwriter.toml.example .env; do
  [ -f "$f" ] && sed -i 's/\r$//' "$f"
done

# The binary arrives as ghostwriter-rm2 (the GitHub artifact name) or ghostwriter.
if [ -f ghostwriter-rm2 ]; then
  mv -f ghostwriter-rm2 ghostwriter
fi
[ -f ghostwriter ] || { echo "No binary found in $DIR (expected ghostwriter or ghostwriter-rm2)"; exit 1; }
chmod 755 ghostwriter

# Settings file: created from the example on first install, never overwritten.
if [ ! -f /home/root/.ghostwriter.toml ] && [ -f ghostwriter.toml.example ]; then
  cp ghostwriter.toml.example /home/root/.ghostwriter.toml
fi
chmod 600 /home/root/.ghostwriter.toml 2>/dev/null || true

# API key file: an empty template on first install.
if [ ! -f .env ]; then
  printf 'OPENAI_API_KEY=\n' > .env
fi
chmod 600 .env

mkdir -p prompts

# Service unit: always reinstalled because firmware updates remove it.
cp -f ghostwriter.service "$UNIT"
systemctl daemon-reload
systemctl enable ghostwriter >/dev/null 2>&1 || true
systemctl restart ghostwriter

sleep 3
if systemctl is-active --quiet ghostwriter; then
  echo "Ghostwriter is running."
else
  echo "Ghostwriter did not start. Last log lines:"
  journalctl -u ghostwriter -n 30 --no-pager || true
  exit 1
fi

if ! grep -q '^OPENAI_API_KEY=..*' .env; then
  echo "NOTE: no API key yet. Put it in $DIR/.env as OPENAI_API_KEY=sk-... then run: systemctl restart ghostwriter"
fi
