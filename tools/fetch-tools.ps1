<#
.SYNOPSIS
  Fills src-tauri\tools\ with the tools Video Vacuum ships (D-32): yt-dlp,
  Deno and the small FFmpeg build. Downloads exactly what src-tauri\tools.json
  pins and refuses any file whose SHA-256 doesn't match.

.DESCRIPTION
  Run before `npm run tauri dev` or `npm run tauri build` on a fresh checkout.
  The Release workflow runs it too, so local and released installers carry
  the same files. Re-running is safe: files already in place with the right
  checksum aren't downloaded again.

.PARAMETER FfmpegZip
  Use a local copy of the FFmpeg zip instead of downloading it (for example
  the artifact of the "Build FFmpeg" workflow before it's published). Its
  SHA-256 must still match tools.json, unless -AllowUnpinnedFfmpeg is given.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File tools\fetch-tools.ps1
#>
param(
  [string]$FfmpegZip,
  [switch]$AllowUnpinnedFfmpeg
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
# Windows PowerShell 5.1 defaults to older TLS on some systems; GitHub needs 1.2+.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$root = Split-Path -Parent $PSScriptRoot
$pinsPath = Join-Path $root 'src-tauri\tools.json'
$dest = Join-Path $root 'src-tauri\tools'
$licenses = Join-Path $dest 'licenses'
$work = Join-Path ([IO.Path]::GetTempPath()) ("video-vacuum-tools-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Force -Path $dest, $licenses, $work | Out-Null

function Get-Sha256([string]$path) {
  (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
}

try {
  $pins = (Get-Content -Raw -LiteralPath $pinsPath | ConvertFrom-Json).tools
  foreach ($pin in $pins) {
    # Keep the real extension: Windows PowerShell 5.1's Expand-Archive only accepts ".zip" files.
    $download = Join-Path $work ($pin.id + '-download.' + $pin.download.kind)
    $expected = $pin.download.sha256.ToLowerInvariant()

    if ($pin.id -eq 'ffmpeg' -and $FfmpegZip) {
      Copy-Item -LiteralPath $FfmpegZip -Destination $download
      Write-Host "FFmpeg: using local zip $FfmpegZip"
    } else {
      Write-Host "$($pin.name) $($pin.version): downloading $($pin.download.url)"
      Invoke-WebRequest -UseBasicParsing -Uri $pin.download.url -OutFile $download
    }

    $actual = Get-Sha256 $download
    if ($actual -ne $expected) {
      if ($pin.id -eq 'ffmpeg' -and $AllowUnpinnedFfmpeg) {
        Write-Warning "FFmpeg zip SHA-256 is $actual (tools.json says $expected); allowed by -AllowUnpinnedFfmpeg."
      } else {
        throw "$($pin.name): SHA-256 mismatch. Expected $expected, got $actual. Nothing was installed."
      }
    } else {
      Write-Host "  SHA-256 OK ($actual)"
    }

    if ($pin.download.kind -eq 'zip') {
      $unpacked = Join-Path $work ($pin.id + '-unpacked')
      Expand-Archive -LiteralPath $download -DestinationPath $unpacked -Force
      foreach ($file in $pin.files) {
        $found = Get-ChildItem -LiteralPath $unpacked -Recurse -File -Filter $file | Select-Object -First 1
        if (-not $found) { throw "$($pin.name): $file isn't in the zip." }
        Copy-Item -LiteralPath $found.FullName -Destination (Join-Path $dest $file) -Force
      }
      # License and source notes that come inside the zip (FFmpeg's) go to tools\licenses.
      Get-ChildItem -LiteralPath $unpacked -Recurse -File |
        Where-Object { $_.Extension -in '.txt', '.md' } |
        ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $licenses $_.Name) -Force }
    } else {
      Copy-Item -LiteralPath $download -Destination (Join-Path $dest $pin.files[0]) -Force
    }
    Write-Host "  installed: $($pin.files -join ', ')"
  }
  Write-Host "Done: $dest"
  Get-ChildItem -LiteralPath $dest -Recurse -File | Select-Object @{n='File';e={$_.FullName.Substring($dest.Length + 1)}}, @{n='MB';e={[math]::Round($_.Length / 1MB, 1)}} | Format-Table -AutoSize
} finally {
  Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
