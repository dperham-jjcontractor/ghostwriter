<#
Install the "Lined medium + coach" page template on the tablet: the normal
lined page with a small brain icon in the top-right corner marking the tap spot.

Usage (PowerShell, from the repo folder):
  .\deploy\add-template.ps1 -Tablet 192.168.199.110

What it does: backs up the tablet's template catalogue, adds one entry to it,
copies the template image files, and restarts the tablet's interface (about
ten seconds; the tablet is usable again right after). Safe to re-run.
Tablet software updates replace these files, so re-run this after an update.

Afterwards, on the tablet: open the notebook, open the toolbar, choose the page
template, and pick "Lined medium + coach" under Lines. New pages keep the
template of the page before them.
#>
param([string] $Tablet = "10.11.99.1")

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$src = Join-Path $here "templates"
$target = "root@$Tablet"
$remoteDir = "/usr/share/remarkable/templates"

# 1. Fetch the tablet's current catalogue and add the entry (done here; the tablet has no Python).
$work = Join-Path $env:TEMP "ghostwriter-template"
New-Item -ItemType Directory -Force $work | Out-Null
scp "${target}:$remoteDir/templates.json" (Join-Path $work "templates.json")
$py = @'
import json, sys, copy
path = sys.argv[1]
data = json.load(open(path, encoding="utf-8"))
templates = data["templates"]
if not any(t.get("filename") == "P Lines medium coach" for t in templates):
    i = next(i for i, t in enumerate(templates) if t.get("filename") == "P Lines medium")
    entry = copy.deepcopy(templates[i])
    entry["name"] = "Lined medium + coach"
    entry["filename"] = "P Lines medium coach"
    templates.insert(i + 1, entry)
    json.dump(data, open(path, "w", encoding="utf-8", newline="\n"), indent=2)
    print("entry added")
else:
    print("entry already present")
'@
$pyFile = Join-Path $work "add-entry.py"
[System.IO.File]::WriteAllText($pyFile, $py)
python $pyFile (Join-Path $work "templates.json")

# 2. Stage files under simple names (scp and spaces do not mix), then move them into place.
scp (Join-Path $src "P Lines medium coach.png") "${target}:/home/root/stage_coach.png"
scp (Join-Path $src "P Lines medium coach.svg") "${target}:/home/root/stage_coach.svg"
scp (Join-Path $work "templates.json") "${target}:/home/root/stage_templates.json"
ssh $target "cd $remoteDir && ([ -f templates.json.bak-coach ] || cp templates.json templates.json.bak-coach) && cp /home/root/stage_coach.png 'P Lines medium coach.png' && cp /home/root/stage_coach.svg 'P Lines medium coach.svg' && cp /home/root/stage_templates.json templates.json && chmod 644 'P Lines medium coach.png' 'P Lines medium coach.svg' templates.json && systemctl restart xochitl && sleep 12 && pidof xochitl >/dev/null && echo 'Template installed; the tablet interface has restarted.'"
