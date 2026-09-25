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
# A dedicated key with no passphrase, used only for the tablet, so scripts never
# have to prompt for anything. Your normal ssh key (if any) is left alone.
$keyFile = "$env:USERPROFILE\.ssh\ghostwriter_tablet_ed25519"
if (-not (Test-Path "$keyFile.pub")) {
    Write-Host "Creating a tablet-only SSH key for this PC ..."
    & ssh-keygen -q -t ed25519 -N '' -C "ghostwriter-tablet-from-$env:COMPUTERNAME" -f $keyFile
}
$publicKey = (Get-Content "$keyFile.pub" -Raw).Trim()

# Tell ssh to use that key for the tablet's addresses (USB and Wi-Fi) and for the short name "tablet".
$sshDir = "$env:USERPROFILE\.ssh"
$configFile = Join-Path $sshDir "config"
$block = @"

# Ghostwriter tablet (added by deploy\authorize-pc.ps1)
Host tablet
    HostName $Tablet
Host tablet 10.11.99.1 192.168.199.110 $Tablet
    User root
    IdentityFile ~/.ssh/ghostwriter_tablet_ed25519
    IdentitiesOnly yes
    StrictHostKeyChecking accept-new
"@
if (-not (Test-Path $configFile) -or -not (Select-String -Path $configFile -Pattern 'ghostwriter_tablet_ed25519' -Quiet)) {
    Add-Content -Path $configFile -Value $block
    Write-Host "Added a tablet entry to $configFile"
}

Write-Host "Installing this PC's key on root@$Tablet. Enter the tablet password when asked."
$publicKey | ssh -o PreferredAuthentications=password -o PubkeyAuthentication=no "root@$Tablet" "mkdir -p /home/root/.ssh; tr -d '\r' >> /home/root/.ssh/authorized_keys; chmod 700 /home/root/.ssh; chmod 600 /home/root/.ssh/authorized_keys"

Write-Host "Testing a password-free login ..."
ssh -o BatchMode=yes "root@$Tablet" "echo OK: this PC can now log in without a password"
