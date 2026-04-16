param(
    [int]$MaxParallel = 4,
    [string]$Compiler = "ng_native.ng"
)

$ErrorActionPreference = "Stop"

$workspace = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $workspace

. (Join-Path $workspace "direct_toolchain_common.ps1")

$compilerExe = Resolve-CompilerExecutable -Workspace $workspace -Compiler $Compiler -BuildTag "direct_parallel_compiler"
$cases = Read-RuntimeCorpusCases -Workspace $workspace
$workerCount = [Math]::Min([Math]::Max($MaxParallel, 1), 8)
$runnerScript = Join-Path $workspace "corpus_case_runner.ps1"
$active = @()
$results = @()

function Flush-CompletedCases {
    param([switch]$WaitAll)

    while ($true) {
        $completed = @($active | Where-Object { $_.Process.HasExited })
        if ($completed.Count -eq 0) {
            if ($WaitAll -and $active.Count -gt 0) {
                Start-Sleep -Milliseconds 200
                continue
            }
            break
        }

        foreach ($entry in $completed) {
            $resultPath = Join-Path $entry.CaseDir "result.json"
            if (Test-Path -LiteralPath $resultPath) {
                $result = Get-Content -LiteralPath $resultPath -Raw | ConvertFrom-Json
            } else {
                $stdout = Read-FileSafe -Path $entry.StdoutPath
                $stderr = Read-FileSafe -Path $entry.StderrPath
                $detail = @(
                    "runner failed before producing result.json"
                    "== runner stdout =="
                    $stdout
                    "== runner stderr =="
                    $stderr
                ) -join [Environment]::NewLine
                $result = [PSCustomObject]@{
                    Name = $entry.Case.Name
                    Ok = $false
                    Detail = $detail
                }
            }

            $script:results += $result
            if ($result.Ok) {
                Write-Output "PASS $($result.Name)"
            } else {
                Write-Output "FAIL $($result.Name)"
            }
        }

        $script:active = @($active | Where-Object { -not $_.Process.HasExited })
    }
}

foreach ($case in $cases) {
    while ($active.Count -ge $workerCount) {
        Flush-CompletedCases
        if ($active.Count -ge $workerCount) {
            Start-Sleep -Milliseconds 200
        }
    }

    $caseDir = Join-Path $workspace ("direct_parallel_case_" + $case.Name)
    New-Item -ItemType Directory -Force -Path $caseDir | Out-Null

    $caseJson = Join-Path $caseDir "case.json"
    $stdoutPath = Join-Path $caseDir "runner_stdout.txt"
    $stderrPath = Join-Path $caseDir "runner_stderr.txt"

    $case | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $caseJson -Encoding ASCII
    Remove-Item -LiteralPath $stdoutPath, $stderrPath -Force -ErrorAction SilentlyContinue

    $process = Start-Process -FilePath "powershell" `
        -ArgumentList @(
            "-ExecutionPolicy", "Bypass",
            "-File", $runnerScript,
            "-Workspace", $workspace,
            "-CompilerExe", $compilerExe,
            "-CaseJson", $caseJson
        ) `
        -WorkingDirectory $workspace `
        -WindowStyle Hidden `
        -PassThru `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath

    $active += [PSCustomObject]@{
        Process = $process
        Case = $case
        CaseDir = $caseDir
        StdoutPath = $stdoutPath
        StderrPath = $stderrPath
    }
}

Flush-CompletedCases -WaitAll
$results = @($results | Sort-Object Name)
$failed = @($results | Where-Object { -not $_.Ok })

if ($failed.Count -eq 0) {
    Write-Output "PARALLEL CORPUS PASS"
    exit 0
}

Write-Output "PARALLEL CORPUS FAIL"
foreach ($result in $failed) {
    Write-Output "== FAIL $($result.Name) =="
    Write-Output $result.Detail
}
exit 1
