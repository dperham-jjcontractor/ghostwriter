<#
One-time setup: let this PC log into the tablet without a password, so the
other scripts stop asking for it. You type the tablet's root password once.
The password is on the tablet under Settings > Help > Copyrights and licenses.

Usage (PowerShell, from the repo folder):
  .\deploy\authorize-pc.ps1                          # over the USB cable
  .\deploy\authorize-pc.ps1 -Tablet 192.168.199.110  # over Wi-Fi

Firmware updates keep /home, so this survives them.
#>
param([string] $Tablet = "10.11.99.1")

$ErrorActionPreference = "Stop"
$keyFile = "$env:USERPROFILE\.ssh\id_ed25519"
if (-not (Test-Path "$keyFile.pub")) {
    Write-Host "Creating an SSH key for this PC ..."
    ssh-keygen -t ed25519 -N "" -f $keyFile | Out-Null
}
$publicKey = (Get-Content "$keyFile.pub" -Raw).Trim()

Write-Host "Installing this PC's key on root@$Tablet. Enter the tablet password when asked."
$publicKey | ssh -o StrictHostKeyChecking=accept-new "root@$Tablet" "mkdir -p /home/root/.ssh; tr -d '\r' >> /home/root/.ssh/authorized_keys; chmod 700 /home/root/.ssh; chmod 600 /home/root/.ssh/authorized_keys"

Write-Host "Testing a password-free login ..."
ssh -o BatchMode=yes "root@$Tablet" "echo OK: this PC can now log in without a password"
