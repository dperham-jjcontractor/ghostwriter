<#
Store the OpenAI API key on the tablet and restart the coach.

How to use:
  1. Copy the key (it starts with sk-) so it is on the clipboard.
  2. Run:  .\deploy\set-key.ps1 -Tablet 192.168.199.110
The script takes the key from the clipboard, checks that it looks like a key,
sends it to the tablet, and then clears the clipboard. If the clipboard does
not hold a key it asks you to paste it into a visible prompt instead.

Run .\deploy\authorize-pc.ps1 first so this does not ask for the tablet password.
#>
param([string] $Tablet = "10.11.99.1")

$ErrorActionPreference = "Stop"

function Test-LooksLikeKey([string] $value) {
    return ($value -match '^sk-[A-Za-z0-9_-]{30,}$')
}

$key = ""
try { $key = (Get-Clipboard -Raw -ErrorAction Stop).Trim() } catch { $key = "" }

if (-not (Test-LooksLikeKey $key)) {
    Write-Host "The clipboard does not hold an OpenAI key. Paste it here (it will be visible on this screen only):"
    $key = (Read-Host).Trim()
}
if (-not (Test-LooksLikeKey $key)) {
    throw "That does not look like an OpenAI key (expected sk-... with letters, digits, - and _). Nothing was changed."
}

# Sent over stdin so the key never appears on a command line; CRs are stripped on the tablet.
"OPENAI_API_KEY=$key" | ssh "root@$Tablet" "umask 077; tr -d '\r' > /home/root/ghostwriter/.env && systemctl restart ghostwriter; sleep 3; systemctl is-active ghostwriter"

try { Set-Clipboard -Value " " } catch { }
Write-Host "Key saved on the tablet ($($key.Length) characters) and the coach restarted. Clipboard cleared."
