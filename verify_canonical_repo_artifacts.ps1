Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

$expectedArtifacts = @(
    @{ Path = "ng_selfhost_clean.exe"; Length = 175104; Hash = "D0D08BF340B220D904064DF4FFF87D2CD987958E760BFBE217E4CAE376C71653" },
    @{ Path = "test_input.txt"; Length = 18; Hash = "2467888FF1B92451A42FFFCC8A53170986D6105520CACC7D6D5DB4F1AA1B6E6D" },
    @{ Path = "tests\\reference\\corpus_repro_compiler_ops_ref.exe"; Length = 5120; Hash = "66497B50F5A017992E450CCF37EC1787A5669BAA3EEEDF30CE31ED57FDB8886A" },
    @{ Path = "tests\\reference\\corpus_repro_exit0_ref.exe"; Length = 4608; Hash = "E06B27C2AB4D192F4BEE44FF0D422A4CE03B9EBD36A198AD5C3E28AC46094CA0" },
    @{ Path = "tests\\reference\\corpus_repro_fileio_native_ref.exe"; Length = 4608; Hash = "EA9392563F6569FD28C0ECE391C7B600F24F41C129E24F47D92B3A6DF7306A71" },
    @{ Path = "tests\\reference\\corpus_repro_strops_native_ref.exe"; Length = 4608; Hash = "3C05CC2D8A50766D2AD50FDBED79730D0C23773AF2C3C8522313A57A69B95F16" },
    @{ Path = "tests\\reference\\direct_repro_buf_native_ref.exe"; Length = 4608; Hash = "3E55CCB6D5F75530FE1F76FD30EAB86D5DE4FBBDDF8E842F71A03028159ED310" },
    @{ Path = "tests\\reference\\direct_repro_exit42_ref.exe"; Length = 4608; Hash = "C25CF858290BE94292AC66307DFD7D3C8D6F9EFB9F779B03891E13396D414DE4" },
    @{ Path = "tests\\reference\\direct_repro_main_source_ref.exe"; Length = 4608; Hash = "014248389D53F5D2BA6915D8B1C4189061A968846C79F9E92D7EEF7CE9243923" },
    @{ Path = "tests\\reference\\direct_repro_struct_native_ref.exe"; Length = 4608; Hash = "41CABD57EC8F8C3D0A0A97EBAA70ED99D0FE7F96D50702CE220CB09098095F2C" }
)

$expectedReferencePaths = @(
    "tests\\reference\\corpus_repro_compiler_ops_ref.exe",
    "tests\\reference\\corpus_repro_exit0_ref.exe",
    "tests\\reference\\corpus_repro_fileio_native_ref.exe",
    "tests\\reference\\corpus_repro_strops_native_ref.exe",
    "tests\\reference\\direct_repro_buf_native_ref.exe",
    "tests\\reference\\direct_repro_exit42_ref.exe",
    "tests\\reference\\direct_repro_main_source_ref.exe",
    "tests\\reference\\direct_repro_struct_native_ref.exe"
)

function Assert-ExpectedArtifact {
    param(
        [Parameter(Mandatory = $true)][hashtable]$Artifact
    )

    $relativePath = $Artifact.Path
    $fullPath = Join-Path $workspace $relativePath

    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "missing canonical artifact: $relativePath"
    }

    $item = Get-Item -LiteralPath $fullPath
    if ($item.Length -ne $Artifact.Length) {
        throw "unexpected size for ${relativePath}: expected $($Artifact.Length), got $($item.Length)"
    }

    $actualHash = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash
    if ($actualHash -ne $Artifact.Hash) {
        throw "unexpected SHA-256 for ${relativePath}: expected $($Artifact.Hash), got $actualHash"
    }

    [pscustomobject]@{
        Path = $relativePath
        Length = $item.Length
        Hash = $actualHash
    }
}

$referenceDir = Join-Path $workspace "tests\\reference"
if (-not (Test-Path -LiteralPath $referenceDir -PathType Container)) {
    throw "missing canonical reference directory: tests\\reference"
}

$actualReferencePaths = @(Get-ChildItem -LiteralPath $referenceDir -File -Filter *.exe |
    Sort-Object Name |
    ForEach-Object { "tests\\reference\\$($_.Name)" })

$unexpectedReferencePaths = @($actualReferencePaths | Where-Object { $_ -notin $expectedReferencePaths })
if ($unexpectedReferencePaths.Count -gt 0) {
    throw "unexpected reference artifacts: $($unexpectedReferencePaths -join ', ')"
}

$missingReferencePaths = @($expectedReferencePaths | Where-Object { $_ -notin $actualReferencePaths })
if ($missingReferencePaths.Count -gt 0) {
    throw "missing reference artifacts: $($missingReferencePaths -join ', ')"
}

$verified = foreach ($artifact in $expectedArtifacts) {
    Assert-ExpectedArtifact -Artifact $artifact
}

$verifyPeLayoutScript = Join-Path $workspace "verify_pe_layout_contract.ps1"
if (-not (Test-Path -LiteralPath $verifyPeLayoutScript -PathType Leaf)) {
    throw "missing PE layout verifier: verify_pe_layout_contract.ps1"
}

$verifiedExePaths = @($verified |
    Where-Object { $_.Path -like "*.exe" } |
    ForEach-Object { Join-Path $workspace $_.Path })

Write-Output "CANONICAL REPO ARTIFACTS PASS"
Write-Output "verified: $($verified.Count)"
foreach ($entry in $verified) {
    Write-Output " - $($entry.Path) [$($entry.Length)] $($entry.Hash)"
}

& $verifyPeLayoutScript -Path $verifiedExePaths
