<#
Install or update Ghostwriter on the reMarkable from this Windows PC.

Usage (PowerShell, from the repo folder):
  .\deploy\deploy.ps1 -Binary C:\path\to\ghostwriter-rm2 -Tablet 192.168.199.110
  .\deploy\deploy.ps1 -Binary ... -SetKey      # also store the OpenAI key (copy it to the clipboard first)
Without -Tablet it uses the USB cable address, 10.11.99.1.

Where the binary comes from: GitHub > Releases > the newest release > ghostwriter-rm2
(no GitHub login needed). Every push also leaves one under Actions > the latest green
"Build rm2 binary" run > Artifacts, but that needs a login and downloads as a zip.

Run .\deploy\authorize-pc.ps1 once first; after that nothing asks for the tablet password.
Re-run this after every tablet software update: updates remove the service.
#>
param(
    [Parameter(Mandatory = $true)] [string] $Binary,
    [string] $Tablet = "10.11.99.1",
    [switch] $SetKey
)

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$prompts = Join-Path $here "..\prompts"
$target = "root@$Tablet"

if (-not (Test-Path $Binary)) { throw "Binary not found: $Binary" }
# coach.local.json (built by tools\make-prompt-json.ps1 from coach.txt + store-context.txt) is
# the personalised prompt and stays out of git; fall back to the generic coach.json.
$coach = Join-Path $prompts "coach.local.json"
if (-not (Test-Path $coach)) { $coach = Join-Path $prompts "coach.json" }
Write-Host "Prompt: $coach"

Write-Host "Copying files to $target ..."
ssh $target "mkdir -p /home/root/ghostwriter/prompts"
scp $Binary "${target}:/home/root/ghostwriter/ghostwriter-rm2"
scp (Join-Path $here "ghostwriter.service") (Join-Path $here "install.sh") (Join-Path $here "fix-clock.sh") (Join-Path $here "ghostwriter.toml.example") "${target}:/home/root/ghostwriter/"
scp $coach "${target}:/home/root/ghostwriter/prompts/coach.json"
# Files in the tablet's prompts folder override the copies built into the binary, so
# refresh them on every install; a stale tool description would otherwise win.
scp (Join-Path $prompts "tool_draw_text.json") (Join-Path $prompts "tool_draw_svg.json") (Join-Path $prompts "memory.json") "${target}:/home/root/ghostwriter/prompts/"

Write-Host "Installing ..."
ssh $target "sh /home/root/ghostwriter/install.sh"

if ($SetKey) {
    & (Join-Path $here "set-key.ps1") -Tablet $Tablet
}
