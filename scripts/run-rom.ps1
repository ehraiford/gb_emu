<#
.SYNOPSIS
    Lists the ROMs in a directory, asks which one to run, and launches the emulator on it.
.DESCRIPTION
    Deliberately non-recursive: test_roms/ also holds the several hundred ROMs of the blargg and
    mooneye suites, and those are driven by the test harnesses rather than by hand.
#>
param(
    [string]$RomDir = "test_roms",
    [switch]$DebugBuild,
    # Skips the prompt when you already know which entry you want.
    [int]$Choice = 0
)

$ErrorActionPreference = "Stop"

$CART_TYPES = @{
    0x00 = "ROM ONLY"; 0x01 = "MBC1"; 0x02 = "MBC1+RAM"; 0x03 = "MBC1+RAM+BAT"
    0x05 = "MBC2"; 0x06 = "MBC2+BAT"; 0x08 = "ROM+RAM"; 0x09 = "ROM+RAM+BAT"
    0x0F = "MBC3+TIMER+BAT"; 0x10 = "MBC3+TIMER+RAM+BAT"; 0x11 = "MBC3"
    0x12 = "MBC3+RAM"; 0x13 = "MBC3+RAM+BAT"; 0x19 = "MBC5"; 0x1A = "MBC5+RAM"
    0x1B = "MBC5+RAM+BAT"; 0x1C = "MBC5+RUMBLE"; 0x1D = "MBC5+RUMBLE+RAM"
    0x1E = "MBC5+RUMBLE+RAM+BAT"; 0x20 = "MBC6"; 0x22 = "MBC7+SENSOR+RUMBLE+RAM+BAT"
    0xFC = "POCKET CAMERA"; 0xFD = "BANDAI TAMA5"; 0xFE = "HuC3"; 0xFF = "HuC1+RAM+BAT"
}
# Everything else is refused by Cartridge::new with UnsupportedMemoryBankController.
$SUPPORTED = @(0x00, 0x01, 0x02, 0x03, 0x05, 0x06, 0x08, 0x09)

function Read-Header([string]$Path) {
    $bytes = New-Object byte[] 0x150
    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $read = $stream.Read($bytes, 0, 0x150)
        if ($read -lt 0x150) { return $null }
    } finally { $stream.Dispose() }

    $title = ($bytes[0x134..0x142] | Where-Object { $_ -ge 0x20 -and $_ -lt 0x7F } |
        ForEach-Object { [char]$_ }) -join ""
    $code = $bytes[0x147]
    [PSCustomObject]@{
        Title    = $title.Trim()
        Type     = if ($CART_TYPES.ContainsKey([int]$code)) { $CART_TYPES[[int]$code] } else { "UNKNOWN ($('{0:X2}' -f $code))" }
        Runnable = $SUPPORTED -contains [int]$code
        SizeKb   = if ($bytes[0x148] -le 0x08) { 32 -shl $bytes[0x148] } else { 0 }
    }
}

if (-not (Test-Path -LiteralPath $RomDir)) { throw "No such directory: $RomDir" }

$roms = Get-ChildItem -LiteralPath $RomDir -File |
    Where-Object { $_.Extension -in ".gb", ".gbc" } | Sort-Object Name
if ($roms.Count -eq 0) { throw "No .gb/.gbc files directly in $RomDir" }

$entries = foreach ($rom in $roms) {
    $header = Read-Header $rom.FullName
    [PSCustomObject]@{
        File     = $rom
        Title    = if ($header) { $header.Title } else { "<no header>" }
        Type     = if ($header) { $header.Type } else { "too small" }
        SizeKb   = if ($header) { $header.SizeKb } else { 0 }
        Runnable = [bool]$header -and $header.Runnable
    }
}

if ($Choice -lt 1 -or $Choice -gt $entries.Count) {
    Write-Host ""
    Write-Host "ROMs in $RomDir" -ForegroundColor Cyan
    for ($i = 0; $i -lt $entries.Count; $i++) {
        $e = $entries[$i]
        $colour = if ($e.Runnable) { "White" } else { "DarkGray" }
        $flag = if ($e.Runnable) { " " } else { "!" }
        Write-Host ("{0}{1,3}. {2,-18} {3,-22} {4,5}KB  {5}" -f `
            $flag, ($i + 1), $e.Title, $e.Type, $e.SizeKb, $e.File.Name) -ForegroundColor $colour
    }
    Write-Host "`n! = controller not implemented yet, will refuse to load" -ForegroundColor DarkGray
    Write-Host ""

    $Choice = 0
    while ($Choice -lt 1 -or $Choice -gt $entries.Count) {
        $answer = Read-Host "Which ROM? (1-$($entries.Count), or q to quit)"
        if ($answer -match '^\s*[qQ]') { return }
        [int]::TryParse($answer, [ref]$Choice) | Out-Null
    }
}

$selected = $entries[$Choice - 1]
if (-not $selected.Runnable) {
    Write-Host "Heads up: $($selected.Type) isn't implemented, expect a load failure." -ForegroundColor Yellow
}

$cargoArgs = @("run")
if (-not $DebugBuild) { $cargoArgs += "--release" }
$cargoArgs += @("--", "run", "--tile-map-viewer", $selected.File.FullName)

Write-Host "`n> cargo $($cargoArgs -join ' ')`n" -ForegroundColor DarkGray
& cargo @cargoArgs
