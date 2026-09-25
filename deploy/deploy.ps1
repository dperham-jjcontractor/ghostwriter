<#
Install or update Ghostwriter on the reMarkable from this Windows PC.

Usage (PowerShell, from the repo folder):
  .\deploy\deploy.ps1 -Binary C:\Users\you\Downloads\ghostwriter-rm2\ghostwriter-rm2
  .\deploy\deploy.ps1 -Binary ... -Tablet 192.168.1.50   # over Wi-Fi instead of the USB cable
  .\deploy\deploy.ps1 -Binary ... -SetKey                 # also store the OpenAI API key on the tablet

Where the binary comes from: GitHub > Actions > the latest green "Build rm2 binary" run
> Artifacts > ghostwriter-rm2. That is a zip; unzip it first and point -Binary at the file inside.

The tablet asks for its root password on each command. It is shown on the tablet under
Settings > Help > Copyrights and licenses (scroll to the bottom). Over the USB cable the
tablet is 10.11.99.1; over Wi-Fi its address is shown on the same screen.

Re-run this after every tablet software update: updates remove the service.
#>
param(
    [Parameter(Mandatory = $true)] [string] $Binary,
    [string] $Tablet = "10.11.99.1",
    [switch] $SetKey
)

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$target = "root@$Tablet"

if (-not (Test-Path $Binary)) { throw "Binary not found: $Binary" }
$coach = Join-Path $here "..\prompts\coach.json"

Write-Host "Copying files to $target ..."
ssh $target "mkdir -p /home/root/ghostwriter/prompts"
scp $Binary "${target}:/home/root/ghostwriter/ghostwriter-rm2"
scp (Join-Path $here "ghostwriter.service") (Join-Path $here "install.sh") (Join-Path $here "ghostwriter.toml.example") "${target}:/home/root/ghostwriter/"
scp $coach "${target}:/home/root/ghostwriter/prompts/coach.json"

if ($SetKey) {
    $secure = Read-Host -AsSecureString "Paste the OpenAI API key (it is not shown)"
    $plain = [System.Net.NetworkCredential]::new("", $secure).Password.Trim()
    if ($plain.Length -gt 0) {
        # Sent over stdin so the key never appears on a command line; CRs are stripped on the tablet.
        "OPENAI_API_KEY=$plain" | ssh $target "umask 077; tr -d '\r' > /home/root/ghostwriter/.env"
        Write-Host "API key saved on the tablet."
    }
}

Write-Host "Installing ..."
ssh $target "sh /home/root/ghostwriter/install.sh"
