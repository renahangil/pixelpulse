[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9.-]{3,50}$')][string]$IdentityName,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$Publisher,
    [Parameter(Mandatory)][ValidateNotNullOrEmpty()][string]$PublisherDisplayName,
    [ValidatePattern('^[0-9]+\.[0-9]+\.[0-9]+\.0$')][string]$Version = '0.2.0.0',
    [string]$MakeAppxPath
)

$ErrorActionPreference = 'Stop'
$tauri = Split-Path -Parent $PSScriptRoot
$target = Join-Path $tauri 'target'
$executable = Join-Path $target 'release/pixelpulse.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw 'Compile primeiro: npm run tauri build -- --no-bundle'
}
$versionObject = [version]$Version
if ($versionObject.Major -eq 0 -and $versionObject.Minor -eq 0 -and $versionObject.Build -eq 0) {
    throw 'A versão do pacote deve ser maior que 0.0.0.0.'
}
if (@($versionObject.Major, $versionObject.Minor, $versionObject.Build) |
    Where-Object { $_ -gt 65535 }) {
    throw 'Cada componente da versão MSIX deve caber em 16 bits.'
}
if (-not $MakeAppxPath) {
    $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
    $sdk = Get-ChildItem -LiteralPath $sdkRoot -Directory |
        Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
        Sort-Object { [version]$_.Name } -Descending |
        Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName 'x64/makeappx.exe') } |
        Select-Object -First 1
    if (-not $sdk) { throw 'Instale o Windows SDK ou passe -MakeAppxPath.' }
    $MakeAppxPath = Join-Path $sdk.FullName 'x64/makeappx.exe'
}

# O pacote declara x64. Verificar o cabeçalho PE evita empacotar outro alvo por engano.
$reader = [IO.BinaryReader]::new([IO.File]::OpenRead($executable))
try {
    $reader.BaseStream.Position = 0x3c
    $reader.BaseStream.Position = $reader.ReadInt32()
    if ($reader.ReadUInt32() -ne 0x4550 -or $reader.ReadUInt16() -ne 0x8664) {
        throw 'O executável release não é um PE Windows x64.'
    }
} finally { $reader.Dispose() }

$output = Join-Path $target 'msix'
$stage = Join-Path $output ('stage-' + [guid]::NewGuid().ToString('N'))
$assets = Join-Path $stage 'Assets'
$null = New-Item -ItemType Directory -Path $assets
Copy-Item -LiteralPath $executable -Destination $stage
Copy-Item -LiteralPath (Join-Path (Split-Path -Parent $tauri) 'LICENSE') -Destination $stage
foreach ($name in 'StoreLogo.png', 'Square150x150Logo.png', 'Square44x44Logo.png') {
    Copy-Item -LiteralPath (Join-Path $tauri "icons/$name") -Destination $assets
}

[xml]$manifest = Get-Content (Join-Path $PSScriptRoot 'AppxManifest.xml') -Raw -Encoding utf8
$manifest.Package.Identity.SetAttribute('Name', $IdentityName)
$manifest.Package.Identity.SetAttribute('Publisher', $Publisher)
$manifest.Package.Identity.SetAttribute('Version', $Version)
$manifest.Package.Properties.PublisherDisplayName = $PublisherDisplayName
$manifest.Save((Join-Path $stage 'AppxManifest.xml'))

$package = Join-Path $output "$IdentityName-$Version-x64.msix"
& $MakeAppxPath pack /d $stage /p $package /o
if ($LASTEXITCODE -ne 0) { throw "MakeAppx falhou: $LASTEXITCODE" }
Write-Output $package
