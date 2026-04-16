param(
    [string]$Compiler = "ng_selfhost_clean.exe"
)

$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

. (Join-Path $workspace "direct_toolchain_common.ps1")

$compilerExe = Resolve-CompilerExecutable -Workspace $workspace -Compiler $Compiler -BuildTag "direct_stack_compiler"
$suites = @(
    @{ Name = "smoke_verify"; Path = "tests/programs/tooling_smoke_verify.txt" },
    @{ Name = "dispatch_guard"; Path = "tests/programs/tooling_dispatch_guard.txt" },
    @{ Name = "fast_check"; Path = "tests/programs/tooling_fast_check.txt" },
    @{ Name = "corpus_compile"; Path = "tests/programs/tooling_corpus_compile.txt" },
    @{ Name = "direct_repro"; Path = "tests/programs/tooling_direct_repro.txt" },
    @{ Name = "direct_corpus_repro"; Path = "tests/programs/tooling_direct_corpus_repro.txt" }
)

foreach ($suite in $suites) {
    Write-Output "DIRECT STACK: $($suite.Name)"
    & $compilerExe $suite.Path
    if ($LASTEXITCODE -ne 0) {
        throw "direct stack suite failed: $($suite.Name) (exit $LASTEXITCODE)"
    }
}

Write-Output "DIRECT STACK: direct_fixed_point"
$fixedPointDir = Join-Path $workspace "direct_fixed_point_compiler"
$fixedPointOutput = Invoke-NativeCompilerBuild -CompilerExe $compilerExe -Workspace $workspace -Program "ng_native.ng" -OutDir $fixedPointDir
$fixedPointArtifact = Join-Path $workspace "direct_fixed_point_selfhost_clean.exe"
$fixedPointExpected = $compilerExe
Copy-Item -LiteralPath $fixedPointOutput -Destination $fixedPointArtifact -Force

$actualBytes = [System.IO.File]::ReadAllBytes($fixedPointArtifact)
$expectedBytes = [System.IO.File]::ReadAllBytes($fixedPointExpected)
if ($actualBytes.Length -ne $expectedBytes.Length) {
    throw "direct fixed point mismatch: expected $($expectedBytes.Length) bytes, got $($actualBytes.Length)"
}
for ($i = 0; $i -lt $actualBytes.Length; $i++) {
    if ($actualBytes[$i] -ne $expectedBytes[$i]) {
        throw "direct fixed point mismatch at byte offset $i"
    }
}
Write-Output "DIRECT FIXED POINT P selfhost_clean"

Write-Output "DIRECT STACK PASS"
exit 0
