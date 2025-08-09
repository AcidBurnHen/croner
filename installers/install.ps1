param([string]$Version = "latest")

$repo = "AcidBurnHen/croner"
if ($Version -eq "latest") {
    $Version = (Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest").tag_name
}

$asset = "croner-$Version-x86_64-pc-windows-msvc.zip"
$base  = "https://github.com/$repo/releases/download/$Version"
$tmp   = New-Item -ItemType Directory -Path ([System.IO.Path]::Combine([IO.Path]::GetTempPath(), [guid]::NewGuid()))
$zip   = Join-Path $tmp $asset

Invoke-WebRequest "$base/$asset" -OutFile $zip
# Optional checksum
try {
    $sumUrl = "$base/$asset.sha256"
    $expected = (Invoke-WebRequest $sumUrl -UseBasicParsing).Content.Trim()
    $actual = (Get-FileHash $zip -Algorithm SHA256).Hash
    if ($expected -and $expected.Split(" ")[0] -ne $actual) { throw "Checksum mismatch" }
} catch {}

Expand-Archive $zip -DestinationPath $tmp -Force
$dest = "$Env:ProgramFiles\croner"
New-Item -Force -ItemType Directory -Path $dest | Out-Null
Move-Item -Force (Join-Path $tmp "croner.exe") $dest
$bin = "$dest\croner.exe"

# Add to PATH for current user if needed
if (-not (($env:Path -split ";") -contains $dest)) {
    [Environment]::SetEnvironmentVariable("Path", $env:Path + ";" + $dest, "User")
    Write-Host "Added $dest to PATH (reopen terminal)."
}
Write-Host "Installed $bin"
