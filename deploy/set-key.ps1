<#
Store the OpenAI API key on the tablet and restart the coach.
You paste the key when asked; it is not echoed and never appears on a command line.

Usage (PowerShell, from the repo folder):
  .\deploy\set-key.ps1                          # over the USB cable
  .\deploy\set-key.ps1 -Tablet 192.168.199.110  # over Wi-Fi

Run .\deploy\authorize-pc.ps1 first so this does not ask for the tablet password.
#>
param([string] $Tablet = "10.11.99.1")

$ErrorActionPreference = "Stop"
$secure = Read-Host -AsSecureString "Paste the OpenAI API key"
$plain = [System.Net.NetworkCredential]::new("", $secure).Password.Trim()
if ($plain.Length -eq 0) { throw "No key entered." }

"OPENAI_API_KEY=$plain" | ssh "root@$Tablet" "umask 077; tr -d '\r' > /home/root/ghostwriter/.env && systemctl restart ghostwriter; sleep 3; systemctl is-active ghostwriter"
Write-Host "Key saved on the tablet and the coach restarted."
