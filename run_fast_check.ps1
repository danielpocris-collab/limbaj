param(
    [string]$Compiler = "ng_selfhost_clean.exe"
)

$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

. (Join-Path $workspace "direct_toolchain_common.ps1")

$compilerExe = Resolve-CompilerExecutable -Workspace $workspace -Compiler $Compiler -BuildTag "direct_fast_check_compiler"
& $compilerExe "tests/programs/tooling_fast_check.txt"
exit $LASTEXITCODE
