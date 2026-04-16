param(
    [int]$MaxParallel = 2,
    [string]$Compiler = "ng_selfhost_clean.exe"
)

$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

. (Join-Path $workspace "direct_toolchain_common.ps1")

function Get-Sha256Hex {
    param([Parameter(Mandatory = $true)][string]$Path)

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $sha = [System.Security.Cryptography.SHA256]::Create()
        try {
            $hash = $sha.ComputeHash($stream)
        } finally {
            $sha.Dispose()
        }
    } finally {
        $stream.Dispose()
    }

    return ([System.BitConverter]::ToString($hash)).Replace("-", "")
}

$trustedCompiler = Resolve-CompilerExecutable -Workspace $workspace -Compiler $Compiler -BuildTag "direct_vertical_trusted"
$compilerSource = Resolve-WorkspacePath -Workspace $workspace -Path "ng_native.ng"
$gen1Dir = Join-Path $workspace "bootstrap_gen1"
$gen2Dir = Join-Path $workspace "bootstrap_gen2"
$gen1Exe = Join-Path $workspace "ngc_gen1_bootstrap.exe"
$gen2Exe = Join-Path $workspace "ngc_gen2_bootstrap.exe"
$reportPath = Join-Path $workspace "bootstrap_vertical_report.md"
$auditPath = Join-Path $workspace "BOOTSTRAP_REPRO_AUDIT.md"

if (-not (Test-Path -LiteralPath $compilerSource)) {
    throw "missing compiler source: $compilerSource"
}

Write-Output "VERTICAL: trusted direct stack"
& powershell -ExecutionPolicy Bypass -File (Join-Path $workspace "run_direct_stack.ps1") -Compiler $trustedCompiler
if ($LASTEXITCODE -ne 0) {
    throw "trusted direct stack failed with exit $LASTEXITCODE"
}

Write-Output "VERTICAL: trusted -> gen1"
$gen1Output = Invoke-NativeCompilerBuild -CompilerExe $trustedCompiler -Workspace $workspace -Program "ng_native.ng" -OutDir $gen1Dir
Copy-Item -LiteralPath $gen1Output -Destination $gen1Exe -Force

Write-Output "VERTICAL: direct stack gen1"
& powershell -ExecutionPolicy Bypass -File (Join-Path $workspace "run_direct_stack.ps1") -Compiler $gen1Exe
if ($LASTEXITCODE -ne 0) {
    throw "gen1 direct stack failed with exit $LASTEXITCODE"
}

Write-Output "VERTICAL: corpus gen1"
& powershell -ExecutionPolicy Bypass -File (Join-Path $workspace "run_corpus_parallel.ps1") -MaxParallel $MaxParallel -Compiler $gen1Exe
if ($LASTEXITCODE -ne 0) {
    throw "gen1 corpus failed with exit $LASTEXITCODE"
}

Write-Output "VERTICAL: gen1 -> gen2"
$gen2Output = Invoke-NativeCompilerBuild -CompilerExe $gen1Exe -Workspace $workspace -Program "ng_native.ng" -OutDir $gen2Dir
Copy-Item -LiteralPath $gen2Output -Destination $gen2Exe -Force

Write-Output "VERTICAL: direct stack gen2"
& powershell -ExecutionPolicy Bypass -File (Join-Path $workspace "run_direct_stack.ps1") -Compiler $gen2Exe
if ($LASTEXITCODE -ne 0) {
    throw "gen2 direct stack failed with exit $LASTEXITCODE"
}

Write-Output "VERTICAL: corpus gen2"
& powershell -ExecutionPolicy Bypass -File (Join-Path $workspace "run_corpus_parallel.ps1") -MaxParallel $MaxParallel -Compiler $gen2Exe
if ($LASTEXITCODE -ne 0) {
    throw "gen2 corpus failed with exit $LASTEXITCODE"
}

Write-Output "VERTICAL: compare gen1/gen2"
$gen1Bytes = [System.IO.File]::ReadAllBytes($gen1Exe)
$gen2Bytes = [System.IO.File]::ReadAllBytes($gen2Exe)
if ($gen1Bytes.Length -ne $gen2Bytes.Length) {
    throw "fixed-point mismatch: gen1 $($gen1Bytes.Length) bytes, gen2 $($gen2Bytes.Length) bytes"
}
for ($i = 0; $i -lt $gen1Bytes.Length; $i++) {
    if ($gen1Bytes[$i] -ne $gen2Bytes[$i]) {
        throw "fixed-point mismatch at byte offset $i"
    }
}

$trustedHash = Get-Sha256Hex -Path $trustedCompiler
$gen1Hash = Get-Sha256Hex -Path $gen1Exe
$gen2Hash = Get-Sha256Hex -Path $gen2Exe
$timestamp = [DateTime]::UtcNow.ToString("o")
$trustedMatchesGen1 = ($trustedHash -eq $gen1Hash)

$report = @(
    "# Direct Bootstrap Vertical Report"
    ""
    "timestamp_utc: ``$timestamp``"
    "max_parallel: ``$MaxParallel``"
    ""
    "## Result"
    ""
    "- ``trusted direct stack`` PASS"
    "- ``trusted -> gen1`` PASS"
    "- ``gen1 direct stack`` PASS"
    "- ``gen1 corpus`` PASS"
    "- ``gen1 -> gen2`` PASS"
    "- ``gen2 direct stack`` PASS"
    "- ``gen2 corpus`` PASS"
    "- ``gen1 == gen2`` PASS"
    ""
    "## Artifacts"
    ""
    "- trusted compiler: ``$trustedCompiler``"
    "- gen1 bootstrap: ``$gen1Exe``"
    "- gen2 bootstrap: ``$gen2Exe``"
    "- bootstrap artifact size: ``$($gen1Bytes.Length)`` bytes"
    ""
    "## Hashes"
    ""
    "- trusted SHA-256: ``$trustedHash``"
    "- gen1 SHA-256: ``$gen1Hash``"
    "- gen2 SHA-256: ``$gen2Hash``"
) -join [Environment]::NewLine
[System.IO.File]::WriteAllText($reportPath, $report, [System.Text.Encoding]::ASCII)

$audit = @(
    "# Direct Bootstrap Repro Audit"
    ""
    "## Fixed-Point Result"
    ""
    "- trusted == gen1: ``$trustedMatchesGen1``"
    "- gen1 == gen2: ``$true``"
    ""
    "## Hashes"
    ""
    "- trusted SHA-256: ``$trustedHash``"
    "- gen1 SHA-256: ``$gen1Hash``"
    "- gen2 SHA-256: ``$gen2Hash``"
    ""
    "## Practical Verdict"
    ""
    "- direct selfhost trust root is executable without Rust orchestration"
    "- corpus and direct stack both passed on gen1 and gen2"
    "- current bootstrap pair is byte-identical"
) -join [Environment]::NewLine
[System.IO.File]::WriteAllText($auditPath, $audit, [System.Text.Encoding]::ASCII)

Write-Output "BOOTSTRAP VERTICAL PASS: gen1 == gen2 ($($gen1Bytes.Length) bytes)"
exit 0
