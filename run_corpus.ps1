param(
    [string]$Compiler = "ng_native.ng"
)

$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

. (Join-Path $workspace "direct_toolchain_common.ps1")

$compilerExe = Resolve-CompilerExecutable -Workspace $workspace -Compiler $Compiler -BuildTag "direct_corpus_compiler"
$cases = Read-RuntimeCorpusCases -Workspace $workspace
$results = @()

foreach ($case in $cases) {
    Write-Output "== CASE $($case.Name) =="
    $caseDir = Join-Path $workspace ("direct_corpus_case_" + $case.Name)
    $result = Invoke-RuntimeCorpusCase -Workspace $workspace -CompilerExe $compilerExe -Case $case -CaseDir $caseDir
    $results += $result
    if ($result.Ok) {
        Write-Output "PASS $($result.Name)"
    } else {
        Write-Output "FAIL $($result.Name)"
    }
}

$failed = @($results | Where-Object { -not $_.Ok })
if ($failed.Count -eq 0) {
    Write-Output "CORPUS PASS"
    exit 0
}

Write-Output "CORPUS FAIL"
foreach ($result in $failed) {
    Write-Output "== FAIL $($result.Name) =="
    Write-Output $result.Detail
}
exit 1
